use std::f64::consts::PI;
use serde_json::{Value,json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op, "geo.distance2d" | "geo.distance3d" | "geo.midpoint2d" | "geo.circle_area" | "geo.circle_circumference" | "geo.rectangle_area" | "geo.rectangle_perimeter" | "geo.triangle_area") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "geo.distance2d"=>{let p=points(args,2)?;finite(dist(&p.0,&p.1))},
        "geo.distance3d"=>{let p=points(args,3)?;finite(dist(&p.0,&p.1))},
        "geo.midpoint2d"=>{let(a,b)=points(args,2)?;Ok(json!([(a[0]+b[0])/2.0,(a[1]+b[1])/2.0]))},
        "geo.circle_area"=>{let r=one(args)?;if r<0.0{Err("DOMAIN")}else{finite(PI*r*r)}},
        "geo.circle_circumference"=>{let r=one(args)?;if r<0.0{Err("DOMAIN")}else{finite(2.0*PI*r)}},
        "geo.rectangle_area"=>{let(a,b)=two(args)?;if a<0.0||b<0.0{Err("DOMAIN")}else{finite(a*b)}},
        "geo.rectangle_perimeter"=>{let(a,b)=two(args)?;if a<0.0||b<0.0{Err("DOMAIN")}else{finite(2.0*(a+b))}},
        "geo.triangle_area"=>triangle_area(args),
        _ => Err("OP"),
    }
}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{if args.len()!=1{return Err("ARG");}num(&args[0])}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{if args.len()!=2{return Err("ARG");}Ok((num(&args[0])?,num(&args[1])?))}
fn point(v:&Value,n:usize)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=n{return Err("SHAPE");}a.iter().map(num).collect()}
fn points(args:&[Value],n:usize)->Result<(Vec<f64>,Vec<f64>),&'static str>{if args.len()!=2{return Err("ARG");}Ok((point(&args[0],n)?,point(&args[1],n)?))}
fn dist(a:&[f64],b:&[f64])->f64{a.iter().zip(b.iter()).map(|(x,y)|(x-y)*(x-y)).sum::<f64>().sqrt()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn triangle_area(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let a=num(&args[0])?;let b=num(&args[1])?;let c=num(&args[2])?;if a<=0.0||b<=0.0||c<=0.0||a+b<=c||a+c<=b||b+c<=a{return Err("DOMAIN");}let s=(a+b+c)/2.0;finite((s*(s-a)*(s-b)*(s-c)).sqrt())}
