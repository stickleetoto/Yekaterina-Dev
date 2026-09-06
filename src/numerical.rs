use std::collections::HashMap;
use serde_json::{Value, json};
use crate::formula;

const MAX_STEPS: usize = 100_000;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    if matches!(op,
        "num.lerp" | "num.inv_lerp" | "num.map_range" | "num.trapezoid" |
        "num.simpson_uniform" | "num.derivative3" | "num.derivative5" |
        "num.bisect" | "num.integrate" | "num.derivative_expr" | "num.newton"
    ){Some(run(op,args))}else{None}
}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "num.lerp"=>{if args.len()!=3{return Err("ARG");}let a=num(&args[0])?;let b=num(&args[1])?;let t=num(&args[2])?;finite(a+(b-a)*t)},
    "num.inv_lerp"=>{if args.len()!=3{return Err("ARG");}let a=num(&args[0])?;let b=num(&args[1])?;let x=num(&args[2])?;if a==b{Err("DIV0")}else{finite((x-a)/(b-a))}},
    "num.map_range"=>map_range(args),
    "num.trapezoid"=>trapezoid(args),
    "num.simpson_uniform"=>simpson(args),
    "num.derivative3"=>derivative3(args),
    "num.derivative5"=>derivative5(args),
    "num.bisect"=>bisect(args),
    "num.integrate"=>integrate_expr(args),
    "num.derivative_expr"=>derivative_expr(args),
    "num.newton"=>newton(args),
    _=>Err("OP")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn vec_from(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>MAX_STEPS{return Err("LIMIT");}a.iter().map(num).collect()}
fn map_range(args:&[Value])->Result<Value,&'static str>{if args.len()!=5{return Err("ARG");}let x=num(&args[0])?;let a=num(&args[1])?;let b=num(&args[2])?;let c=num(&args[3])?;let d=num(&args[4])?;if a==b{return Err("DIV0");}finite(c+(x-a)*(d-c)/(b-a))}
fn trapezoid(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec_from(&args[0])?;let y=vec_from(&args[1])?;if x.len()!=y.len()||x.len()<2{return Err("SHAPE");}let mut s=0.0;for i in 1..x.len(){let dx=x[i]-x[i-1];s+=dx*(y[i]+y[i-1])*0.5;}finite(s)}
fn simpson(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let y=vec_from(&args[0])?;let dx=num(&args[1])?;if y.len()<3||y.len()%2==0||dx==0.0{return Err("SHAPE");}let mut s=y[0]+y[y.len()-1];for(i,v)in y.iter().enumerate().take(y.len()-1).skip(1){s+=if i%2==0{2.0*v}else{4.0*v};}finite(s*dx/3.0)}
fn derivative3(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let fm=num(&args[0])?;let fp=num(&args[1])?;let h=num(&args[2])?;if h==0.0{Err("DIV0")}else{finite((fp-fm)/(2.0*h))}}
fn derivative5(args:&[Value])->Result<Value,&'static str>{if args.len()!=5{return Err("ARG");}let fm2=num(&args[0])?;let fm1=num(&args[1])?;let fp1=num(&args[2])?;let fp2=num(&args[3])?;let h=num(&args[4])?;if h==0.0{Err("DIV0")}else{finite((fm2-8.0*fm1+8.0*fp1-fp2)/(12.0*h))}}
fn expr_obj(v:&Value)->Result<(&str,HashMap<String,f64>),&'static str>{let o=v.as_object().ok_or("TYPE")?;let e=o.get("e").and_then(Value::as_str).ok_or("ARG")?;let mut vars=HashMap::new();if let Some(map)=o.get("v"){let map=map.as_object().ok_or("TYPE")?;if map.len()>formula::MAX_PARAMS{return Err("LIMIT");}for(k,v)in map{vars.insert(k.clone(),num(v)?);}}Ok((e,vars))}
// Phase 2C-2: the environment is built once per operation and the loop
// variable overwritten in place, instead of cloning the map and allocating the
// key on every evaluation.
fn env_of(base:&HashMap<String,f64>)->formula::Env{formula::Env::new(base,&["x"])}
fn eval_x(env:&formula::Env,e:&str,x:f64)->Result<f64,&'static str>{env.set("x",x);env.eval(e)}
fn bisect(args:&[Value])->Result<Value,&'static str>{if args.len()<3||args.len()>5{return Err("ARG");}let(e,vars)=expr_obj(&args[0])?;let mut lo=num(&args[1])?;let mut hi=num(&args[2])?;let tol=if args.len()>=4{num(&args[3])?}else{1e-10};let max=if args.len()==5{args[4].as_u64().ok_or("TYPE")?.min(10_000) as usize}else{128};if !(tol>0.0)||lo>=hi{return Err("DOMAIN");}let env=env_of(&vars);let mut flo=eval_x(&env,e,lo)?;let fhi=eval_x(&env,e,hi)?;if flo==0.0{return finite(lo);}if fhi==0.0{return finite(hi);}if flo.signum()==fhi.signum(){return Err("BRACKET");}for _ in 0..max{let mid=(lo+hi)*0.5;let fm=eval_x(&env,e,mid)?;if fm.abs()<=tol||(hi-lo).abs()<=tol{return finite(mid);}if flo.signum()!=fm.signum(){hi=mid;}else{lo=mid;flo=fm;}}finite((lo+hi)*0.5)}
fn integrate_expr(args:&[Value])->Result<Value,&'static str>{if args.len()<3||args.len()>4{return Err("ARG");}let(e,vars)=expr_obj(&args[0])?;let a=num(&args[1])?;let b=num(&args[2])?;let n=if args.len()==4{args[3].as_u64().ok_or("TYPE")? as usize}else{1024};if n==0||n>MAX_STEPS{return Err("LIMIT");}let h=(b-a)/n as f64;let env=env_of(&vars);let fa=eval_x(&env,e,a)?;let fb=eval_x(&env,e,b)?;let mut s=0.5*(fa+fb);for i in 1..n{s+=eval_x(&env,e,a+h*i as f64)?;}finite(s*h)}
fn derivative_expr(args:&[Value])->Result<Value,&'static str>{if args.len()<2||args.len()>3{return Err("ARG");}let(e,vars)=expr_obj(&args[0])?;let x=num(&args[1])?;let h=if args.len()==3{num(&args[2])?}else{1e-5_f64*(1.0+x.abs())};if h==0.0{return Err("DIV0");}let env=env_of(&vars);let fp=eval_x(&env,e,x+h)?;let fm=eval_x(&env,e,x-h)?;finite((fp-fm)/(2.0*h))}
fn newton(args:&[Value])->Result<Value,&'static str>{if args.len()<2||args.len()>4{return Err("ARG");}let(e,vars)=expr_obj(&args[0])?;let mut x=num(&args[1])?;let tol=if args.len()>=3{num(&args[2])?}else{1e-10};let max=if args.len()==4{args[3].as_u64().ok_or("TYPE")?.min(10_000) as usize}else{64};if tol<=0.0{return Err("DOMAIN");}let env=env_of(&vars);for _ in 0..max{let fx=eval_x(&env,e,x)?;if fx.abs()<=tol{return finite(x);}let h=1e-6_f64*(1.0+x.abs());let dp=eval_x(&env,e,x+h)?;let dm=eval_x(&env,e,x-h)?;let d=(dp-dm)/(2.0*h);if d.abs()<1e-15{return Err("DERIV");}let nx=x-fx/d;if !nx.is_finite(){return Err("NONFINITE");}if (nx-x).abs()<=tol{return finite(nx);}x=nx;}finite(x)}
