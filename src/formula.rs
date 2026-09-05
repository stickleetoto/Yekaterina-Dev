use std::collections::HashMap;

pub const MAX_EXPR_LEN: usize = 4096;
pub const MAX_PARAMS: usize = 32;
pub const MAX_DEPTH: usize = 64;

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
            return self.vars.get(&name).copied().ok_or("VAR");
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

    fn parse_ident(&mut self) -> Result<String, &'static str> {
        let start = self.pos;
        self.pos += 1;
        while self.peek().is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_') { self.pos += 1; }
        let s = std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| "EXPR")?;
        Ok(s.to_string())
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
