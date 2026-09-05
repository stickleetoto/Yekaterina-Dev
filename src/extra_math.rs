use serde_json::{Value, json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op, "math.cbrt" | "math.trunc" | "math.fract" | "math.sign" | "math.hypot" | "math.deg2rad" | "math.rad2deg" | "math.asin" | "math.acos" | "math.atan" | "math.atan2" | "math.sinh" | "math.cosh" | "math.tanh" | "math.log2" | "math.log" | "math.exp2" | "math.recip" | "math.lerp" | "math.approx_eq") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "math.cbrt" => finite(one(args)?.cbrt()),
        "math.trunc" => finite(one(args)?.trunc()),
        "math.fract" => finite(one(args)?.fract()),
        "math.sign" => {
            let x = one(args)?;
            finite(if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 })
        }
        "math.hypot" => { let (a,b)=two(args)?; finite(a.hypot(b)) },
        "math.deg2rad" => finite(one(args)?.to_radians()),
        "math.rad2deg" => finite(one(args)?.to_degrees()),
        "math.asin" => { let x=one(args)?; if !(-1.0..=1.0).contains(&x){Err("DOMAIN")}else{finite(x.asin())} },
        "math.acos" => { let x=one(args)?; if !(-1.0..=1.0).contains(&x){Err("DOMAIN")}else{finite(x.acos())} },
        "math.atan" => finite(one(args)?.atan()),
        "math.atan2" => { let (y,x)=two(args)?; finite(y.atan2(x)) },
        "math.sinh" => finite(one(args)?.sinh()),
        "math.cosh" => finite(one(args)?.cosh()),
        "math.tanh" => finite(one(args)?.tanh()),
        "math.log2" => { let x=one(args)?; if x<=0.0{Err("DOMAIN")}else{finite(x.log2())} },
        "math.log" => { let (x,base)=two(args)?; if x<=0.0 || base<=0.0 || base==1.0 {Err("DOMAIN")} else {finite(x.log(base))} },
        "math.exp2" => finite(one(args)?.exp2()),
        "math.recip" => { let x=one(args)?; if x==0.0{Err("DIV0")}else{finite(x.recip())} },
        "math.lerp" => { if args.len()!=3{return Err("ARG");} let a=num(&args[0])?; let b=num(&args[1])?; let t=num(&args[2])?; finite(a+(b-a)*t) },
        "math.approx_eq" => { if args.len()!=3{return Err("ARG");} let a=num(&args[0])?; let b=num(&args[1])?; let eps=num(&args[2])?; if eps<0.0 {Err("DOMAIN")} else {Ok(json!((a-b).abs()<=eps))} },
        _ => Err("OP"),
    }
}

fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{if args.len()!=1{return Err("ARG");}num(&args[0])}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{if args.len()!=2{return Err("ARG");}Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if !x.is_finite(){Err("NONFINITE")}else{Ok(json!(x))}}
