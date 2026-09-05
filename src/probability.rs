use serde_json::{Value, json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "prob.combination" | "prob.permutation" | "prob.binomial_pmf" | "prob.binomial_cdf" |
        "prob.poisson_pmf" | "prob.poisson_cdf" | "prob.normal_pdf" | "prob.normal_cdf" |
        "prob.exponential_pdf" | "prob.exponential_cdf" | "prob.uniform_pdf" | "prob.uniform_cdf" |
        "prob.sigmoid" | "prob.softmax" | "prob.logsumexp" | "prob.binary_entropy" |
        "prob.odds" | "prob.from_odds"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "prob.combination" => comb_op(args),
        "prob.permutation" => perm_op(args),
        "prob.binomial_pmf" => binomial_pmf(args),
        "prob.binomial_cdf" => binomial_cdf(args),
        "prob.poisson_pmf" => poisson_pmf(args),
        "prob.poisson_cdf" => poisson_cdf(args),
        "prob.normal_pdf" => normal_pdf(args),
        "prob.normal_cdf" => normal_cdf(args),
        "prob.exponential_pdf" => exponential_pdf(args),
        "prob.exponential_cdf" => exponential_cdf(args),
        "prob.uniform_pdf" => uniform_pdf(args),
        "prob.uniform_cdf" => uniform_cdf(args),
        "prob.sigmoid" => finite(sigmoid(one(args)?)),
        "prob.softmax" => softmax(args),
        "prob.logsumexp" => logsumexp(args),
        "prob.binary_entropy" => binary_entropy(args),
        "prob.odds" => { let p=one(args)?; if !(0.0..1.0).contains(&p){Err("DOMAIN")}else{finite(p/(1.0-p))} },
        "prob.from_odds" => { let o=one(args)?; if o<0.0{Err("DOMAIN")}else{finite(o/(1.0+o))} },
        _ => Err("OP"),
    }
}

fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{if args.len()!=1{return Err("ARG");}num(&args[0])}
fn uint(v:&Value)->Result<u64,&'static str>{v.as_u64().ok_or("TYPE")}
fn vec_from(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>100_000{return Err("LIMIT");}a.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}

fn comb(n:u64,k:u64)->Result<f64,&'static str>{if k>n{return Err("DOMAIN");}let k=k.min(n-k);let mut x=1.0;for i in 1..=k{x*=((n-k+i) as f64)/(i as f64);if !x.is_finite(){return Err("NONFINITE");}}Ok(x)}
fn perm(n:u64,k:u64)->Result<f64,&'static str>{if k>n{return Err("DOMAIN");}let mut x=1.0;for i in 0..k{x*=(n-i) as f64;if !x.is_finite(){return Err("NONFINITE");}}Ok(x)}
fn comb_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}finite(comb(uint(&args[0])?,uint(&args[1])?)?)}
fn perm_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}finite(perm(uint(&args[0])?,uint(&args[1])?)?)}

fn binomial_pmf(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let n=uint(&args[0])?;let k=uint(&args[1])?;let p=num(&args[2])?;if k>n||!(0.0..=1.0).contains(&p){return Err("DOMAIN");}finite(comb(n,k)?*p.powf(k as f64)*(1.0-p).powf((n-k) as f64))}
fn binomial_cdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let n=uint(&args[0])?;let k=uint(&args[1])?.min(n);let p=num(&args[2])?;if !(0.0..=1.0).contains(&p)||n>100_000{return Err("DOMAIN");}let mut s=0.0;for i in 0..=k{s+=comb(n,i)?*p.powf(i as f64)*(1.0-p).powf((n-i) as f64);}finite(s.min(1.0))}

fn factorial_f64(k:u64)->Result<f64,&'static str>{if k>170{return Err("LIMIT");}let mut f=1.0;for i in 2..=k{f*=i as f64;}Ok(f)}
fn poisson_pmf(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let lambda=num(&args[0])?;let k=uint(&args[1])?;if lambda<0.0{return Err("DOMAIN");}finite((-lambda).exp()*lambda.powf(k as f64)/factorial_f64(k)?)}
fn poisson_cdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let lambda=num(&args[0])?;let k=uint(&args[1])?;if lambda<0.0||k>170{return Err("DOMAIN");}let mut s=0.0;for i in 0..=k{s+=(-lambda).exp()*lambda.powf(i as f64)/factorial_f64(i)?;}finite(s.min(1.0))}

fn normal_pdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let mu=num(&args[1])?;let sd=num(&args[2])?;if sd<=0.0{return Err("DOMAIN");}let z=(x-mu)/sd;finite((-0.5*z*z).exp()/(sd*(2.0*std::f64::consts::PI).sqrt()))}
fn normal_cdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let mu=num(&args[1])?;let sd=num(&args[2])?;if sd<=0.0{return Err("DOMAIN");}finite(0.5*(1.0+erf((x-mu)/(sd*2.0_f64.sqrt()))))}

fn erf(x:f64)->f64{
    let sign=if x<0.0{-1.0}else{1.0};let x=x.abs();let t=1.0/(1.0+0.3275911*x);
    let a1=0.254829592;let a2=-0.284496736;let a3=1.421413741;let a4=-1.453152027;let a5=1.061405429;
    sign*(1.0-(((((a5*t+a4)*t)+a3)*t+a2)*t+a1)*t*(-x*x).exp())
}

fn exponential_pdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=num(&args[0])?;let rate=num(&args[1])?;if rate<=0.0{return Err("DOMAIN");}finite(if x<0.0{0.0}else{rate*(-rate*x).exp()})}
fn exponential_cdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=num(&args[0])?;let rate=num(&args[1])?;if rate<=0.0{return Err("DOMAIN");}finite(if x<0.0{0.0}else{1.0-(-rate*x).exp()})}
fn uniform_pdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let a=num(&args[1])?;let b=num(&args[2])?;if a>=b{return Err("DOMAIN");}finite(if x<a||x>b{0.0}else{1.0/(b-a)})}
fn uniform_cdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let a=num(&args[1])?;let b=num(&args[2])?;if a>=b{return Err("DOMAIN");}finite(if x<=a{0.0}else if x>=b{1.0}else{(x-a)/(b-a)})}
fn sigmoid(x:f64)->f64{if x>=0.0{1.0/(1.0+(-x).exp())}else{let e=x.exp();e/(1.0+e)}}
fn softmax(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let xs=vec_from(&args[0])?;if xs.is_empty(){return Err("EMPTY");}let m=xs.iter().copied().reduce(f64::max).unwrap();let es:Vec<f64>=xs.iter().map(|x|(x-m).exp()).collect();let s: f64=es.iter().sum();if !s.is_finite()||s==0.0{return Err("NONFINITE");}Ok(json!(es.into_iter().map(|x|x/s).collect::<Vec<_>>()))}
fn logsumexp(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let xs=vec_from(&args[0])?;if xs.is_empty(){return Err("EMPTY");}let m=xs.iter().copied().reduce(f64::max).unwrap();finite(m+xs.iter().map(|x|(x-m).exp()).sum::<f64>().ln())}
fn binary_entropy(args:&[Value])->Result<Value,&'static str>{let p=one(args)?;if !(0.0..=1.0).contains(&p){return Err("DOMAIN");}if p==0.0||p==1.0{return finite(0.0);}finite(-p*p.log2()-(1.0-p)*(1.0-p).log2())}
