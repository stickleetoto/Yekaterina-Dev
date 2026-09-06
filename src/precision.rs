use std::str::FromStr;

use bigdecimal::{BigDecimal, RoundingMode};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{Signed, ToPrimitive, Zero};
use serde_json::{Value, json};

const MAX_EXACT_INPUT_CHARS: usize = 100_000;
const MAX_EXACT_RESULT_CHARS: usize = 1_000_000;
const MAX_INT_POW_EXP: u64 = 1_000_000;
const MAX_SHIFT_BITS: u64 = 1_000_000;
const MAX_SEQUENCE: usize = 100_000;
/// Decimal places accepted by the rounding operations. Wider than any real
/// money or measurement use, narrow enough that a typo cannot ask for a
/// gigabyte of zeros.
const MAX_DEC_DIGITS: i64 = 100_000;

pub fn execute(opcode: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    match opcode {
        "int.add" => Some(int_binary(args, |a, b| a + b)),
        "int.sub" => Some(int_binary(args, |a, b| a - b)),
        "int.mul" => Some(int_binary(args, |a, b| a * b)),
        "int.div" => Some((|| {
            let (a, b) = two_bigint(args)?;
            if b.is_zero() { Err("DIV0") } else { exact_bigint(a / b) }
        })()),
        "int.mod" => Some((|| {
            let (a, b) = two_bigint(args)?;
            if b.is_zero() { Err("DIV0") } else { exact_bigint(a % b) }
        })()),
        "int.gcd" => Some((|| {
            let (a, b) = two_bigint(args)?;
            exact_bigint(a.gcd(&b))
        })()),
        "int.lcm" => Some((|| {
            let (a, b) = two_bigint(args)?;
            exact_bigint(a.lcm(&b))
        })()),
        "int.pow" => Some((|| {
            if args.len() != 2 { return Err("ARG"); }
            let a = bigint(&args[0])?;
            let exp = args[1].as_u64().ok_or("TYPE")?;
            if exp > MAX_INT_POW_EXP { return Err("LIMIT"); }
            if estimated_pow_digits(&a, exp) > MAX_EXACT_RESULT_CHARS { return Err("OUT_LIMIT"); }
            exact_bigint(a.pow(exp as u32))
        })()),
        "int.abs" => Some((|| exact_bigint(one_bigint(args)?.abs()))()),
        "int.neg" => Some((|| exact_bigint(-one_bigint(args)?))()),
        "int.sign" => Some((|| Ok(json!(sign_of(&one_bigint(args)?))))()),
        "int.cmp" => Some((|| {
            let (a, b) = two_bigint(args)?;
            Ok(json!(ordering_code(a.cmp(&b))))
        })()),
        // Truncated division: the quotient rounds toward zero and the remainder
        // takes the sign of the dividend, matching int.div and int.mod.
        "int.divmod" => Some((|| {
            let (a, b) = two_bigint(args)?;
            if b.is_zero() { return Err("DIV0"); }
            let (q, r) = (&a / &b, &a % &b);
            Ok(json!([exact_str(q)?, exact_str(r)?]))
        })()),
        // Floor division: the quotient rounds toward negative infinity and the
        // remainder takes the sign of the divisor. -7 by 2 is -4 here and -3
        // under int.div; both are correct, for different definitions, and which
        // one a caller wants is not guessable, so both are offered.
        "int.div_floor" => Some((|| {
            let (a, b) = two_bigint(args)?;
            if b.is_zero() { return Err("DIV0"); }
            exact_bigint(a.div_floor(&b))
        })()),
        "int.mod_floor" => Some((|| {
            let (a, b) = two_bigint(args)?;
            if b.is_zero() { return Err("DIV0"); }
            exact_bigint(a.mod_floor(&b))
        })()),
        "int.divmod_floor" => Some((|| {
            let (a, b) = two_bigint(args)?;
            if b.is_zero() { return Err("DIV0"); }
            let (q, r) = a.div_mod_floor(&b);
            Ok(json!([exact_str(q)?, exact_str(r)?]))
        })()),
        "int.sqrt" => Some((|| {
            let a = one_bigint(args)?;
            if a.is_negative() { return Err("DOMAIN"); }
            exact_bigint(a.sqrt())
        })()),
        "int.shl" => Some((|| { let (v, n) = shift(args)?; exact_bigint(v << n) })()),
        "int.shr" => Some((|| { let (v, n) = shift(args)?; exact_bigint(v >> n) })()),
        "int.bit_length" => Some((|| Ok(json!(one_bigint(args)?.bits())))()),
        "int.min" => Some((|| exact_bigint(fold_bigint(args, |a, b| if b < a { b } else { a })?))()),
        "int.max" => Some((|| exact_bigint(fold_bigint(args, |a, b| if b > a { b } else { a })?))()),
        "int.sum" => Some((|| exact_bigint(fold_bigint(args, |a, b| a + b)?))()),
        "int.product" => Some((|| exact_bigint(fold_bigint(args, |a, b| a * b)?))()),
        // Arbitrary-precision modular arithmetic. alg.mod_pow and
        // alg.mod_inverse are the u64/i64 versions and fail above that range;
        // these have no such ceiling, which is what key-sized numbers need.
        "int.mod_pow" => Some((|| {
            if args.len() != 3 { return Err("ARG"); }
            let base = bigint(&args[0])?;
            let exp = bigint(&args[1])?;
            let modulus = bigint(&args[2])?;
            if modulus.is_zero() { return Err("DIV0"); }
            if exp.is_negative() { return Err("DOMAIN"); }
            exact_bigint(base.modpow(&exp, &modulus))
        })()),
        "int.mod_inverse" => Some((|| {
            let (a, m) = two_bigint(args)?;
            if !m.is_positive() { return Err("DOMAIN"); }
            match a.modinv(&m) {
                Some(inv) => exact_bigint(inv),
                None => Err("DOMAIN"),   // not coprime: no inverse exists
            }
        })()),
        "dec.add" => Some(dec_binary(args, |a, b| a + b)),
        "dec.sub" => Some(dec_binary(args, |a, b| a - b)),
        "dec.mul" => Some(dec_binary(args, |a, b| a * b)),
        "dec.div" => Some((|| {
            let (a, b) = two_decimal(args)?;
            if b.is_zero() { Err("DIV0") } else { exact_decimal(a / b) }
        })()),
        "dec.mod" => Some((|| {
            let (a, b) = two_decimal(args)?;
            if b.is_zero() { Err("DIV0") } else { exact_decimal(a % b) }
        })()),
        "dec.abs" => Some((|| exact_decimal(one_decimal(args)?.abs()))()),
        "dec.neg" => Some((|| exact_decimal(-one_decimal(args)?))()),
        "dec.cmp" => Some((|| {
            let (a, b) = two_decimal(args)?;
            Ok(json!(ordering_code(a.cmp(&b))))
        })()),
        // Ties away from zero: 2.5 to 3, -2.5 to -3. What a spreadsheet and
        // most invoicing rules do.
        "dec.round" => Some(round_to(args, RoundingMode::HalfUp)),
        // Ties to even: 2.5 to 2, 3.5 to 4. Banker's rounding, which does not
        // bias a long column of sums upward.
        "dec.round_even" => Some(round_to(args, RoundingMode::HalfEven)),
        "dec.floor" => Some((|| exact_decimal(one_decimal(args)?.with_scale_round(0, RoundingMode::Floor)))()),
        "dec.ceil" => Some((|| exact_decimal(one_decimal(args)?.with_scale_round(0, RoundingMode::Ceiling)))()),
        "dec.trunc" => Some((|| exact_decimal(one_decimal(args)?.with_scale_round(0, RoundingMode::Down)))()),
        "dec.scale" => Some((|| Ok(json!(one_decimal(args)?.fractional_digit_count().max(0))))()),
        "dec.min" => Some((|| exact_decimal(fold_decimal(args, |a, b| if b < a { b } else { a })?))()),
        "dec.max" => Some((|| exact_decimal(fold_decimal(args, |a, b| if b > a { b } else { a })?))()),
        "dec.sum" => Some((|| exact_decimal(fold_decimal(args, |a, b| a + b)?))()),
        "dec.product" => Some((|| exact_decimal(fold_decimal(args, |a, b| a * b)?))()),
        "dec.pow" => Some(dec_pow(args)),
        "dec.to_number" => Some((|| {
            let d = one_decimal(args)?;
            match d.to_f64() {
                Some(x) if x.is_finite() => Ok(json!(x)),
                _ => Err("NONFINITE"),
            }
        })()),
        _ => None,
    }
}

fn one_bigint(args: &[Value]) -> Result<BigInt, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    bigint(&args[0])
}

fn one_decimal(args: &[Value]) -> Result<BigDecimal, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    decimal(&args[0])
}

fn sign_of(v: &BigInt) -> i32 {
    if v.is_zero() { 0 } else if v.is_negative() { -1 } else { 1 }
}

fn ordering_code(o: std::cmp::Ordering) -> i32 {
    match o {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn exact_str(value: BigInt) -> Result<String, &'static str> {
    let s = value.to_string();
    if s.len() > MAX_EXACT_RESULT_CHARS { return Err("OUT_LIMIT"); }
    Ok(s)
}

/// A shift is bounded by the digits it can produce, not by the shift count
/// alone: shifting a already-large number is the expensive case. Three bits per
/// decimal digit is a deliberate under-estimate of log2(10), so the guard trips
/// before the result can exceed the result budget rather than after.
fn shift(args: &[Value]) -> Result<(BigInt, usize), &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let value = bigint(&args[0])?;
    let bits = args[1].as_u64().ok_or("TYPE")?;
    if bits > MAX_SHIFT_BITS { return Err("LIMIT"); }
    if value.bits().saturating_add(bits) / 3 > MAX_EXACT_RESULT_CHARS as u64 {
        return Err("OUT_LIMIT");
    }
    Ok((value, bits as usize))
}

fn sequence(args: &[Value]) -> Result<&Vec<Value>, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    let items = args[0].as_array().ok_or("TYPE")?;
    if items.is_empty() { return Err("EMPTY"); }
    if items.len() > MAX_SEQUENCE { return Err("LIMIT"); }
    Ok(items)
}

fn fold_bigint<F>(args: &[Value], f: F) -> Result<BigInt, &'static str>
where
    F: Fn(BigInt, BigInt) -> BigInt,
{
    let items = sequence(args)?;
    let mut acc = bigint(&items[0])?;
    for item in &items[1..] {
        acc = f(acc, bigint(item)?);
        if acc.to_string().len() > MAX_EXACT_RESULT_CHARS { return Err("OUT_LIMIT"); }
    }
    Ok(acc)
}

fn fold_decimal<F>(args: &[Value], f: F) -> Result<BigDecimal, &'static str>
where
    F: Fn(BigDecimal, BigDecimal) -> BigDecimal,
{
    let items = sequence(args)?;
    let mut acc = decimal(&items[0])?;
    for item in &items[1..] {
        acc = f(acc, decimal(item)?);
    }
    Ok(acc)
}

fn round_to(args: &[Value], mode: RoundingMode) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let value = decimal(&args[0])?;
    let digits = args[1].as_i64().ok_or("TYPE")?;
    if !(-MAX_DEC_DIGITS..=MAX_DEC_DIGITS).contains(&digits) { return Err("LIMIT"); }
    exact_decimal(value.with_scale_round(digits, mode))
}

/// Exact decimal power for a non-negative whole exponent. A negative exponent
/// would need a rounding rule to terminate, and this family's contract is that
/// every result is exact, so it is refused rather than silently approximated.
fn dec_pow(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    let base = decimal(&args[0])?;
    let exp = args[1].as_u64().ok_or("TYPE")?;
    if exp > MAX_INT_POW_EXP { return Err("LIMIT"); }
    let mut acc = BigDecimal::from(1);
    for _ in 0..exp {
        acc *= &base;
        if acc.digits() > MAX_EXACT_RESULT_CHARS as u64 { return Err("OUT_LIMIT"); }
    }
    exact_decimal(acc)
}


fn bigint(v: &Value) -> Result<BigInt, &'static str> {
    match v {
        Value::String(s) => {
            let s = s.trim();
            if s.len() > MAX_EXACT_INPUT_CHARS { return Err("LIMIT"); }
            BigInt::from_str(s).map_err(|_| "TYPE")
        }
        Value::Number(n) => BigInt::from_str(&n.to_string()).map_err(|_| "TYPE"),
        _ => Err("TYPE"),
    }
}

fn decimal(v: &Value) -> Result<BigDecimal, &'static str> {
    match v {
        Value::String(s) => {
            let s = s.trim();
            if s.len() > MAX_EXACT_INPUT_CHARS { return Err("LIMIT"); }
            BigDecimal::from_str(s).map_err(|_| "TYPE")
        }
        Value::Number(n) => BigDecimal::from_str(&n.to_string()).map_err(|_| "TYPE"),
        _ => Err("TYPE"),
    }
}

fn two_bigint(args: &[Value]) -> Result<(BigInt, BigInt), &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    Ok((bigint(&args[0])?, bigint(&args[1])?))
}

fn two_decimal(args: &[Value]) -> Result<(BigDecimal, BigDecimal), &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    Ok((decimal(&args[0])?, decimal(&args[1])?))
}

fn int_binary<F>(args: &[Value], f: F) -> Result<Value, &'static str>
where
    F: FnOnce(BigInt, BigInt) -> BigInt,
{
    let (a, b) = two_bigint(args)?;
    exact_bigint(f(a, b))
}

fn dec_binary<F>(args: &[Value], f: F) -> Result<Value, &'static str>
where
    F: FnOnce(BigDecimal, BigDecimal) -> BigDecimal,
{
    let (a, b) = two_decimal(args)?;
    exact_decimal(f(a, b))
}

fn exact_bigint(value: BigInt) -> Result<Value, &'static str> {
    exact_string(value.to_string())
}

fn exact_decimal(value: BigDecimal) -> Result<Value, &'static str> {
    exact_string(value.normalized().to_string())
}

fn exact_string(value: String) -> Result<Value, &'static str> {
    if value.len() > MAX_EXACT_RESULT_CHARS { return Err("OUT_LIMIT"); }
    Ok(json!(value))
}

fn estimated_pow_digits(base: &BigInt, exp: u64) -> usize {
    if exp == 0 || base.is_zero() { return 1; }
    let digits = base.to_string().trim_start_matches('-').len();
    if digits <= 1 { return 1; }
    (digits - 1).saturating_mul(exp as usize).saturating_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(op: &str, args: &[Value]) -> Result<Value, &'static str> {
        execute(op, args).expect("op not routed")
    }
    fn s(op: &str, args: &[Value]) -> String {
        call(op, args).expect("op errored").as_str().unwrap().to_string()
    }
    fn i(op: &str, args: &[Value]) -> i64 {
        call(op, args).expect("op errored").as_i64().unwrap()
    }
    fn err(op: &str, args: &[Value]) -> &'static str {
        call(op, args).unwrap_err()
    }
    fn big(v: &str) -> BigInt { BigInt::from_str(v).unwrap() }
    fn dec(v: &str) -> BigDecimal { BigDecimal::from_str(v).unwrap() }

    /// 2^128 + 1: past the exact-integer range of both f64 and i64.
    const BIG_STR: &str = "340282366920938463463374607431768211457";

    #[test]
    fn exact_integers_survive_past_the_float_and_i64_ranges() {
        assert_eq!(s("int.abs", &[json!(format!("-{BIG_STR}"))]), BIG_STR);
        assert_eq!(s("int.neg", &[json!(BIG_STR)]), format!("-{BIG_STR}"));
        assert_eq!(i("int.sign", &[json!(format!("-{BIG_STR}"))]), -1);
        assert_eq!(i("int.sign", &[json!("0")]), 0);
        // 2^53 and 2^53+1 are the same f64; as exact integers they are not.
        assert_eq!(i("int.cmp", &[json!("9007199254740993"), json!("9007199254740992")]), 1);
    }

    /// Both division conventions must satisfy q*divisor + r == dividend. They
    /// differ only in which way the quotient rounds, and that difference is the
    /// reason both are offered rather than one being picked for the caller.
    #[test]
    fn both_division_conventions_reconstruct_the_dividend() {
        for (a, b) in [("-7", "2"), ("7", "-2"), ("-7", "-2"), ("7", "2"),
                       (BIG_STR, "97"), (BIG_STR, "-97")] {
            let args = [json!(a), json!(b)];
            let (av, bv) = (big(a), big(b));

            let t = call("int.divmod", &args).unwrap();
            let tq = big(t[0].as_str().unwrap());
            let tr = big(t[1].as_str().unwrap());
            assert_eq!(&tq * &bv + &tr, av, "truncated divmod {a}/{b}");
            // Truncated: the remainder carries the sign of the dividend.
            assert!(tr.is_zero() || tr.is_negative() == av.is_negative(), "{a}/{b}");

            let f = call("int.divmod_floor", &args).unwrap();
            let fq = big(f[0].as_str().unwrap());
            let fr = big(f[1].as_str().unwrap());
            assert_eq!(&fq * &bv + &fr, av, "floor divmod {a}/{b}");
            // Floor: the remainder carries the sign of the divisor.
            assert!(fr.is_zero() || fr.is_negative() == bv.is_negative(), "{a}/{b}");

            assert_eq!(s("int.div_floor", &args), fq.to_string());
            assert_eq!(s("int.mod_floor", &args), fr.to_string());
        }
        // The documented disagreement between the two, pinned so it cannot drift.
        assert_eq!(s("int.div_floor", &[json!("-7"), json!("2")]), "-4");
        assert_eq!(s("int.div", &[json!("-7"), json!("2")]), "-3");
    }

    #[test]
    fn integer_sqrt_is_the_exact_floor() {
        for n in ["0", "1", "2", "99", "100", "101", BIG_STR,
                  "1000000000000000000000000000000"] {
            let root = big(&s("int.sqrt", &[json!(n)]));
            let value = big(n);
            assert!(&root * &root <= value, "sqrt({n}) came out too large");
            let next = &root + 1;
            assert!(&next * &next > value, "sqrt({n}) came out too small");
        }
        assert_eq!(err("int.sqrt", &[json!("-1")]), "DOMAIN");
    }

    #[test]
    fn shifts_round_trip_and_match_multiplication() {
        assert_eq!(s("int.shl", &[json!("1"), json!(128)]),
                   "340282366920938463463374607431768211456");
        for v in ["1", "12345", BIG_STR] {
            let up = s("int.shl", &[json!(v), json!(40)]);
            assert_eq!(s("int.shr", &[json!(up.clone()), json!(40)]), v);
            // A left shift is multiplication by a power of two.
            assert_eq!(big(&up), big(v) * big("1099511627776"));
        }
        assert_eq!(err("int.shl", &[json!("1"), json!(99_999_999)]), "LIMIT");
    }

    #[test]
    fn modular_arithmetic_holds_above_the_u64_ceiling() {
        // alg.mod_pow takes u64 arguments and cannot express this case at all.
        let m = "340282366920938463463374607431768211507";
        let p = s("int.mod_pow", &[json!(BIG_STR), json!("65537"), json!(m)]);
        assert!(big(&p) < big(m) && !big(&p).is_negative());
        // Small case checked against repeated multiplication.
        let mut expect = BigInt::from(1);
        for _ in 0..20 { expect = expect * 7 % 13; }
        assert_eq!(s("int.mod_pow", &[json!("7"), json!("20"), json!("13")]), expect.to_string());
        // An inverse must actually invert.
        for (a, modulus) in [("3", "1000000007"), (BIG_STR, "1000000007"), ("5", "8")] {
            let inv = big(&s("int.mod_inverse", &[json!(a), json!(modulus)]));
            assert_eq!((big(a) * inv).mod_floor(&big(modulus)), BigInt::from(1),
                       "inverse of {a} mod {modulus}");
        }
        // 4 and 8 share a factor, so no inverse exists.
        assert_eq!(err("int.mod_inverse", &[json!("4"), json!("8")]), "DOMAIN");
        assert_eq!(err("int.mod_pow", &[json!("2"), json!("-1"), json!("7")]), "DOMAIN");
    }

    #[test]
    fn integer_aggregates_are_exact() {
        assert_eq!(s("int.sum", &[json!(["9007199254740992", "1", "9007199254740992"])]),
                   "18014398509481985");
        assert_eq!(s("int.product", &[json!(["99999999999", "99999999999", "99999999999"])]),
                   big("99999999999").pow(3).to_string());
        let mixed = json!([BIG_STR, "-5", "0"]);
        assert_eq!(s("int.min", std::slice::from_ref(&mixed)), "-5");
        assert_eq!(s("int.max", &[mixed]), BIG_STR);
        assert_eq!(err("int.sum", &[json!([])]), "EMPTY");
    }

    /// The reason this family exists: 0.1 + 0.2 is 0.3, not 0.30000000000000004.
    #[test]
    fn decimal_addition_does_not_drift() {
        assert_eq!(s("dec.sum", &[json!(["0.1", "0.2"])]), "0.3");
        let ten_tenths: Vec<Value> = (0..10).map(|_| json!("0.1")).collect();
        assert_eq!(s("dec.sum", &[json!(ten_tenths)]), "1");
        // The float answer, for contrast, is not 1.0.
        let float_sum: f64 = (0..10).map(|_| 0.1_f64).sum();
        assert_ne!(float_sum, 1.0);
    }

    #[test]
    fn rounding_modes_differ_only_on_ties() {
        // 2.675 is exact as a decimal, so the tie is real and rounds away.
        assert_eq!(s("dec.round", &[json!("2.675"), json!(2)]), "2.68");
        assert_eq!(s("dec.round", &[json!("2.5"), json!(0)]), "3");
        assert_eq!(s("dec.round", &[json!("-2.5"), json!(0)]), "-3");
        assert_eq!(s("dec.round_even", &[json!("2.5"), json!(0)]), "2");
        assert_eq!(s("dec.round_even", &[json!("3.5"), json!(0)]), "4");
        // Away from a tie the two modes must agree.
        for v in ["2.4", "2.6", "-2.4", "-2.6", "0", "12345.6789"] {
            assert_eq!(s("dec.round", &[json!(v), json!(0)]),
                       s("dec.round_even", &[json!(v), json!(0)]), "{v}");
        }
        // Negative digits round to the left of the point.
        assert_eq!(s("dec.round", &[json!("1234.5"), json!(-2)]), "1200");
        assert_eq!(err("dec.round", &[json!("1"), json!(999_999)]), "LIMIT");
    }

    #[test]
    fn floor_and_ceiling_bracket_the_value() {
        for v in ["-2.1", "2.1", "-2.9", "0", "5"] {
            let lo = dec(&s("dec.floor", &[json!(v)]));
            let hi = dec(&s("dec.ceil", &[json!(v)]));
            let x = dec(v);
            assert!(lo <= x && x <= hi, "{v}: {lo} <= {x} <= {hi} failed");
            assert!(&hi - &lo <= dec("1"), "{v}: bracket wider than one");
            // Truncation moves toward zero, so it stays inside the bracket.
            let t = dec(&s("dec.trunc", &[json!(v)]));
            assert!(lo <= t && t <= hi, "{v}");
        }
        assert_eq!(s("dec.trunc", &[json!("-2.9")]), "-2");
        assert_eq!(s("dec.floor", &[json!("-2.9")]), "-3");
    }

    #[test]
    fn decimal_power_equals_repeated_multiplication() {
        for (base, exp) in [("1.05", 20u32), ("2", 10), ("0.1", 5), ("-1.5", 3)] {
            let mut expect = dec("1");
            for _ in 0..exp { expect *= dec(base); }
            assert_eq!(dec(&s("dec.pow", &[json!(base), json!(exp)])), expect.normalized(),
                       "{base} to the {exp}");
        }
        assert_eq!(s("dec.pow", &[json!("2"), json!(0)]), "1");
        // A negative exponent has no exact decimal answer, so it is refused.
        assert_eq!(err("dec.pow", &[json!("2"), json!(-1)]), "TYPE");
    }

    #[test]
    fn decimal_comparison_ignores_trailing_zeros_but_scale_reports_them() {
        assert_eq!(i("dec.cmp", &[json!("0.1"), json!("0.10")]), 0);
        assert_eq!(i("dec.cmp", &[json!("0.2"), json!("0.1")]), 1);
        assert_eq!(i("dec.scale", &[json!("1.2300")]), 4);
        assert_eq!(i("dec.scale", &[json!("5")]), 0);
        assert_eq!(s("dec.min", &[json!(["0.1", "0.02", "0.3"])]), "0.02");
        assert_eq!(s("dec.max", &[json!(["0.1", "0.02", "0.3"])]), "0.3");
    }

    /// Exact results must never come back in exponent notation: a caller
    /// re-reading "1E+2" as an integer string would be surprised.
    #[test]
    fn exact_results_are_written_in_plain_notation() {
        for (op, args) in [
            ("dec.sum", json!([["100", "200"]])),
            ("dec.mul", json!(["10", "10"])),
            ("dec.pow", json!(["10", 12])),
            ("dec.round", json!(["1234.5", -2])),
            ("int.shl", json!(["1", 200])),
            ("int.product", json!([["1000000", "1000000", "1000000"]])),
        ] {
            let out = s(op, args.as_array().unwrap());
            assert!(!out.contains('e') && !out.contains('E'), "{op} produced {out}");
        }
    }

    #[test]
    fn exact_guards_reject_impossible_input() {
        assert_eq!(err("int.divmod", &[json!("1"), json!("0")]), "DIV0");
        assert_eq!(err("int.div_floor", &[json!("1"), json!("0")]), "DIV0");
        assert_eq!(err("dec.mod", &[json!("1"), json!("0")]), "DIV0");
        assert_eq!(err("int.min", &[json!("not a list")]), "TYPE");
        assert_eq!(err("dec.sum", &[json!([])]), "EMPTY");
        assert_eq!(err("int.abs", &[json!("1"), json!("2")]), "ARG");
        assert_eq!(err("dec.to_number", &[json!("not a number")]), "TYPE");
        // A decimal no float can hold is a conversion failure, not bad input.
        assert_eq!(err("dec.to_number", &[json!("1e400")]), "NONFINITE");
    }
}
