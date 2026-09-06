use std::cell::RefCell;
use std::collections::HashMap;

pub const MAX_EXPR_LEN: usize = 4096;
pub const MAX_PARAMS: usize = 32;
pub const MAX_DEPTH: usize = 64;

/// A reusable variable environment for iterative evaluation.
///
/// The `num.`, `ode.`, `optimize.` and `series.` solvers evaluate one expression
/// many times, varying only a loop variable. v1.0.0 expressed that as
/// `base.clone()` followed by `insert("x".to_string(), value)` on **every**
/// evaluation, so a 10,000-step integration performed 10,001 map clones and
/// 10,001 key allocations. `ode.rk4` evaluates four times per step, and the step
/// bound is 1,000,000.
///
/// [`Env`] hoists that out of the loop: the map is built once with the loop
/// variables pre-inserted, and each iteration overwrites their values in place.
///
/// The contents handed to [`eval`] are identical to what v1.0.0 built, so
/// results are bit-identical by construction. In particular, pre-inserting a
/// name that already exists in `base` and then overwriting it reproduces
/// `clone` + `insert`, where the later insert wins.
///
/// The variable map is held behind a `RefCell` so that updating a loop variable
/// takes `&self` rather than `&mut self`.
///
/// That is not incidental. `optimization.rs` evaluates the objective twice
/// inside single expressions -- `f2(..., x1, y)? < f2(..., x2, y)?`, and a
/// `sort_by` comparator that calls it on both operands. With `&mut self` those
/// call sites would have to be broken apart into sequential `let` bindings, and
/// rewriting float expressions is exactly the kind of edit that silently changes
/// evaluation order. Interior mutability keeps every call site byte-for-byte as
/// v1.0.0 wrote it.
///
/// There is no re-entrancy: [`eval`] is a pure function over the borrowed map
/// and never calls back into `Env`. Each borrow is released before the next.
/// `Env` is a local inside synchronous engine code, never shared across threads
/// and never held across an await.
pub struct Env {
    vars: RefCell<HashMap<String, f64>>,
}

impl Env {
    /// Clone `base` once and reserve a slot for each loop variable.
    pub fn new(base: &HashMap<String, f64>, names: &[&str]) -> Self {
        let mut vars = base.clone();
        for name in names {
            vars.insert((*name).to_string(), 0.0);
        }
        Self { vars: RefCell::new(vars) }
    }

    /// Overwrite a loop variable in place. Names not reserved by [`Env::new`]
    /// are ignored, which cannot happen for the fixed loop variables the
    /// solvers use.
    pub fn set(&self, name: &str, value: f64) {
        if let Some(slot) = self.vars.borrow_mut().get_mut(name) {
            *slot = value;
        }
    }

    pub fn eval(&self, expr: &str) -> Result<f64, &'static str> {
        eval(expr, &self.vars.borrow())
    }
}

pub fn eval(expr: &str, vars: &HashMap<String, f64>) -> Result<f64, &'static str> {
    if expr.len() > MAX_EXPR_LEN { return Err("LIMIT"); }
    let mut p = Parser { bytes: expr.as_bytes(), pos: 0, vars, depth: 0 };
    let v = p.parse_expr()?;
    p.skip_ws();
    if p.pos != p.bytes.len() { return Err("EXPR"); }
    if !v.is_finite() { return Err("NONFINITE"); }
    Ok(v)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    vars: &'a HashMap<String, f64>,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn parse_expr(&mut self) -> Result<f64, &'static str> {
        self.with_depth(|p| {
            let mut v = p.parse_term()?;
            loop {
                p.skip_ws();
                if p.eat(b'+') { v += p.parse_term()?; }
                else if p.eat(b'-') { v -= p.parse_term()?; }
                else { break; }
            }
            Ok(v)
        })
    }

    fn parse_term(&mut self) -> Result<f64, &'static str> {
        let mut v = self.parse_power()?;
        loop {
            self.skip_ws();
            if self.eat(b'*') { v *= self.parse_power()?; }
            else if self.eat(b'/') {
                let rhs = self.parse_power()?;
                if rhs == 0.0 { return Err("DIV0"); }
                v /= rhs;
            } else if self.eat(b'%') {
                let rhs = self.parse_power()?;
                if rhs == 0.0 { return Err("DIV0"); }
                v %= rhs;
            } else { break; }
        }
        Ok(v)
    }

    fn parse_power(&mut self) -> Result<f64, &'static str> {
        self.with_depth(|p| {
            let left = p.parse_unary()?;
            p.skip_ws();
            if p.eat(b'^') {
                let right = p.parse_power()?;
                Ok(left.powf(right))
            } else {
                Ok(left)
            }
        })
    }

    fn parse_unary(&mut self) -> Result<f64, &'static str> {
        self.with_depth(|p| {
            p.skip_ws();
            if p.eat(b'+') { return p.parse_unary(); }
            if p.eat(b'-') { return Ok(-p.parse_unary()?); }
            p.parse_primary()
        })
    }

    fn parse_primary(&mut self) -> Result<f64, &'static str> {
        self.skip_ws();
        if self.eat(b'(') {
            let v = self.parse_expr()?;
            self.skip_ws();
            if !self.eat(b')') { return Err("EXPR"); }
            return Ok(v);
        }
        if self.peek().is_some_and(|b| b.is_ascii_digit() || b == b'.') { return self.parse_number(); }
        if self.peek().is_some_and(|b| b.is_ascii_alphabetic() || b == b'_') {
            let name = self.parse_ident()?;
            // `HashMap<String, f64>::get` accepts `&str` through `Borrow`, so the
            // identifier never needs to be turned into an owned `String`.
            return self.vars.get(name).copied().ok_or("VAR");
        }
        Err("EXPR")
    }

    fn parse_number(&mut self) -> Result<f64, &'static str> {
        let start = self.pos;
        while self.peek().is_some_and(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-')) {
            let b = self.peek().unwrap();
            if (b == b'+' || b == b'-') && self.pos > start {
                let prev = self.bytes[self.pos - 1];
                if prev != b'e' && prev != b'E' { break; }
            }
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| "EXPR")?;
        s.parse::<f64>().map_err(|_| "EXPR")
    }

    /// Borrows the identifier out of the source expression instead of copying
    /// it. v1.0.0 allocated a `String` for every identifier *occurrence*, so
    /// `x*x` cost two allocations on every evaluation, and the iterative solvers
    /// evaluate their expression up to millions of times per request.
    fn parse_ident(&mut self) -> Result<&'a str, &'static str> {
        let bytes = self.bytes;
        let start = self.pos;
        self.pos += 1;
        while self.peek().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') { self.pos += 1; }
        std::str::from_utf8(&bytes[start..self.pos]).map_err(|_| "EXPR")
    }

    fn with_depth<F>(&mut self, f: F) -> Result<f64, &'static str>
    where F: FnOnce(&mut Self) -> Result<f64, &'static str> {
        if self.depth >= MAX_DEPTH { return Err("LIMIT"); }
        self.depth += 1;
        let out = f(self);
        self.depth -= 1;
        out
    }

    fn skip_ws(&mut self) { while self.peek().is_some_and(|b| b.is_ascii_whitespace()) { self.pos += 1; } }
    fn eat(&mut self, b: u8) -> bool { if self.peek() == Some(b) { self.pos += 1; true } else { false } }
    fn peek(&self) -> Option<u8> { self.bytes.get(self.pos).copied() }
}
