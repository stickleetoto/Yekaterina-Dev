use std::collections::BTreeMap;
use serde_json::{Value, json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op, "stat.product" | "stat.range" | "stat.percentile" | "stat.q1" | "stat.q3" | "stat.iqr" | "stat.mode" | "stat.geomean" | "stat.hmean" | "stat.rms" | "stat.mad" | "stat.sample_variance" | "stat.sample_std" | "stat.covariance" | "stat.correlation" | "stat.zscore" | "stat.minmax" | "stat.cumsum" | "stat.weighted_mean" | "stat.skewness" | "stat.kurtosis" | "stat.stderr" | "stat.cv") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "stat.product" => finite(array(args)?.into_iter().product()),
        "stat.range" => { let xs=array_nonempty(args)?; finite(max(&xs)-min(&xs)) },
        "stat.percentile" => { if args.len()!=2{return Err("ARG");} let mut xs=vec_from(&args[0])?; if xs.is_empty(){Err("EMPTY")} else { let p=num(&args[1])?; percentile(&mut xs,p) } },
        "stat.q1" => { let mut xs=array_nonempty(args)?; percentile(&mut xs,25.0) },
        "stat.q3" => { let mut xs=array_nonempty(args)?; percentile(&mut xs,75.0) },
        "stat.iqr" => { let mut xs=array_nonempty(args)?; let mut ys=xs.clone(); let q1=value_num(percentile(&mut xs,25.0)?)?; let q3=value_num(percentile(&mut ys,75.0)?)?; finite(q3-q1) },
        "stat.mode" => mode(args),
        "stat.geomean" => { let xs=array_nonempty(args)?; if xs.iter().any(|x| *x<=0.0){Err("DOMAIN")}else{finite((xs.iter().map(|x|x.ln()).sum::<f64>()/xs.len() as f64).exp())} },
        "stat.hmean" => { let xs=array_nonempty(args)?; if xs.iter().any(|x| *x==0.0){Err("DIV0")}else{finite(xs.len() as f64 / xs.iter().map(|x|1.0/x).sum::<f64>())} },
        "stat.rms" => { let xs=array_nonempty(args)?; finite((xs.iter().map(|x|x*x).sum::<f64>()/xs.len() as f64).sqrt()) },
        "stat.mad" => { let xs=array_nonempty(args)?; let m=xs.iter().sum::<f64>()/xs.len() as f64; finite(xs.iter().map(|x|(x-m).abs()).sum::<f64>()/xs.len() as f64) },
        "stat.sample_variance" => { let xs=array_nonempty(args)?; sample_variance(&xs).and_then(finite) },
        "stat.sample_std" => { let xs=array_nonempty(args)?; sample_variance(&xs).and_then(|v|finite(v.sqrt())) },
        "stat.covariance" => covariance(args,false),
        "stat.correlation" => correlation(args),
        "stat.zscore" => zscore(args),
        "stat.minmax" => minmax_norm(args),
        "stat.cumsum" => cumsum(args),
        "stat.weighted_mean" => weighted_mean(args),
        "stat.skewness" => skewness(args),
        "stat.kurtosis" => kurtosis(args),
        "stat.stderr" => stderr(args),
        "stat.cv" => cv(args),
        _ => Err("OP"),
    }
}

fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn value_num(v:Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn vec_from(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?; if a.len()>100_000{return Err("LIMIT");} a.iter().map(num).collect()}
fn array(args:&[Value])->Result<Vec<f64>,&'static str>{if args.len()!=1{return Err("ARG");}vec_from(&args[0])}
fn array_nonempty(args:&[Value])->Result<Vec<f64>,&'static str>{let xs=array(args)?;if xs.is_empty(){Err("EMPTY")}else{Ok(xs)}}
fn finite(x:f64)->Result<Value,&'static str>{if !x.is_finite(){Err("NONFINITE")}else{Ok(json!(x))}}
fn min(xs:&[f64])->f64{xs.iter().copied().reduce(f64::min).unwrap()}
fn max(xs:&[f64])->f64{xs.iter().copied().reduce(f64::max).unwrap()}
fn percentile(xs:&mut [f64],p:f64)->Result<Value,&'static str>{if !(0.0..=100.0).contains(&p){return Err("DOMAIN");} xs.sort_by(f64::total_cmp); if xs.len()==1{return finite(xs[0]);} let pos=(p/100.0)*(xs.len()-1) as f64; let lo=pos.floor() as usize; let hi=pos.ceil() as usize; let t=pos-lo as f64; finite(xs[lo]+(xs[hi]-xs[lo])*t)}
fn mode(args:&[Value])->Result<Value,&'static str>{let xs=array_nonempty(args)?; let mut counts:BTreeMap<u64,(usize,f64)>=BTreeMap::new(); for x in xs{let k=x.to_bits(); let e=counts.entry(k).or_insert((0,x));e.0+=1;} let best=counts.values().max_by(|a,b|a.0.cmp(&b.0).then_with(|| b.1.total_cmp(&a.1))).unwrap(); finite(best.1)}
fn sample_variance(xs:&[f64])->Result<f64,&'static str>{if xs.len()<2{return Err("EMPTY");}let m=xs.iter().sum::<f64>()/xs.len() as f64; let v=xs.iter().map(|x|{let d=*x-m;d*d}).sum::<f64>()/(xs.len()-1) as f64;if v.is_finite(){Ok(v)}else{Err("NONFINITE")}}
fn pair(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{if args.len()!=2{return Err("ARG");}let x=vec_from(&args[0])?;let y=vec_from(&args[1])?;if x.is_empty()||x.len()!=y.len(){return Err("SHAPE");}Ok((x,y))}
fn covariance(args:&[Value],sample:bool)->Result<Value,&'static str>{let (x,y)=pair(args)?; if sample&&x.len()<2{return Err("EMPTY");}let mx=x.iter().sum::<f64>()/x.len() as f64;let my=y.iter().sum::<f64>()/y.len() as f64;let denom=if sample{(x.len()-1) as f64}else{x.len() as f64}; finite(x.iter().zip(y.iter()).map(|(a,b)|(a-mx)*(b-my)).sum::<f64>()/denom)}
fn correlation(args:&[Value])->Result<Value,&'static str>{let (x,y)=pair(args)?;let mx=x.iter().sum::<f64>()/x.len() as f64;let my=y.iter().sum::<f64>()/y.len() as f64;let mut nume=0.0;let mut dx=0.0;let mut dy=0.0;for (a,b) in x.iter().zip(y.iter()){let ax=*a-mx;let by=*b-my;nume+=ax*by;dx+=ax*ax;dy+=by*by;}if dx==0.0||dy==0.0{return Err("DOMAIN");}finite(nume/(dx*dy).sqrt())}
fn zscore(args:&[Value])->Result<Value,&'static str>{let xs=array_nonempty(args)?;let m=xs.iter().sum::<f64>()/xs.len() as f64;let var=xs.iter().map(|x|{let d=*x-m;d*d}).sum::<f64>()/xs.len() as f64;let sd=var.sqrt();if sd==0.0{return Err("DOMAIN");}Ok(json!(xs.into_iter().map(|x|(x-m)/sd).collect::<Vec<_>>()))}
fn minmax_norm(args:&[Value])->Result<Value,&'static str>{let xs=array_nonempty(args)?;let lo=min(&xs);let hi=max(&xs);if hi==lo{return Err("DOMAIN");}Ok(json!(xs.into_iter().map(|x|(x-lo)/(hi-lo)).collect::<Vec<_>>()))}
fn cumsum(args:&[Value])->Result<Value,&'static str>{let xs=array(args)?;let mut s=0.0;let mut out=Vec::with_capacity(xs.len());for x in xs{s+=x;if !s.is_finite(){return Err("NONFINITE");}out.push(s);}Ok(json!(out))}

fn weighted_mean(args:&[Value])->Result<Value,&'static str>{let(x,w)=pair(args)?;let sw=w.iter().sum::<f64>();if sw==0.0{return Err("DIV0");}finite(x.iter().zip(w.iter()).map(|(a,b)|a*b).sum::<f64>()/sw)}
fn moments(args:&[Value])->Result<(Vec<f64>,f64,f64),&'static str>{let xs=array_nonempty(args)?;let m=xs.iter().sum::<f64>()/xs.len() as f64;let var=xs.iter().map(|x|{let d=*x-m;d*d}).sum::<f64>()/xs.len() as f64;let sd=var.sqrt();if sd==0.0{return Err("DOMAIN");}Ok((xs,m,sd))}
fn skewness(args:&[Value])->Result<Value,&'static str>{let(xs,m,sd)=moments(args)?;finite(xs.iter().map(|x|((*x-m)/sd).powi(3)).sum::<f64>()/xs.len() as f64)}
fn kurtosis(args:&[Value])->Result<Value,&'static str>{let(xs,m,sd)=moments(args)?;finite(xs.iter().map(|x|((*x-m)/sd).powi(4)).sum::<f64>()/xs.len() as f64-3.0)}
fn stderr(args:&[Value])->Result<Value,&'static str>{let xs=array_nonempty(args)?;let v=sample_variance(&xs)?;finite(v.sqrt()/(xs.len() as f64).sqrt())}
fn cv(args:&[Value])->Result<Value,&'static str>{let xs=array_nonempty(args)?;let m=xs.iter().sum::<f64>()/xs.len() as f64;if m==0.0{return Err("DIV0");}let var=xs.iter().map(|x|{let d=*x-m;d*d}).sum::<f64>()/xs.len() as f64;finite(var.sqrt()/m.abs())}
