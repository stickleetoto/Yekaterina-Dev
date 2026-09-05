use serde_json::{Value,json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "signal.diff" | "signal.moving_avg" | "signal.ema" | "signal.convolve" |
        "signal.correlate" | "signal.energy" | "signal.power" | "signal.rms" |
        "signal.normalize_peak" | "signal.zero_crossings" | "signal.cumulative" |
        "signal.decimate"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "signal.diff"=>diff(args),
        "signal.moving_avg"=>moving_avg(args),
        "signal.ema"=>ema(args),
        "signal.convolve"=>convolve(args),
        "signal.correlate"=>correlate(args),
        "signal.energy"=>{let x=one(args)?;finite(x.iter().map(|v|v*v).sum())},
        "signal.power"=>{let x=one_nonempty(args)?;finite(x.iter().map(|v|v*v).sum::<f64>()/x.len() as f64)},
        "signal.rms"=>{let x=one_nonempty(args)?;finite((x.iter().map(|v|v*v).sum::<f64>()/x.len() as f64).sqrt())},
        "signal.normalize_peak"=>normalize_peak(args),
        "signal.zero_crossings"=>zero_crossings(args),
        "signal.cumulative"=>cumulative(args),
        "signal.decimate"=>decimate(args),
        _ => Err("OP"),
    }
}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn vec(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>100_000{return Err("LIMIT");}a.iter().map(num).collect()}
fn one(args:&[Value])->Result<Vec<f64>,&'static str>{if args.len()!=1{return Err("ARG");}vec(&args[0])}
fn one_nonempty(args:&[Value])->Result<Vec<f64>,&'static str>{let x=one(args)?;if x.is_empty(){Err("EMPTY")}else{Ok(x)}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn diff(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;Ok(json!(x.windows(2).map(|w|w[1]-w[0]).collect::<Vec<_>>()))}
fn moving_avg(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let w=args[1].as_u64().ok_or("TYPE")? as usize;if w==0||w>x.len(){return Err("DOMAIN");}let mut s=x[..w].iter().sum::<f64>();let mut out=vec![s/w as f64];for i in w..x.len(){s+=x[i]-x[i-w];out.push(s/w as f64);}Ok(json!(out))}
fn ema(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;if x.is_empty(){return Err("EMPTY");}let a=num(&args[1])?;if !(0.0<a&&a<=1.0){return Err("DOMAIN");}let mut prev=x[0];let mut out=Vec::with_capacity(x.len());out.push(prev);for v in x.into_iter().skip(1){prev=a*v+(1.0-a)*prev;out.push(prev);}Ok(json!(out))}
fn convolve(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let h=vec(&args[1])?;if x.is_empty()||h.is_empty(){return Err("EMPTY");}if x.len().saturating_mul(h.len())>2_000_000{return Err("LIMIT");}let mut out=vec![0.0;x.len()+h.len()-1];for(i,a)in x.iter().enumerate(){for(j,b)in h.iter().enumerate(){out[i+j]+=a*b;}}if out.iter().any(|v|!v.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}}
fn correlate(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let mut y=vec(&args[1])?;y.reverse();convolve(&[args[0].clone(),json!(y)])}
fn normalize_peak(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let peak=x.iter().map(|v|v.abs()).reduce(f64::max).unwrap_or(0.0);if peak==0.0{return Err("DOMAIN");}Ok(json!(x.into_iter().map(|v|v/peak).collect::<Vec<_>>()))}
fn zero_crossings(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let mut n=0usize;for w in x.windows(2){if (w[0]<0.0&&w[1]>=0.0)||(w[0]>0.0&&w[1]<=0.0){n+=1;}}Ok(json!(n))}
fn cumulative(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let mut s=0.0;let mut out=Vec::with_capacity(x.len());for v in x{s+=v;if !s.is_finite(){return Err("NONFINITE");}out.push(s);}Ok(json!(out))}
fn decimate(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let factor=args[1].as_u64().ok_or("TYPE")? as usize;if factor==0{return Err("DOMAIN");}Ok(json!(x.into_iter().step_by(factor).collect::<Vec<_>>()))}
