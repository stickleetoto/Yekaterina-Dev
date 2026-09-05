use num_bigint::BigInt;
use serde_json::{Value, json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "bit.and" | "bit.or" | "bit.xor" | "bit.not" | "bit.shl" | "bit.shr" |
        "bit.popcount" | "bit.test" | "bit.set" | "bit.clear" | "bit.toggle" |
        "base.convert" | "base.to10" | "base.from10" | "base.is_valid"
    ) { Some(run(op, args)) } else { None }
}

fn run(op:&str,args:&[Value])->Result<Value,&'static str>{
    match op {
        "bit.and"=>binary_u64(args,|a,b|a&b),
        "bit.or"=>binary_u64(args,|a,b|a|b),
        "bit.xor"=>binary_u64(args,|a,b|a^b),
        "bit.not"=>{if args.len()!=1{return Err("ARG");}Ok(json!(!u64v(&args[0])?))},
        "bit.shl"=>shift(args,true),
        "bit.shr"=>shift(args,false),
        "bit.popcount"=>{if args.len()!=1{return Err("ARG");}Ok(json!(u64v(&args[0])?.count_ones()))},
        "bit.test"=>bit_mut(args,"test"),
        "bit.set"=>bit_mut(args,"set"),
        "bit.clear"=>bit_mut(args,"clear"),
        "bit.toggle"=>bit_mut(args,"toggle"),
        "base.convert"=>base_convert(args),
        "base.to10"=>base_to10(args),
        "base.from10"=>base_from10(args),
        "base.is_valid"=>base_valid(args),
        _=>Err("OP"),
    }
}
fn u64v(v:&Value)->Result<u64,&'static str>{v.as_u64().ok_or("TYPE")}
fn binary_u64<F:FnOnce(u64,u64)->u64>(args:&[Value],f:F)->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}Ok(json!(f(u64v(&args[0])?,u64v(&args[1])?)))}
fn shift(args:&[Value],left:bool)->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=u64v(&args[0])?;let n=u64v(&args[1])?;if n>=64{return Err("DOMAIN");}Ok(json!(if left{x<<(n as u32)}else{x>>(n as u32)}))}
fn bit_mut(args:&[Value],mode:&str)->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=u64v(&args[0])?;let n=u64v(&args[1])?;if n>=64{return Err("DOMAIN");}let mask=1u64<<(n as u32);Ok(match mode{"test"=>json!((x&mask)!=0),"set"=>json!(x|mask),"clear"=>json!(x&!mask),"toggle"=>json!(x^mask),_=>return Err("OP")})}
fn radix(v:&Value)->Result<u32,&'static str>{let r=v.as_u64().ok_or("TYPE")? as u32;if !(2..=36).contains(&r){Err("DOMAIN")}else{Ok(r)}}
fn parse_big(s:&str,r:u32)->Result<BigInt,&'static str>{let t=s.trim();if t.len()>100_000{return Err("LIMIT");}let (neg,digits)=if let Some(rest)=t.strip_prefix('-'){(true,rest)}else{(false,t)};if digits.is_empty(){return Err("TYPE");}let mut x=BigInt::parse_bytes(digits.as_bytes(),r).ok_or("TYPE")?;if neg{x=-x;}Ok(x)}
fn base_convert(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let s=args[0].as_str().ok_or("TYPE")?;let from=radix(&args[1])?;let to=radix(&args[2])?;Ok(json!(parse_big(s,from)?.to_str_radix(to)))}
fn base_to10(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let s=args[0].as_str().ok_or("TYPE")?;let from=radix(&args[1])?;Ok(json!(parse_big(s,from)?.to_str_radix(10)))}
fn base_from10(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let to=radix(&args[1])?;let x=match &args[0]{Value::String(s)=>parse_big(s,10)?,v if v.as_i64().is_some()=>BigInt::from(v.as_i64().unwrap()),v if v.as_u64().is_some()=>BigInt::from(v.as_u64().unwrap()),_=>return Err("TYPE")};Ok(json!(x.to_str_radix(to)))}
fn base_valid(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let s=args[0].as_str().ok_or("TYPE")?;let r=radix(&args[1])?;Ok(json!(parse_big(s,r).is_ok()))}

