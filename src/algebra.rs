use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use serde_json::{json, Value};

const MAX_N: u64 = 1_000_000;
const MAX_BIG_N: u64 = 100_000;

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if op.starts_with("alg.") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "alg.factorial" => factorial(args),
        "alg.double_factorial" => double_factorial(args),
        "alg.fibonacci" => fibonacci(args),
        "alg.lucas" => lucas(args),
        "alg.combination_exact" => combination_exact(args),
        "alg.permutation_exact" => permutation_exact(args),
        "alg.catalan" => catalan(args),
        "alg.stirling2" => stirling2(args),
        "alg.gcd_many" => gcd_many(args),
        "alg.lcm_many" => lcm_many(args),
        "alg.is_prime" => is_prime_op(args),
        "alg.next_prime" => next_prime(args),
        "alg.prev_prime" => prev_prime(args),
        "alg.prime_count" => prime_count(args),
        "alg.prime_factors" => prime_factors_op(args),
        "alg.divisors" => divisors_op(args),
        "alg.divisor_count" => divisor_count(args),
        "alg.divisor_sum" => divisor_sum(args),
        "alg.totient" => totient(args),
        "alg.mobius" => mobius(args),
        "alg.mod_pow" => mod_pow(args),
        "alg.mod_inverse" => mod_inverse(args),
        "alg.ext_gcd" => ext_gcd_op(args),
        "alg.crt_pair" => crt_pair(args),
        "alg.floor_div" => floor_div(args),
        "alg.ceil_div" => ceil_div(args),
        "alg.quadratic_roots" => quadratic_roots(args),
        "alg.poly_eval" => poly_eval(args),
        "alg.poly_derivative" => poly_derivative(args),
        "alg.poly_integral" => poly_integral(args),
        "alg.synthetic_div" => synthetic_div(args),
        "alg.arithmetic_sum" => arithmetic_sum(args),
        "alg.geometric_sum" => geometric_sum(args),
        "alg.harmonic_number" => harmonic_number(args),
        "alg.triangular" => triangular(args),
        "alg.square_pyramidal" => square_pyramidal(args),
        _ => Err("OP"),
    }
}

fn u64arg(v: &Value) -> Result<u64, &'static str> { v.as_u64().ok_or("TYPE") }
fn i64arg(v: &Value) -> Result<i64, &'static str> { v.as_i64().ok_or("TYPE") }
fn f64arg(v: &Value) -> Result<f64, &'static str> { v.as_f64().ok_or("TYPE") }
fn need(args: &[Value], n: usize) -> Result<(), &'static str> { if args.len() == n { Ok(()) } else { Err("ARG") } }
fn finite(x: f64) -> Result<Value, &'static str> { if x.is_finite() { Ok(json!(x)) } else { Err("NONFINITE") } }
fn bigint_from_value(v: &Value) -> Result<BigInt, &'static str> {
    match v {
        Value::String(s) => s.parse::<BigInt>().map_err(|_| "TYPE"),
        _ if v.as_i64().is_some() => Ok(BigInt::from(v.as_i64().unwrap())),
        _ if v.as_u64().is_some() => Ok(BigInt::from(v.as_u64().unwrap())),
        _ => Err("TYPE"),
    }
}
fn bigint_array(v: &Value) -> Result<Vec<BigInt>, &'static str> {
    let a = v.as_array().ok_or("TYPE")?;
    if a.is_empty() { return Err("EMPTY"); }
    if a.len() > 100_000 { return Err("LIMIT"); }
    a.iter().map(bigint_from_value).collect()
}

fn factorial(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 1)?; let n = u64arg(&args[0])?; if n > MAX_BIG_N { return Err("LIMIT"); }
    let mut out = BigInt::one(); for i in 2..=n { out *= i; } Ok(json!(out.to_string()))
}
fn double_factorial(args: &[Value]) -> Result<Value, &'static str> {
    need(args, 1)?; let n = u64arg(&args[0])?; if n > MAX_BIG_N { return Err("LIMIT"); }
    let mut out = BigInt::one(); let start = if n <= 1 { 1 } else { n }; let mut i = start;
    while i > 1 { out *= i; i = i.saturating_sub(2); }
    Ok(json!(out.to_string()))
}
fn fib_pair(n: u64) -> (BigInt, BigInt) {
    if n == 0 { return (BigInt::zero(), BigInt::one()); }
    let (a, b) = fib_pair(n / 2);
    let c = &a * ((&b << 1u32) - &a);
    let d = &a * &a + &b * &b;
    if n % 2 == 0 { (c, d) } else { (d.clone(), c + d) }
}
fn fibonacci(args: &[Value]) -> Result<Value, &'static str> {
    need(args,1)?; let n=u64arg(&args[0])?; if n>1_000_000{return Err("LIMIT");} Ok(json!(fib_pair(n).0.to_string()))
}
fn lucas(args: &[Value]) -> Result<Value, &'static str> {
    need(args,1)?; let n=u64arg(&args[0])?; if n>1_000_000{return Err("LIMIT");}
    if n==0 { return Ok(json!("2")); } let (f, fp1)=fib_pair(n); let fm1=&fp1-&f; Ok(json!((fm1+fp1).to_string()))
}
fn combination_big(n:u64,k:u64)->BigInt{
    if k>n{return BigInt::zero();} let k=k.min(n-k); let mut r=BigInt::one();
    for i in 1..=k { r *= n-k+i; r /= i; } r
}
fn combination_exact(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let n=u64arg(&args[0])?;let k=u64arg(&args[1])?;if n>MAX_BIG_N{return Err("LIMIT");}Ok(json!(combination_big(n,k).to_string()))}
fn permutation_exact(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let n=u64arg(&args[0])?;let k=u64arg(&args[1])?;if k>n{return Err("DOMAIN");}if n>MAX_BIG_N{return Err("LIMIT");}let mut r=BigInt::one();for i in 0..k{r*=n-i;}Ok(json!(r.to_string()))}
fn catalan(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n>20_000{return Err("LIMIT");}let c=combination_big(2*n,n)/BigInt::from(n+1);Ok(json!(c.to_string()))}
fn stirling2(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let n=u64arg(&args[0])? as usize;let k=u64arg(&args[1])? as usize;if n>1000||k>1000{return Err("LIMIT");}if k>n{return Ok(json!("0"));}let mut dp=vec![BigInt::zero();k+1];dp[0]=BigInt::one();for i in 1..=n{let upto=i.min(k);for j in (1..=upto).rev(){dp[j]=&dp[j-1]+&dp[j]*BigInt::from(j as u64);}dp[0]=BigInt::zero();}Ok(json!(dp[k].to_string()))}
fn gcd_many(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let xs=bigint_array(&args[0])?;let mut g=BigInt::zero();for x in xs{g=g.gcd(&x);}Ok(json!(g.abs().to_string()))}
fn lcm_many(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let xs=bigint_array(&args[0])?;let mut l=BigInt::one();for x in xs{l=l.lcm(&x);}Ok(json!(l.abs().to_string()))}
fn is_prime(n:u64)->bool{if n<2{return false;}if n%2==0{return n==2;}if n%3==0{return n==3;}let mut i=5u64;while i<=n/i{if n%i==0||n%(i+2)==0{return false;}i+=6;}true}
fn is_prime_op(args:&[Value])->Result<Value,&'static str>{need(args,1)?;Ok(json!(is_prime(u64arg(&args[0])?)))}
fn next_prime(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let mut n=u64arg(&args[0])?.saturating_add(1);if n<2{n=2;}while !is_prime(n){n=n.checked_add(1).ok_or("LIMIT")?;}Ok(json!(n))}
fn prev_prime(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let mut n=u64arg(&args[0])?;if n<=2{return Err("DOMAIN");}n-=1;while n>=2&&!is_prime(n){n-=1;}if n<2{Err("DOMAIN")}else{Ok(json!(n))}}
fn prime_count(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n>10_000_000{return Err("LIMIT");}if n<2{return Ok(json!(0));}let mut sieve=vec![true;(n as usize)+1];sieve[0]=false;sieve[1]=false;let lim=(n as f64).sqrt() as usize;for p in 2..=lim{if sieve[p]{let mut m=p*p;while m<=n as usize{sieve[m]=false;m+=p;}}}Ok(json!(sieve.into_iter().filter(|x|*x).count()))}
fn factors(mut n:u64)->Vec<(u64,u32)>{let mut out=Vec::new();let mut p=2u64;while p<=n/p{if n%p==0{let mut e=0;while n%p==0{n/=p;e+=1;}out.push((p,e));}p=if p==2{3}else{p+2};}if n>1{out.push((n,1));}out}
fn prime_factors_op(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n==0{return Err("DOMAIN");}Ok(json!(factors(n).into_iter().map(|(p,e)|json!([p,e])).collect::<Vec<_>>()))}
fn divisors_from_factors(fs:&[(u64,u32)])->Vec<u64>{let mut d=vec![1u64];for&(p,e)in fs{let base=d.clone();let mut pe=1u64;for _ in 0..e{pe=pe.saturating_mul(p);for &x in &base{d.push(x.saturating_mul(pe));}}}d.sort_unstable();d}
fn divisors_op(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n==0||n>1_000_000_000_000{return Err("DOMAIN");}Ok(json!(divisors_from_factors(&factors(n))))}
fn divisor_count(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n==0{return Err("DOMAIN");}let c=factors(n).into_iter().fold(1u64,|a,(_,e)|a*(e as u64+1));Ok(json!(c))}
fn divisor_sum(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n==0{return Err("DOMAIN");}let mut total=BigInt::one();for(p,e)in factors(n){let mut s=BigInt::one();let mut pe=BigInt::one();for _ in 0..e{pe*=p;s+=&pe;}total*=s;}Ok(json!(total.to_string()))}
fn totient(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n==0{return Err("DOMAIN");}let mut r=n;for(p,_)in factors(n){r=r/p*(p-1);}Ok(json!(r))}
fn mobius(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n==0{return Err("DOMAIN");}let fs=factors(n);if fs.iter().any(|(_,e)|*e>1){Ok(json!(0))}else{Ok(json!(if fs.len()%2==0{1}else{-1}))}}
fn mod_pow(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let mut base=u64arg(&args[0])?;let mut exp=u64arg(&args[1])?;let m=u64arg(&args[2])?;if m==0{return Err("DIV0");}base%=m;let mut r=1u128;let mm=m as u128;let mut b=base as u128;while exp>0{if exp&1==1{r=r*b%mm;}b=b*b%mm;exp>>=1;}Ok(json!(r as u64))}
fn egcd(a:i128,b:i128)->(i128,i128,i128){if b==0{(a.abs(),a.signum(),0)}else{let(g,x1,y1)=egcd(b,a%b);(g,y1,x1-(a/b)*y1)}}
fn mod_inverse(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let a=i64arg(&args[0])? as i128;let m=i64arg(&args[1])? as i128;if m<=0{return Err("DOMAIN");}let(g,x,_)=egcd(a,m);if g!=1{return Err("DOMAIN");}let inv=((x%m)+m)%m;Ok(json!(inv as i64))}
fn ext_gcd_op(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let a=i64arg(&args[0])? as i128;let b=i64arg(&args[1])? as i128;let(g,x,y)=egcd(a,b);if g>i64::MAX as i128||x<i64::MIN as i128||x>i64::MAX as i128||y<i64::MIN as i128||y>i64::MAX as i128{return Err("LIMIT");}Ok(json!([g as i64,x as i64,y as i64]))}
fn crt_pair(args:&[Value])->Result<Value,&'static str>{need(args,4)?;let a1=i64arg(&args[0])? as i128;let m1=i64arg(&args[1])? as i128;let a2=i64arg(&args[2])? as i128;let m2=i64arg(&args[3])? as i128;if m1<=0||m2<=0{return Err("DOMAIN");}let(g,s,_)=egcd(m1,m2);let diff=a2-a1;if diff%g!=0{return Err("DOMAIN");}let l=m1/g*m2;let k=((diff/g)*s).rem_euclid(m2/g);let x=(a1+m1*k).rem_euclid(l);if x>i64::MAX as i128||l>i64::MAX as i128{return Err("LIMIT");}Ok(json!([x as i64,l as i64]))}
fn floor_div(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let a=i64arg(&args[0])?;let b=i64arg(&args[1])?;if b==0{return Err("DIV0");}Ok(json!(num_integer::Integer::div_floor(&a, &b)))}
fn ceil_div(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let a=i64arg(&args[0])?;let b=i64arg(&args[1])?;if b==0{return Err("DIV0");}Ok(json!(num_integer::Integer::div_ceil(&a, &b)))}
fn quadratic_roots(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let a=f64arg(&args[0])?;let b=f64arg(&args[1])?;let c=f64arg(&args[2])?;if a==0.0{return Err("DOMAIN");}let d=b*b-4.0*a*c;if d<0.0{return Err("DOMAIN");}let s=d.sqrt();let q=-0.5*(b+b.signum()*s);let (r1,r2)=if q==0.0{(-b/(2.0*a),-b/(2.0*a))}else{(q/a,c/q)};let mut r=vec![r1,r2];r.sort_by(f64::total_cmp);Ok(json!(r))}
fn coeffs(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.is_empty(){return Err("EMPTY");}if a.len()>100_000{return Err("LIMIT");}a.iter().map(f64arg).collect()}
fn poly_eval(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let c=coeffs(&args[0])?;let x=f64arg(&args[1])?;let mut y=0.0;for a in c{y=y*x+a;}finite(y)}
fn poly_derivative(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let c=coeffs(&args[0])?;if c.len()==1{return Ok(json!([0.0]));}let n=c.len()-1;Ok(json!(c.into_iter().take(n).enumerate().map(|(i,a)|a*(n-i) as f64).collect::<Vec<_>>()))}
fn poly_integral(args:&[Value])->Result<Value,&'static str>{if args.len()<1||args.len()>2{return Err("ARG");}let c=coeffs(&args[0])?;let c0=if args.len()==2{f64arg(&args[1])?}else{0.0};let n=c.len();let mut out=Vec::with_capacity(n+1);for(i,a)in c.into_iter().enumerate(){out.push(a/(n-i) as f64);}out.push(c0);Ok(json!(out))}
fn synthetic_div(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let c=coeffs(&args[0])?;if c.len()<2{return Err("SHAPE");}let r=f64arg(&args[1])?;let mut q=Vec::with_capacity(c.len()-1);let mut acc=c[0];q.push(acc);for a in c.iter().skip(1).take(c.len()-2){acc=*a+acc*r;q.push(acc);}let rem=c[c.len()-1]+acc*r;Ok(json!({"q":q,"r":rem}))}
fn arithmetic_sum(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let a=f64arg(&args[0])?;let d=f64arg(&args[1])?;let n=u64arg(&args[2])? as f64;finite(n*(2.0*a+(n-1.0)*d)/2.0)}
fn geometric_sum(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let a=f64arg(&args[0])?;let r=f64arg(&args[1])?;let nu=u64arg(&args[2])?;if nu>i32::MAX as u64{return Err("LIMIT");}let n=nu as i32;if r==1.0{finite(a*n as f64)}else{finite(a*(1.0-r.powi(n))/(1.0-r))}}
fn harmonic_number(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])?;if n>10_000_000{return Err("LIMIT");}let mut s=0.0;for k in 1..=n{s+=1.0/k as f64;}finite(s)}
fn triangular(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])? as u128;let x=n*(n+1)/2;if x>u64::MAX as u128{return Err("LIMIT");}Ok(json!(x as u64))}
fn square_pyramidal(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let n=u64arg(&args[0])? as u128;let x=n*(n+1)*(2*n+1)/6;if x>u64::MAX as u128{return Err("LIMIT");}Ok(json!(x as u64))}
