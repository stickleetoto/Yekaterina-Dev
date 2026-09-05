use num_bigint::{BigInt, BigUint};
use num_traits::{One, Zero};
use serde_json::{json, Value};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if op.starts_with("disc.") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "disc.ncr" => { let (n,k)=two_u64(args)?; exact(ncr(n,k)?) }
        "disc.npr" => { let (n,k)=two_u64(args)?; exact(npr(n,k)?) }
        "disc.multiset" => { let (n,k)=two_u64(args)?; if n==0 && k>0{return Err("DOMAIN");} exact(ncr(n.saturating_add(k).saturating_sub(1),k)?) }
        "disc.derangements" => { let n=one_u64(args)?; exact(derangements(n)?) }
        "disc.stirling2" => { let (n,k)=two_u64(args)?; exact(stirling2(n,k)?) }
        "disc.bell" => { let n=one_u64(args)?; exact(bell(n)?) }
        "disc.lah" => { let (n,k)=two_u64(args)?; exact(lah(n,k)?) }
        "disc.compositions" => { let (n,k)=two_u64(args)?; if n==0||k==0||k>n{return Err("DOMAIN");} exact(ncr(n-1,k-1)?) }
        "disc.weak_compositions" => { let (n,k)=two_u64(args)?; if k==0{return Err("DOMAIN");} exact(ncr(n+k-1,k-1)?) }
        "disc.handshake" => { let n=one_u64(args)?; exact(BigUint::from(n)*BigUint::from(n.saturating_sub(1))/2u32) }
        "disc.power_set_count" => { let n=one_u64(args)?; if n>100_000{return Err("LIMIT");} exact(BigUint::one()<<usize::try_from(n).map_err(|_|"LIMIT")?) }
        "disc.surjections" => { let (n,k)=two_u64(args)?; exact_signed(surjections(n,k)?) }
        "disc.circular_permutations" => { let n=one_u64(args)?; if n==0{return Err("DOMAIN");} exact(factorial(n-1)?) }
        "disc.multinomial" => multinomial(args),
        "disc.central_binomial" => { let n=one_u64(args)?; exact(ncr(n.checked_mul(2).ok_or("LIMIT")?,n)?) }
        "disc.lattice_paths" => { let (x,y)=two_u64(args)?; exact(ncr(x.checked_add(y).ok_or("LIMIT")?,x)?) }
        "disc.tribonacci" => { let n=one_u64(args)?; exact(linear3(n,0,0,1)?) }
        "disc.lucas" => { let n=one_u64(args)?; exact(linear2(n,2,1)?) }
        "disc.pell" => { let n=one_u64(args)?; exact(pell(n)?) }
        "disc.jacobsthal" => { let n=one_u64(args)?; exact(jacobsthal(n)?) }
        "disc.triangular" => { let n=one_u64(args)?; exact(BigUint::from(n)*BigUint::from(n+1)/2u32) }
        "disc.tetrahedral" => { let n=one_u64(args)?; exact(BigUint::from(n)*BigUint::from(n+1)*BigUint::from(n+2)/6u32) }
        "disc.pentagonal" => { let n=one_u64(args)?; exact(BigUint::from(n)*BigUint::from(3*n-1)/2u32) }
        "disc.hexagonal" => { let n=one_u64(args)?; exact(BigUint::from(n)*BigUint::from(2*n-1)) }
        "disc.sum_n" => { let n=one_u64(args)?; exact(BigUint::from(n)*BigUint::from(n+1)/2u32) }
        "disc.sum_squares" => { let n=one_u64(args)?; exact(BigUint::from(n)*BigUint::from(n+1)*BigUint::from(2*n+1)/6u32) }
        "disc.sum_cubes" => { let n=one_u64(args)?; let t=BigUint::from(n)*BigUint::from(n+1)/2u32; exact(&t*&t) }
        "disc.collatz_steps" => collatz(args),
        "disc.digit_sum" => digit_sum(args),
        "disc.digital_root" => digital_root(args),
        _ => Err("OP"),
    }
}

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn one_u64(args:&[Value])->Result<u64,&'static str>{need(args,1)?; args[0].as_u64().ok_or("TYPE")}
fn two_u64(args:&[Value])->Result<(u64,u64),&'static str>{need(args,2)?;Ok((args[0].as_u64().ok_or("TYPE")?,args[1].as_u64().ok_or("TYPE")?))}
fn exact(x:BigUint)->Result<Value,&'static str>{Ok(json!(x.to_string()))}
fn exact_signed(x:BigInt)->Result<Value,&'static str>{Ok(json!(x.to_string()))}

fn factorial(n:u64)->Result<BigUint,&'static str>{if n>10_000{return Err("LIMIT");}let mut r=BigUint::one();for i in 2..=n{r*=i;}Ok(r)}
fn ncr(n:u64,k:u64)->Result<BigUint,&'static str>{if k>n{return Err("DOMAIN");}if n>100_000{return Err("LIMIT");}let k=k.min(n-k);let mut r=BigUint::one();for i in 0..k{r*=n-i;r/=i+1;}Ok(r)}
fn npr(n:u64,k:u64)->Result<BigUint,&'static str>{if k>n{return Err("DOMAIN");}if k>10_000{return Err("LIMIT");}let mut r=BigUint::one();for i in 0..k{r*=n-i;}Ok(r)}
fn derangements(n:u64)->Result<BigUint,&'static str>{if n>10_000{return Err("LIMIT");}if n==0{return Ok(BigUint::one());}let mut a=BigUint::one();let mut b=BigUint::zero();for i in 2..=n{let c=BigUint::from(i-1)*(&a+&b);a=b;b=c;}Ok(b)}
fn stirling2(n:u64,k:u64)->Result<BigUint,&'static str>{if k>n{return Ok(BigUint::zero());}if n>2_000{return Err("LIMIT");}let k_us=usize::try_from(k).map_err(|_|"LIMIT")?;let mut dp=vec![BigUint::zero();k_us+1];dp[0]=BigUint::one();for i in 1..=n{let maxj=k.min(i);for j in (1..=maxj).rev(){let u=usize::try_from(j).map_err(|_|"LIMIT")?;dp[u]=&dp[u-1]+BigUint::from(j)*&dp[u];}dp[0]=BigUint::zero();}Ok(dp[k_us].clone())}
fn bell(n:u64)->Result<BigUint,&'static str>{if n>300{return Err("LIMIT");}let mut sum=BigUint::zero();for k in 0..=n{sum+=stirling2(n,k)?;}Ok(sum)}
fn lah(n:u64,k:u64)->Result<BigUint,&'static str>{if k==0{return Ok(if n==0{BigUint::one()}else{BigUint::zero()});}if k>n{return Ok(BigUint::zero());}let c=ncr(n-1,k-1)?;Ok(c*factorial(n)?/factorial(k)?)}
fn surjections(n:u64,k:u64)->Result<BigInt,&'static str>{if n>500||k>500{return Err("LIMIT");}if k==0{return Ok(if n==0{BigInt::one()}else{BigInt::zero()});}let mut s=BigInt::zero();for i in 0..=k{let c=BigInt::from(ncr(k,i)?);let p=BigInt::from(BigUint::from(k-i).pow(u32::try_from(n).map_err(|_|"LIMIT")?));if i%2==0{s+=c*p}else{s-=c*p}}Ok(s)}
fn multinomial(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let a=args[0].as_array().ok_or("TYPE")?;if a.is_empty()||a.len()>1_000{return Err("LIMIT");}let mut total=0u64;let mut denom=BigUint::one();for v in a{let n=v.as_u64().ok_or("TYPE")?;total=total.checked_add(n).ok_or("LIMIT")?;denom*=factorial(n)?;}let out=factorial(total)?/denom;exact(out)}
fn linear2(n:u64,a0:u64,a1:u64)->Result<BigUint,&'static str>{if n>100_000{return Err("LIMIT");}if n==0{return Ok(BigUint::from(a0));}let mut a=BigUint::from(a0);let mut b=BigUint::from(a1);for _ in 1..n{let c=&a+&b;a=b;b=c;}Ok(b)}
fn linear3(n:u64,a0:u64,a1:u64,a2:u64)->Result<BigUint,&'static str>{if n>50_000{return Err("LIMIT");}match n{0=>Ok(BigUint::from(a0)),1=>Ok(BigUint::from(a1)),2=>Ok(BigUint::from(a2)),_=>{let(mut a,mut b,mut c)=(BigUint::from(a0),BigUint::from(a1),BigUint::from(a2));for _ in 3..=n{let d=&a+&b+&c;a=b;b=c;c=d;}Ok(c)}}}
fn pell(n:u64)->Result<BigUint,&'static str>{if n>100_000{return Err("LIMIT");}if n==0{return Ok(BigUint::zero());}let(mut a,mut b)=(BigUint::zero(),BigUint::one());for _ in 1..n{let c=&a+BigUint::from(2u32)*&b;a=b;b=c;}Ok(b)}
fn jacobsthal(n:u64)->Result<BigUint,&'static str>{if n>100_000{return Err("LIMIT");}if n==0{return Ok(BigUint::zero());}let(mut a,mut b)=(BigUint::zero(),BigUint::one());for _ in 1..n{let c=BigUint::from(2u32)*&a+&b;a=b;b=c;}Ok(b)}
fn collatz(args:&[Value])->Result<Value,&'static str>{let mut n=one_u64(args)?;if n==0{return Err("DOMAIN");}let mut steps=0u64;while n!=1{if steps>=10_000_000{return Err("LIMIT");}n=if n%2==0{n/2}else{n.checked_mul(3).and_then(|x|x.checked_add(1)).ok_or("LIMIT")?};steps+=1;}Ok(json!(steps))}
fn digits(args:&[Value])->Result<String,&'static str>{need(args,1)?;let s=if let Some(s)=args[0].as_str(){s.to_string()}else if let Some(n)=args[0].as_u64(){n.to_string()}else{return Err("TYPE")};let s=s.trim_start_matches('+');if s.is_empty()||s.len()>1_000_000||!s.bytes().all(|b|b.is_ascii_digit()){return Err("TYPE");}Ok(s.to_string())}
fn digit_sum(args:&[Value])->Result<Value,&'static str>{let s=digits(args)?;let sum:u64=s.bytes().map(|b|u64::from(b-b'0')).sum();Ok(json!(sum))}
fn digital_root(args:&[Value])->Result<Value,&'static str>{let s=digits(args)?;if s.bytes().all(|b|b==b'0'){return Ok(json!(0));}let sum:u64=s.bytes().map(|b|u64::from(b-b'0')).sum();Ok(json!(1+(sum-1)%9))}
