use std::str::FromStr;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;
use serde_json::{Value, json};

const MAX_EXACT_INPUT_CHARS: usize = 100_000;
const MAX_EXACT_RESULT_CHARS: usize = 1_000_000;
const MAX_INT_POW_EXP: u64 = 1_000_000;

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
        "dec.add" => Some(dec_binary(args, |a, b| a + b)),
        "dec.sub" => Some(dec_binary(args, |a, b| a - b)),
        "dec.mul" => Some(dec_binary(args, |a, b| a * b)),
        "dec.div" => Some((|| {
            let (a, b) = two_decimal(args)?;
            if b.is_zero() { Err("DIV0") } else { exact_decimal(a / b) }
        })()),
        _ => None,
    }
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
