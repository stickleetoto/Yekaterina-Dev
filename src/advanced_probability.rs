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
        "prob.total_probability"|"prob.odds_ratio"|
        "prob.t_pdf"|"prob.t_cdf"|"prob.t_sf"|"prob.t_ppf"|
        "prob.chi2_pdf"|"prob.chi2_cdf"|"prob.chi2_sf"|"prob.chi2_ppf"|
        "prob.f_pdf"|"prob.f_cdf"|"prob.f_sf"|"prob.f_ppf"|
        "prob.normal_sf"|"prob.normal_ppf"|
        "prob.gamma_pdf"|"prob.gamma_cdf"|"prob.beta_pdf"|"prob.beta_cdf"|
        "prob.lognormal_pdf"|"prob.lognormal_cdf"|"prob.weibull_pdf"|"prob.weibull_cdf"
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
    "prob.t_pdf"=>{let(x,df)=x_and_df(args)?;out(t_pdf(x,df))},
    "prob.t_cdf"=>{let(x,df)=x_and_df(args)?;out(t_cdf(x,df))},
    "prob.t_sf"=>{let(x,df)=x_and_df(args)?;out(t_sf(x,df))},
    "prob.t_ppf"=>{need(args,2)?;let p=prob(&args[0])?;let df=positive(&args[1])?;out(t_ppf(p,df))},
    "prob.chi2_pdf"=>{let(x,k)=x_and_df(args)?;out(chi2_pdf(x,k))},
    "prob.chi2_cdf"=>{let(x,k)=x_and_df(args)?;out(chi2_cdf(x,k))},
    "prob.chi2_sf"=>{let(x,k)=x_and_df(args)?;out(chi2_sf(x,k))},
    "prob.chi2_ppf"=>{need(args,2)?;let p=prob(&args[0])?;let k=positive(&args[1])?;out(chi2_ppf(p,k))},
    "prob.f_pdf"=>{let(x,a,b)=x_and_two_df(args)?;out(f_pdf(x,a,b))},
    "prob.f_cdf"=>{let(x,a,b)=x_and_two_df(args)?;out(f_cdf(x,a,b))},
    "prob.f_sf"=>{let(x,a,b)=x_and_two_df(args)?;out(f_sf(x,a,b))},
    "prob.f_ppf"=>{need(args,3)?;let p=prob(&args[0])?;let a=positive(&args[1])?;let b=positive(&args[2])?;out(f_ppf(p,a,b))},
    "prob.normal_sf"=>{need(args,3)?;let x=num(&args[0])?;let m=num(&args[1])?;let s=positive(&args[2])?;out(normal_sf_std((x-m)/s))},
    "prob.normal_ppf"=>{need(args,3)?;let p=prob(&args[0])?;let m=num(&args[1])?;let s=positive(&args[2])?;out(m+s*normal_ppf_std(p))},
    "prob.gamma_pdf"=>{let(x,k,t)=x_and_two_positive(args)?;if x<0.0{return Err("DOMAIN");}
        out(if x==0.0{if k<1.0{f64::INFINITY}else if k==1.0{1.0/t}else{0.0}}
            else{((k-1.0)*(x/t).ln()-x/t-t.ln()-libm::lgamma(k)).exp()})},
    "prob.gamma_cdf"=>{let(x,k,t)=x_and_two_positive(args)?;if x<0.0{return Err("DOMAIN");}out(gamma_p(k,x/t))},
    "prob.beta_pdf"=>{need(args,3)?;let x=num(&args[0])?;let a=positive(&args[1])?;let b=positive(&args[2])?;
        if !(0.0..=1.0).contains(&x){return Err("DOMAIN");}
        out(if x==0.0||x==1.0{
                let edge=if x==0.0{a}else{b};
                if edge<1.0{f64::INFINITY}else if edge==1.0{((1.0-edge)*0.0-ln_beta(a,b)).exp()}else{0.0}
            }else{((a-1.0)*x.ln()+(b-1.0)*(1.0-x).ln()-ln_beta(a,b)).exp()})},
    "prob.beta_cdf"=>{need(args,3)?;let x=num(&args[0])?;let a=positive(&args[1])?;let b=positive(&args[2])?;
        if !(0.0..=1.0).contains(&x){return Err("DOMAIN");}out(beta_inc(a,b,x))},
    "prob.lognormal_pdf"=>{need(args,3)?;let x=num(&args[0])?;let m=num(&args[1])?;let s=positive(&args[2])?;
        if x<0.0{return Err("DOMAIN");}
        out(if x==0.0{0.0}else{let z=(x.ln()-m)/s;(-0.5*z*z).exp()/(x*s*(2.0*std::f64::consts::PI).sqrt())})},
    "prob.lognormal_cdf"=>{need(args,3)?;let x=num(&args[0])?;let m=num(&args[1])?;let s=positive(&args[2])?;
        if x<0.0{return Err("DOMAIN");}out(if x==0.0{0.0}else{normal_cdf_std((x.ln()-m)/s)})},
    "prob.weibull_pdf"=>{let(x,k,l)=x_and_two_positive(args)?;if x<0.0{return Err("DOMAIN");}
        out(if x==0.0{if k<1.0{f64::INFINITY}else if k==1.0{1.0/l}else{0.0}}
            else{(k/l)*(x/l).powf(k-1.0)*(-(x/l).powf(k)).exp()})},
    "prob.weibull_cdf"=>{let(x,k,l)=x_and_two_positive(args)?;if x<0.0{return Err("DOMAIN");}
        out(-(-(x/l).powf(k)).exp_m1())},
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

/// A density or probability that is allowed to be exactly 0 or 1 but never NaN;
/// NaN means the caller asked for a point outside the distribution's support.
fn out(x: f64) -> Result<Value, &'static str> {
    if x.is_nan() { Err("DOMAIN") } else if x.is_infinite() { Err("NONFINITE") } else { Ok(json!(x)) }
}
fn x_and_df(args: &[Value]) -> Result<(f64, f64), &'static str> {
    need(args, 2)?;
    Ok((num(&args[0])?, positive(&args[1])?))
}
fn x_and_two_df(args: &[Value]) -> Result<(f64, f64, f64), &'static str> {
    need(args, 3)?;
    Ok((num(&args[0])?, positive(&args[1])?, positive(&args[2])?))
}
fn x_and_two_positive(args: &[Value]) -> Result<(f64, f64, f64), &'static str> {
    need(args, 3)?;
    Ok((num(&args[0])?, positive(&args[1])?, positive(&args[2])?))
}

// ---------------------------------------------------------------------------
// Continuous distributions built on the regularized incomplete gamma and beta.
//
// Each distribution exposes a survival function alongside its CDF rather than
// leaving callers to compute 1 - cdf. A p-value is a tail probability, and
// 1 - 0.9999999999999999 keeps one significant digit where the survival branch
// keeps all of them. That difference decides whether a reported p-value of
// 1e-15 means anything.
//
// Every quantile is found by bisection with a fixed iteration count over a
// bracket that is grown deterministically. Newton would be faster and would
// make the answer depend on the starting guess and the floating-point path.

use crate::special_functions::{beta_inc, gamma_p, gamma_q};

/// Bisection steps. Far more than needed to reach f64 precision on any bracket
/// this is used with; fixed so the result cannot depend on how fast a
/// particular input happens to converge.
const QUANTILE_ITERS: usize = 200;
/// Doublings allowed while growing a quantile bracket before giving up.
const BRACKET_GROWTH: usize = 200;

fn ln_beta(a: f64, b: f64) -> f64 {
    libm::lgamma(a) + libm::lgamma(b) - libm::lgamma(a + b)
}

// ------------------------------------------------------------------ Student t

fn t_pdf(x: f64, df: f64) -> f64 {
    let h = (df + 1.0) / 2.0;
    (libm::lgamma(h) - libm::lgamma(df / 2.0) - 0.5 * (df * std::f64::consts::PI).ln()
        - h * (1.0 + x * x / df).ln()).exp()
}

/// Lower tail. The incomplete beta is evaluated on the side that keeps its
/// accuracy, and the sign of x selects which side that is.
pub(crate) fn t_cdf(x: f64, df: f64) -> f64 {
    let z = df / (df + x * x);
    let half = 0.5 * beta_inc(df / 2.0, 0.5, z);
    if x > 0.0 { 1.0 - half } else { half }
}

pub(crate) fn t_sf(x: f64, df: f64) -> f64 { t_cdf(-x, df) }

// ---------------------------------------------------------------- chi-squared

fn chi2_pdf(x: f64, k: f64) -> f64 {
    if x < 0.0 { return f64::NAN; }
    if x == 0.0 {
        return if k < 2.0 { f64::INFINITY } else if k == 2.0 { 0.5 } else { 0.0 };
    }
    let h = k / 2.0;
    ((h - 1.0) * x.ln() - x / 2.0 - h * std::f64::consts::LN_2 - libm::lgamma(h)).exp()
}

pub(crate) fn chi2_cdf(x: f64, k: f64) -> f64 { if x <= 0.0 { 0.0 } else { gamma_p(k / 2.0, x / 2.0) } }
pub(crate) fn chi2_sf(x: f64, k: f64) -> f64 { if x <= 0.0 { 1.0 } else { gamma_q(k / 2.0, x / 2.0) } }

// -------------------------------------------------------------------------- F

fn f_pdf(x: f64, d1: f64, d2: f64) -> f64 {
    if x < 0.0 { return f64::NAN; }
    if x == 0.0 { return if d1 < 2.0 { f64::INFINITY } else if d1 == 2.0 { 1.0 } else { 0.0 }; }
    let num = d1 * 0.5 * (d1 / d2).ln() + (d1 * 0.5 - 1.0) * x.ln()
        - (d1 + d2) * 0.5 * (1.0 + d1 * x / d2).ln();
    (num - ln_beta(d1 / 2.0, d2 / 2.0)).exp()
}

pub(crate) fn f_cdf(x: f64, d1: f64, d2: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    beta_inc(d1 / 2.0, d2 / 2.0, d1 * x / (d1 * x + d2))
}

/// Upper tail taken directly from the mirrored incomplete beta, not as 1 - cdf.
pub(crate) fn f_sf(x: f64, d1: f64, d2: f64) -> f64 {
    if x <= 0.0 { return 1.0; }
    beta_inc(d2 / 2.0, d1 / 2.0, d2 / (d1 * x + d2))
}

// --------------------------------------------------------------------- normal

pub(crate) fn normal_cdf_std(z: f64) -> f64 { 0.5 * libm::erfc(-z / std::f64::consts::SQRT_2) }
pub(crate) fn normal_sf_std(z: f64) -> f64 { 0.5 * libm::erfc(z / std::f64::consts::SQRT_2) }

/// Inverse standard normal CDF. Acklam's rational approximation, then two
/// Halley refinements against erfc, which brings it to f64 accuracy.
pub(crate) fn normal_ppf_std(p: f64) -> f64 {
    if !(0.0..=1.0).contains(&p) { return f64::NAN; }
    if p == 0.0 { return f64::NEG_INFINITY; }
    if p == 1.0 { return f64::INFINITY; }
    const A: [f64; 6] = [-3.969683028665376e+01, 2.209460984245205e+02, -2.759285104469687e+02,
                         1.38357751867269e+02, -3.066479806614716e+01, 2.506628277459239e+00];
    const B: [f64; 5] = [-5.447609879822406e+01, 1.615858368580409e+02, -1.556989798598866e+02,
                         6.680131188771972e+01, -1.328068155288572e+01];
    const C: [f64; 6] = [-7.784894002430293e-03, -3.223964580411365e-01, -2.400758277161838e+00,
                         -2.549732539343734e+00, 4.374664141464968e+00, 2.938163982698783e+00];
    const D: [f64; 4] = [7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e+00,
                         3.754408661907416e+00];
    const P_LOW: f64 = 0.02425;
    let mut x = if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if p <= 1.0 - P_LOW {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    for _ in 0..2 {
        let e = normal_cdf_std(x) - p;
        let u = e * (2.0 * std::f64::consts::PI).sqrt() * (x * x / 2.0).exp();
        x -= u / (1.0 + x * u / 2.0);
    }
    x
}

// ------------------------------------------------------------------ quantiles

/// Bisection on a monotone CDF. `lo` and `hi` must bracket the answer; the
/// iteration count is fixed, so two runs on the same input do the same work.
fn bisect<F: Fn(f64) -> f64>(cdf: F, p: f64, mut lo: f64, mut hi: f64) -> f64 {
    for _ in 0..QUANTILE_ITERS {
        let mid = 0.5 * (lo + hi);
        if mid == lo || mid == hi { break; }
        if cdf(mid) < p { lo = mid; } else { hi = mid; }
    }
    0.5 * (lo + hi)
}

/// Grows an upper bound until the CDF passes p, doubling from a scale-aware
/// start rather than from 1 so heavy-tailed cases do not need many steps.
fn grow_upper<F: Fn(f64) -> f64>(cdf: &F, p: f64, start: f64) -> Option<f64> {
    let mut hi = start.max(1.0);
    for _ in 0..BRACKET_GROWTH {
        if cdf(hi) >= p { return Some(hi); }
        hi *= 2.0;
    }
    None
}

/// Bisection on a monotonically *decreasing* function, used to invert a
/// survival function.
fn bisect_down<F: Fn(f64) -> f64>(sf: F, q: f64, mut lo: f64, mut hi: f64) -> f64 {
    for _ in 0..QUANTILE_ITERS {
        let mid = 0.5 * (lo + hi);
        if mid == lo || mid == hi { break; }
        if sf(mid) > q { lo = mid; } else { hi = mid; }
    }
    0.5 * (lo + hi)
}

fn grow_upper_down<F: Fn(f64) -> f64>(sf: &F, q: f64, start: f64) -> Option<f64> {
    let mut hi = start.max(1.0);
    for _ in 0..BRACKET_GROWTH {
        if sf(hi) <= q { return Some(hi); }
        hi *= 2.0;
    }
    None
}

// Which branch a quantile is solved on decides its accuracy. Asking for the
// 1e-8 quantile by solving cdf(x) = 1 - 1e-8 throws away eight digits before
// the search starts, because 1 - 1e-8 is only known to about 1e-16 absolute.
// Each quantile below therefore solves the CDF for the lower half and the
// survival function for the upper half, and never forms 1 - p from a p it was
// given on the tail it is already working on.

fn chi2_ppf(p: f64, k: f64) -> f64 {
    if p <= 0.0 { return 0.0; }
    if p >= 1.0 { return f64::INFINITY; }
    let start = k + 10.0 * (2.0 * k).sqrt() + 10.0;
    if p <= 0.5 {
        let cdf = |x: f64| chi2_cdf(x, k);
        match grow_upper(&cdf, p, start) { Some(hi) => bisect(cdf, p, 0.0, hi), None => f64::NAN }
    } else {
        let q = 1.0 - p;
        let sf = |x: f64| chi2_sf(x, k);
        match grow_upper_down(&sf, q, start) { Some(hi) => bisect_down(sf, q, 0.0, hi), None => f64::NAN }
    }
}

fn f_ppf(p: f64, d1: f64, d2: f64) -> f64 {
    if p <= 0.0 { return 0.0; }
    if p >= 1.0 { return f64::INFINITY; }
    if p <= 0.5 {
        let cdf = |x: f64| f_cdf(x, d1, d2);
        match grow_upper(&cdf, p, 4.0) { Some(hi) => bisect(cdf, p, 0.0, hi), None => f64::NAN }
    } else {
        let q = 1.0 - p;
        let sf = |x: f64| f_sf(x, d1, d2);
        match grow_upper_down(&sf, q, 4.0) { Some(hi) => bisect_down(sf, q, 0.0, hi), None => f64::NAN }
    }
}

fn t_ppf(p: f64, df: f64) -> f64 {
    if p <= 0.0 { return f64::NEG_INFINITY; }
    if p >= 1.0 { return f64::INFINITY; }
    if p == 0.5 { return 0.0; }
    // t is symmetric, so the upper half is the mirror of the lower one. Solving
    // the lower half means solving the CDF where it is small, which is where it
    // is accurate.
    if p > 0.5 { return -t_ppf(1.0 - p, df); }
    let sf = |x: f64| t_cdf(-x, df);   // decreasing in x, equals the lower tail at -x
    match grow_upper_down(&sf, p, 4.0) {
        Some(hi) => -bisect_down(sf, p, 0.0, hi),
        None => f64::NAN,
    }
}
