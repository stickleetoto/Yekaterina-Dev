use std::collections::HashMap;
use serde_json::{json, Value};
use crate::formula;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    if matches!(op,
        "num.secant"|"num.false_position"|"num.second_derivative_expr"|"num.integrate_midpoint"|
        "num.integrate_simpson_expr"|"num.euler_ode"|"num.rk4_ode"|"num.interpolate_linear"|
        "num.interpolate_lagrange"|"num.forward_diff"|"num.backward_diff"|"num.central_diff"|
        "num.trapezoid_uniform"|"num.cumulative_trapezoid"|"num.linspace"|"num.logspace"|
        "num.geomspace"|"num.argmin"|"num.argmax"|"num.newton_forward_step"|"num.richardson_derivative"|
        "num.root_mean_square_error"|"num.relative_error"|"num.absolute_error"|"num.percent_error"
    ){Some(run(op,args))}else{None}
}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "num.secant"=>secant(args),
    "num.false_position"=>false_position(args),
    "num.second_derivative_expr"=>second_derivative(args),
    "num.integrate_midpoint"=>integrate_midpoint(args),
    "num.integrate_simpson_expr"=>integrate_simpson(args),
    "num.euler_ode"=>ode(args,false),
    "num.rk4_ode"=>ode(args,true),
    "num.interpolate_linear"=>interp_linear(args),
    "num.interpolate_lagrange"=>interp_lagrange(args),
    "num.forward_diff"=>diff_seq(args,"forward"),
    "num.backward_diff"=>diff_seq(args,"backward"),
    "num.central_diff"=>central_diff(args),
    "num.trapezoid_uniform"=>trap_uniform(args),
    "num.cumulative_trapezoid"=>cum_trap(args),
    "num.linspace"=>linspace(args),
    "num.logspace"=>logspace(args),
    "num.geomspace"=>geomspace(args),
    "num.argmin"=>argextreme(args,false),
    "num.argmax"=>argextreme(args,true),
    "num.newton_forward_step"=>newton_forward_step(args),
    "num.richardson_derivative"=>richardson(args),
    "num.root_mean_square_error"=>rmse(args),
    "num.relative_error"=>error_metric(args,"rel"),
    "num.absolute_error"=>error_metric(args,"abs"),
    "num.percent_error"=>error_metric(args,"pct"),
    _=>Err("OP")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn vec(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>100_000{return Err("LIMIT");}a.iter().map(num).collect()}
fn expr(v:&Value)->Result<(&str,HashMap<String,f64>),&'static str>{let o=v.as_object().ok_or("TYPE")?;let e=o.get("e").and_then(Value::as_str).ok_or("ARG")?;let mut vars=HashMap::new();if let Some(m)=o.get("v"){let m=m.as_object().ok_or("TYPE")?;for(k,v)in m{vars.insert(k.clone(),num(v)?);}}Ok((e,vars))}
fn env1_of(b:&HashMap<String,f64>)->formula::Env{formula::Env::new(b,&["x"])}
fn env2_of(b:&HashMap<String,f64>)->formula::Env{formula::Env::new(b,&["x","y"])}
fn eval1(env:&formula::Env,e:&str,x:f64)->Result<f64,&'static str>{env.set("x",x);env.eval(e)}
fn eval2(env:&formula::Env,e:&str,x:f64,y:f64)->Result<f64,&'static str>{env.set("x",x);env.set("y",y);env.eval(e)}
fn secant(args:&[Value])->Result<Value,&'static str>{if args.len()<3||args.len()>5{return Err("ARG");}let(e,v)=expr(&args[0])?;let env=env1_of(&v);let mut x0=num(&args[1])?;let mut x1=num(&args[2])?;let tol=if args.len()>=4{num(&args[3])?}else{1e-10};let max=if args.len()==5{args[4].as_u64().ok_or("TYPE")?.min(100_000) as usize}else{100};if tol<=0.0{return Err("DOMAIN");}for _ in 0..max{let f0=eval1(&env,e,x0)?;let f1=eval1(&env,e,x1)?;let d=f1-f0;if d.abs()<1e-18{return Err("DOMAIN");}let x2=x1-f1*(x1-x0)/d;if (x2-x1).abs()<=tol{return finite(x2);}x0=x1;x1=x2;}Err("NO_CONVERGE")}
fn false_position(args:&[Value])->Result<Value,&'static str>{if args.len()<3||args.len()>5{return Err("ARG");}let(e,v)=expr(&args[0])?;let env=env1_of(&v);let mut a=num(&args[1])?;let mut b=num(&args[2])?;let tol=if args.len()>=4{num(&args[3])?}else{1e-10};let max=if args.len()==5{args[4].as_u64().ok_or("TYPE")?.min(100_000) as usize}else{100};let mut fa=eval1(&env,e,a)?;let mut fb=eval1(&env,e,b)?;if fa*fb>0.0{return Err("DOMAIN");}for _ in 0..max{let d=fb-fa;if d.abs()<1e-18{return Err("DOMAIN");}let c=(a*fb-b*fa)/d;let fc=eval1(&env,e,c)?;if fc.abs()<=tol{return finite(c);}if fa*fc<0.0{b=c;fb=fc;}else{a=c;fa=fc;}}Err("NO_CONVERGE")}
fn second_derivative(args:&[Value])->Result<Value,&'static str>{if args.len()<2||args.len()>3{return Err("ARG");}let(e,v)=expr(&args[0])?;let env=env1_of(&v);let x=num(&args[1])?;let h=if args.len()==3{num(&args[2])?}else{1e-4};if h<=0.0{return Err("DOMAIN");}finite((eval1(&env,e,x+h)?-2.0*eval1(&env,e,x)?+eval1(&env,e,x-h)?)/(h*h))}
fn integrate_midpoint(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let(e,v)=expr(&args[0])?;let env=env1_of(&v);let a=num(&args[1])?;let b=num(&args[2])?;let n=args[3].as_u64().ok_or("TYPE")? as usize;if n==0||n>1_000_000{return Err("LIMIT");}let h=(b-a)/n as f64;let mut s=0.0;for i in 0..n{s+=eval1(&env,e,a+(i as f64+0.5)*h)?;}finite(s*h)}
fn integrate_simpson(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let(e,v)=expr(&args[0])?;let env=env1_of(&v);let a=num(&args[1])?;let b=num(&args[2])?;let n=args[3].as_u64().ok_or("TYPE")? as usize;if n==0||n%2!=0||n>1_000_000{return Err("SHAPE");}let h=(b-a)/n as f64;let mut s=eval1(&env,e,a)?+eval1(&env,e,b)?;for i in 1..n{s+=(if i%2==0{2.0}else{4.0})*eval1(&env,e,a+i as f64*h)?;}finite(s*h/3.0)}
fn ode(args:&[Value],rk4:bool)->Result<Value,&'static str>{if args.len()!=5{return Err("ARG");}let(e,v)=expr(&args[0])?;let env=env2_of(&v);let mut x=num(&args[1])?;let mut y=num(&args[2])?;let x1=num(&args[3])?;let n=args[4].as_u64().ok_or("TYPE")? as usize;if n==0||n>1_000_000{return Err("LIMIT");}let h=(x1-x)/n as f64;for _ in 0..n{if rk4{let k1=eval2(&env,e,x,y)?;let k2=eval2(&env,e,x+h/2.0,y+h*k1/2.0)?;let k3=eval2(&env,e,x+h/2.0,y+h*k2/2.0)?;let k4=eval2(&env,e,x+h,y+h*k3)?;y+=h*(k1+2.0*k2+2.0*k3+k4)/6.0;}else{y+=h*eval2(&env,e,x,y)?;}x+=h;if !x.is_finite()||!y.is_finite(){return Err("NONFINITE");}}finite(y)}
fn interp_linear(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=vec(&args[0])?;let y=vec(&args[1])?;let q=num(&args[2])?;if x.len()!=y.len()||x.len()<2{return Err("SHAPE");}for i in 1..x.len(){if x[i]<=x[i-1]{return Err("DOMAIN");}}if q<x[0]||q>x[x.len()-1]{return Err("DOMAIN");}let mut hi=1;while hi<x.len()&&x[hi]<q{hi+=1;}if hi==x.len(){return finite(y[y.len()-1]);}let lo=hi-1;let t=(q-x[lo])/(x[hi]-x[lo]);finite(y[lo]+(y[hi]-y[lo])*t)}
fn interp_lagrange(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=vec(&args[0])?;let y=vec(&args[1])?;let q=num(&args[2])?;if x.len()!=y.len()||x.is_empty()||x.len()>100{return Err("SHAPE");}let mut s=0.0;for i in 0..x.len(){let mut t=y[i];for j in 0..x.len(){if i!=j{let d=x[i]-x[j];if d==0.0{return Err("DOMAIN");}t*= (q-x[j])/d;}}s+=t;}finite(s)}
fn diff_seq(args:&[Value],mode:&str)->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let y=vec(&args[0])?;let h=num(&args[1])?;if y.len()<2||h==0.0{return Err("SHAPE");}let o=if mode=="forward"{y.windows(2).map(|w|(w[1]-w[0])/h).collect::<Vec<_>>()}else{y.windows(2).map(|w|(w[1]-w[0])/h).collect::<Vec<_>>()};Ok(json!(o))}
fn central_diff(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let y=vec(&args[0])?;let h=num(&args[1])?;if y.len()<3||h==0.0{return Err("SHAPE");}Ok(json!((1..y.len()-1).map(|i|(y[i+1]-y[i-1])/(2.0*h)).collect::<Vec<_>>()))}
fn trap_uniform(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let y=vec(&args[0])?;let h=num(&args[1])?;if y.len()<2{return Err("SHAPE");}finite(h*(y[0]/2.0+y[y.len()-1]/2.0+y[1..y.len()-1].iter().sum::<f64>()))}
fn cum_trap(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let y=vec(&args[1])?;if x.len()!=y.len()||x.len()<2{return Err("SHAPE");}let mut s=0.0;let mut o=vec![0.0];for i in 1..x.len(){s+=(x[i]-x[i-1])*(y[i]+y[i-1])/2.0;o.push(s);}Ok(json!(o))}
fn linspace(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let a=num(&args[0])?;let b=num(&args[1])?;let n=args[2].as_u64().ok_or("TYPE")? as usize;if n==0||n>100_000{return Err("LIMIT");}if n==1{return Ok(json!([a]));}let h=(b-a)/(n-1) as f64;Ok(json!((0..n).map(|i|a+i as f64*h).collect::<Vec<_>>()))}
fn logspace(args:&[Value])->Result<Value,&'static str>{if args.len()<3||args.len()>4{return Err("ARG");}let a=num(&args[0])?;let b=num(&args[1])?;let n=args[2].as_u64().ok_or("TYPE")? as usize;let base=if args.len()==4{num(&args[3])?}else{10.0};if n==0||n>100_000||base<=0.0||base==1.0{return Err("DOMAIN");}let vals=if n==1{vec![base.powf(a)]}else{let h=(b-a)/(n-1) as f64;(0..n).map(|i|base.powf(a+i as f64*h)).collect()};Ok(json!(vals))}
fn geomspace(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let a=num(&args[0])?;let b=num(&args[1])?;let n=args[2].as_u64().ok_or("TYPE")? as usize;if a<=0.0||b<=0.0||n==0||n>100_000{return Err("DOMAIN");}if n==1{return Ok(json!([a]));}let r=(b/a).powf(1.0/(n-1) as f64);Ok(json!((0..n).map(|i|a*r.powf(i as f64)).collect::<Vec<_>>()))}
fn argextreme(args:&[Value],max:bool)->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let x=vec(&args[0])?;if x.is_empty(){return Err("EMPTY");}let mut idx=0;for i in 1..x.len(){let better=if max{x[i]>x[idx]}else{x[i]<x[idx]};if better{idx=i;}}Ok(json!(idx))}
fn newton_forward_step(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let x=num(&args[0])?;let fx=num(&args[1])?;let dfx=num(&args[2])?;let damping=num(&args[3])?;if dfx==0.0{return Err("DIV0");}finite(x-damping*fx/dfx)}
fn richardson(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let(e,v)=expr(&args[0])?;let env=env1_of(&v);let x=num(&args[1])?;let h=num(&args[2])?;if h<=0.0{return Err("DOMAIN");}let d1=(eval1(&env,e,x+h)?-eval1(&env,e,x-h)?)/(2.0*h);let h2=h/2.0;let d2=(eval1(&env,e,x+h2)?-eval1(&env,e,x-h2)?)/(2.0*h2);finite((4.0*d2-d1)/3.0)}
fn rmse(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=vec(&args[0])?;let b=vec(&args[1])?;if a.is_empty()||a.len()!=b.len(){return Err("SHAPE");}finite((a.iter().zip(b).map(|(x,y)|{let d=(*x)-y;d*d}).sum::<f64>()/a.len() as f64).sqrt())}
fn error_metric(args:&[Value],mode:&str)->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let exact=num(&args[0])?;let approx=num(&args[1])?;let a=(approx-exact).abs();match mode{"abs"=>finite(a),"rel"=>{if exact==0.0{Err("DIV0")}else{finite(a/exact.abs())}},"pct"=>{if exact==0.0{Err("DIV0")}else{finite(a/exact.abs()*100.0)}},_=>Err("OP")}}
