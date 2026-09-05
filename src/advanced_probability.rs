use serde_json::{json, Value};

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    if matches!(op,
        "prob.bernoulli_pmf"|"prob.bernoulli_mean"|"prob.bernoulli_variance"|
        "prob.binomial_mean"|"prob.binomial_variance"|"prob.poisson_mean"|"prob.poisson_variance"|
        "prob.geometric_pmf"|"prob.geometric_cdf"|"prob.geometric_mean"|"prob.geometric_variance"|
        "prob.negative_binomial_pmf"|"prob.hypergeometric_pmf"|"prob.normal_z"|"prob.normal_interval"|
        "prob.normal_mean"|"prob.normal_variance"|"prob.exponential_mean"|"prob.exponential_variance"|
        "prob.uniform_mean"|"prob.uniform_variance"|"prob.expected_value"|"prob.discrete_variance"|
        "prob.normalize_weights"|"prob.logit"|"prob.inv_logit"|"prob.binary_cross_entropy"|
        "prob.kl_bernoulli"|"prob.js_bernoulli"|"prob.bayes_positive"|"prob.bayes_negative"|
        "prob.complement"|"prob.union_independent"|"prob.intersection_independent"|"prob.conditional"|
        "prob.total_probability"|"prob.odds_ratio"
    ){Some(run(op,args))}else{None}
}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "prob.bernoulli_pmf"=>{need(args,2)?;let x=args[0].as_u64().ok_or("TYPE")?;let p=prob(&args[1])?;if x>1{return Err("DOMAIN");}finite(if x==1{p}else{1.0-p})},
    "prob.bernoulli_mean"=>{need(args,1)?;finite(prob(&args[0])?)},
    "prob.bernoulli_variance"=>{need(args,1)?;let p=prob(&args[0])?;finite(p*(1.0-p))},
    "prob.binomial_mean"=>{need(args,2)?;finite(args[0].as_u64().ok_or("TYPE")? as f64*prob(&args[1])?)},
    "prob.binomial_variance"=>{need(args,2)?;let n=args[0].as_u64().ok_or("TYPE")? as f64;let p=prob(&args[1])?;finite(n*p*(1.0-p))},
    "prob.poisson_mean"=>{need(args,1)?;let l=positive(&args[0])?;finite(l)},
    "prob.poisson_variance"=>{need(args,1)?;let l=positive(&args[0])?;finite(l)},
    "prob.geometric_pmf"=>geometric(args,false),
    "prob.geometric_cdf"=>geometric(args,true),
    "prob.geometric_mean"=>{need(args,1)?;let p=prob_open(&args[0])?;finite(1.0/p)},
    "prob.geometric_variance"=>{need(args,1)?;let p=prob_open(&args[0])?;finite((1.0-p)/(p*p))},
    "prob.negative_binomial_pmf"=>negative_binomial(args),
    "prob.hypergeometric_pmf"=>hypergeometric(args),
    "prob.normal_z"=>normal_z(args),
    "prob.normal_interval"=>normal_interval(args),
    "prob.normal_mean"=>{need(args,2)?;let mu=num(&args[0])?;let _=positive(&args[1])?;finite(mu)},
    "prob.normal_variance"=>{need(args,2)?;let _=num(&args[0])?;let s=positive(&args[1])?;finite(s*s)},
    "prob.exponential_mean"=>{need(args,1)?;let l=positive(&args[0])?;finite(1.0/l)},
    "prob.exponential_variance"=>{need(args,1)?;let l=positive(&args[0])?;finite(1.0/(l*l))},
    "prob.uniform_mean"=>uniform_moment(args,false),
    "prob.uniform_variance"=>uniform_moment(args,true),
    "prob.expected_value"=>expected(args,false),
    "prob.discrete_variance"=>expected(args,true),
    "prob.normalize_weights"=>normalize_weights(args),
    "prob.logit"=>{need(args,1)?;let p=prob_open(&args[0])?;finite((p/(1.0-p)).ln())},
    "prob.inv_logit"=>{need(args,1)?;let x=num(&args[0])?;finite(if x>=0.0{1.0/(1.0+(-x).exp())}else{let e=x.exp();e/(1.0+e)})},
    "prob.binary_cross_entropy"=>cross_entropy(args),
    "prob.kl_bernoulli"=>kl_bernoulli(args),
    "prob.js_bernoulli"=>js_bernoulli(args),
    "prob.bayes_positive"=>bayes(args,true),
    "prob.bayes_negative"=>bayes(args,false),
    "prob.complement"=>{need(args,1)?;finite(1.0-prob(&args[0])?)},
    "prob.union_independent"=>{need(args,2)?;let a=prob(&args[0])?;let b=prob(&args[1])?;finite(a+b-a*b)},
    "prob.intersection_independent"=>{need(args,2)?;finite(prob(&args[0])?*prob(&args[1])?)},
    "prob.conditional"=>{need(args,2)?;let joint=prob(&args[0])?;let cond=prob(&args[1])?;if cond==0.0{return Err("DIV0");}let r=joint/cond;if r>1.0+1e-12{return Err("DOMAIN");}finite(r)},
    "prob.total_probability"=>total_probability(args),
    "prob.odds_ratio"=>odds_ratio(args),
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn prob(v:&Value)->Result<f64,&'static str>{let p=num(v)?;if (0.0..=1.0).contains(&p){Ok(p)}else{Err("DOMAIN")}}
fn prob_open(v:&Value)->Result<f64,&'static str>{let p=num(v)?;if p>0.0&&p<1.0{Ok(p)}else{Err("DOMAIN")}}
fn positive(v:&Value)->Result<f64,&'static str>{let x=num(v)?;if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn comb(n:u64,k:u64)->f64{if k>n{return 0.0;}let k=k.min(n-k);let mut r=1.0;for i in 1..=k{r*=((n-k+i) as f64)/(i as f64);}r}
fn geometric(args:&[Value],cdf:bool)->Result<Value,&'static str>{need(args,2)?;let k=args[0].as_u64().ok_or("TYPE")?;if k==0{return Err("DOMAIN");}let p=prob_open(&args[1])?;finite(if cdf{1.0-(1.0-p).powf(k as f64)}else{(1.0-p).powf((k-1) as f64)*p})}
fn negative_binomial(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let k=args[0].as_u64().ok_or("TYPE")?;let r=args[1].as_u64().ok_or("TYPE")?;let p=prob_open(&args[2])?;if r==0||k<r{return Err("DOMAIN");}finite(comb(k-1,r-1)*p.powf(r as f64)*(1.0-p).powf((k-r) as f64))}
fn hypergeometric(args:&[Value])->Result<Value,&'static str>{need(args,4)?;let n=args[0].as_u64().ok_or("TYPE")?;let k=args[1].as_u64().ok_or("TYPE")?;let draws=args[2].as_u64().ok_or("TYPE")?;let x=args[3].as_u64().ok_or("TYPE")?;if k>n||draws>n||x>k||x>draws||draws-x>n-k{return Ok(json!(0.0));}let den=comb(n,draws);if den==0.0{return Err("DOMAIN");}finite(comb(k,x)*comb(n-k,draws-x)/den)}
fn normal_z(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let x=num(&args[0])?;let mu=num(&args[1])?;let s=positive(&args[2])?;finite((x-mu)/s)}
fn erf(x:f64)->f64{let sign=if x<0.0{-1.0}else{1.0};let x=x.abs();let t=1.0/(1.0+0.3275911*x);let y=1.0-(((((1.061405429*t-1.453152027)*t+1.421413741)*t-0.284496736)*t+0.254829592)*t)*(-x*x).exp();sign*y}
fn normal_cdf(x:f64,mu:f64,s:f64)->f64{0.5*(1.0+erf((x-mu)/(s*std::f64::consts::SQRT_2)))}
fn normal_interval(args:&[Value])->Result<Value,&'static str>{need(args,4)?;let a=num(&args[0])?;let b=num(&args[1])?;let mu=num(&args[2])?;let s=positive(&args[3])?;if a>b{return Err("DOMAIN");}finite(normal_cdf(b,mu,s)-normal_cdf(a,mu,s))}
fn uniform_moment(args:&[Value],var:bool)->Result<Value,&'static str>{need(args,2)?;let a=num(&args[0])?;let b=num(&args[1])?;if a>=b{return Err("DOMAIN");}finite(if var{(b-a)*(b-a)/12.0}else{(a+b)/2.0})}
fn vectors(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{need(args,2)?;let a=args[0].as_array().ok_or("TYPE")?;let b=args[1].as_array().ok_or("TYPE")?;if a.is_empty()||a.len()!=b.len()||a.len()>100_000{return Err("SHAPE");}Ok((a.iter().map(num).collect::<Result<Vec<_>,_>>()?,b.iter().map(num).collect::<Result<Vec<_>,_>>()?))}
fn expected(args:&[Value],variance:bool)->Result<Value,&'static str>{let(x,p)=vectors(args)?;if p.iter().any(|v|*v<0.0){return Err("DOMAIN");}let s=p.iter().sum::<f64>();if (s-1.0).abs()>1e-9{return Err("DOMAIN");}let m=x.iter().zip(&p).map(|(a,b)|a*b).sum::<f64>();if variance{finite(x.iter().zip(p).map(|(a,b)|b*((*a)-m)*((*a)-m)).sum())}else{finite(m)}}
fn normalize_weights(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let a=args[0].as_array().ok_or("TYPE")?;if a.is_empty(){return Err("EMPTY");}let x=a.iter().map(num).collect::<Result<Vec<_>,_>>()?;if x.iter().any(|v|*v<0.0){return Err("DOMAIN");}let s=x.iter().sum::<f64>();if s==0.0{return Err("DIV0");}Ok(json!(x.into_iter().map(|v|v/s).collect::<Vec<_>>()))}
fn cross_entropy(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let y=prob(&args[0])?;let p=prob_open(&args[1])?;finite(-(y*p.ln()+(1.0-y)*(1.0-p).ln()))}
fn kl_term(a:f64,b:f64)->f64{if a==0.0{0.0}else{a*(a/b).ln()}}
fn kl_bernoulli(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let p=prob(&args[0])?;let q=prob_open(&args[1])?;if p==1.0&&q==1.0{return Ok(json!(0.0));}finite(kl_term(p,q)+kl_term(1.0-p,1.0-q))}
fn js_bernoulli(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let p=prob(&args[0])?;let q=prob(&args[1])?;let m=(p+q)/2.0;let k=|a:f64,b:f64|if a==0.0{0.0}else if b==0.0{f64::INFINITY}else{a*(a/b).ln()};finite(0.5*(k(p,m)+k(1.0-p,1.0-m)+k(q,m)+k(1.0-q,1.0-m)))}
fn bayes(args:&[Value],positive_test:bool)->Result<Value,&'static str>{need(args,3)?;let prior=prob(&args[0])?;let sens=prob(&args[1])?;let spec=prob(&args[2])?;let (a,b)=if positive_test{(sens*prior,(1.0-spec)*(1.0-prior))}else{((1.0-sens)*prior,spec*(1.0-prior))};let d=a+b;if d==0.0{return Err("DOMAIN");}finite(a/d)}
fn total_probability(args:&[Value])->Result<Value,&'static str>{let(c,w)=vectors(args)?;if c.iter().any(|v|*v<0.0||*v>1.0)||w.iter().any(|v|*v<0.0){return Err("DOMAIN");}let s=w.iter().sum::<f64>();if (s-1.0).abs()>1e-9{return Err("DOMAIN");}finite(c.iter().zip(w).map(|(a,b)|(*a)*b).sum())}
fn odds_ratio(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let p1=prob_open(&args[0])?;let p2=prob_open(&args[1])?;finite((p1/(1.0-p1))/(p2/(1.0-p2)))}
