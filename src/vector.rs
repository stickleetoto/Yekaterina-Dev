use serde_json::{Value,json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op, "vec.add" | "vec.sub" | "vec.scale" | "vec.dot" | "vec.norm" | "vec.distance" | "vec.cosine" | "vec.cross3" | "vec.sum") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "vec.add"=>binary_vec(args,|a,b|a+b),
        "vec.sub"=>binary_vec(args,|a,b|a-b),
        "vec.scale"=>scale(args),
        "vec.dot"=>dot(args).and_then(finite),
        "vec.norm"=>{let x=one_vec(args)?;finite(x.iter().map(|v|v*v).sum::<f64>().sqrt())},
        "vec.distance"=>{let (a,b)=two_vec(args)?;finite(a.iter().zip(b.iter()).map(|(x,y)|(x-y)*(x-y)).sum::<f64>().sqrt())},
        "vec.cosine"=>cosine(args),
        "vec.cross3"=>cross3(args),
        "vec.sum"=>{let x=one_vec(args)?;finite(x.iter().sum())},
        _ => Err("OP"),
    }
}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn vec(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>100_000{return Err("LIMIT");}a.iter().map(num).collect()}
fn one_vec(args:&[Value])->Result<Vec<f64>,&'static str>{if args.len()!=1{return Err("ARG");}vec(&args[0])}
fn two_vec(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{if args.len()!=2{return Err("ARG");}let a=vec(&args[0])?;let b=vec(&args[1])?;if a.len()!=b.len(){Err("SHAPE")}else{Ok((a,b))}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn binary_vec<F:Fn(f64,f64)->f64>(args:&[Value],f:F)->Result<Value,&'static str>{let(a,b)=two_vec(args)?;let out=a.into_iter().zip(b).map(|(x,y)|f(x,y)).collect::<Vec<_>>();if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}}
fn scale(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=vec(&args[0])?;let s=num(&args[1])?;let out=a.into_iter().map(|x|x*s).collect::<Vec<_>>();if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}}
fn dot(args:&[Value])->Result<f64,&'static str>{let(a,b)=two_vec(args)?;let x=a.iter().zip(b.iter()).map(|(x,y)|x*y).sum::<f64>();if x.is_finite(){Ok(x)}else{Err("NONFINITE")}}
fn cosine(args:&[Value])->Result<Value,&'static str>{let(a,b)=two_vec(args)?;let d=a.iter().zip(b.iter()).map(|(x,y)|x*y).sum::<f64>();let na=a.iter().map(|x|x*x).sum::<f64>().sqrt();let nb=b.iter().map(|x|x*x).sum::<f64>().sqrt();if na==0.0||nb==0.0{Err("DOMAIN")}else{finite(d/(na*nb))}}
fn cross3(args:&[Value])->Result<Value,&'static str>{let(a,b)=two_vec(args)?;if a.len()!=3{return Err("SHAPE");}Ok(json!([a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]))}
