use serde_json::{json, Value};

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("info."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "info.shannon_entropy"=>entropy_op(args),
    "info.cross_entropy"=>cross_entropy(args),
    "info.kl_divergence"=>kl(args),
    "info.js_divergence"=>js(args),
    "info.perplexity"=>{let h=one(args)?;finite(2f64.powf(h))},
    "info.surprisal"=>{let p=one(args)?;prob_scalar(p)?;if p==0.0{return Err("DOMAIN");}finite(-p.log2())},
    "info.binary_entropy"=>{let p=one(args)?;if !(0.0..=1.0).contains(&p){return Err("DOMAIN");}finite(bin_entropy(p))},
    "info.uniform_entropy"=>{let n=one(args)?;if n<1.0{return Err("DOMAIN");}finite(n.log2())},
    "info.max_entropy"=>{let n=one(args)?;if n<1.0{return Err("DOMAIN");}finite(n.log2())},
    "info.redundancy"=>{let(h,m)=two(args)?;if m<=0.0||h<0.0||h>m{return Err("DOMAIN");}finite(1.0-h/m)},
    "info.hamming_string"=>hamming_string(args),
    "info.levenshtein"=>levenshtein_op(args),
    "info.jaccard_chars"=>jaccard_chars(args),
    "info.dice_chars"=>dice_chars(args),
    "info.compression_ratio"=>{let(o,c)=two(args)?;if o<=0.0||c<0.0{return Err("DOMAIN");}finite(c/o)},
    "info.savings_percent"=>{let(o,c)=two(args)?;if o<=0.0||c<0.0{return Err("DOMAIN");}finite((o-c)/o*100.0)},
    "info.code_rate"=>{let(k,n)=two(args)?;if k<0.0||n<=0.0||k>n{return Err("DOMAIN");}finite(k/n)},
    "info.channel_capacity_bsc"=>{let(b,p)=two(args)?;if b<0.0||!(0.0..=1.0).contains(&p){return Err("DOMAIN");}finite(b*(1.0-bin_entropy(p)))},
    "info.shannon_capacity"=>{let(b,snr)=two(args)?;if b<0.0||snr<0.0{return Err("DOMAIN");}finite(b*(1.0+snr).log2())},
    "info.snr_db_to_linear"=>finite(10f64.powf(one(args)?/10.0)),
    "info.snr_linear_to_db"=>{let x=one(args)?;if x<=0.0{return Err("DOMAIN");}finite(10.0*x.log10())},
    "info.nyquist_bitrate"=>{let(b,l)=two(args)?;if b<0.0||l<2.0{return Err("DOMAIN");}finite(2.0*b*l.log2())},
    "info.hartley_information"=>{let n=one(args)?;if n<1.0{return Err("DOMAIN");}finite(n.log2())},
    "info.self_information"=>{let p=one(args)?;if p<=0.0||p>1.0{return Err("DOMAIN");}finite(-p.log2())},
    "info.checksum8"=>checksum8(args),
    "info.xor_checksum"=>xor_checksum(args),
    "info.parity_even"=>parity(args,false),
    "info.parity_odd"=>parity(args,true),
    "info.efficiency"=>{let(h,l)=two(args)?;if h<0.0||l<=0.0||h>l{return Err("DOMAIN");}finite(h/l*100.0)},
    "info.symbols_for_bits"=>{let(bits,levels)=two(args)?;if bits<0.0||levels<2.0{return Err("DOMAIN");}finite((bits/levels.log2()).ceil())},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{need(args,2)?;Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn probs(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.is_empty()||a.len()>100_000{return Err("LIMIT");}let mut s=0.0;let mut out=Vec::with_capacity(a.len());for x in a{let p=num(x)?;if p<0.0||p>1.0{return Err("DOMAIN");}s+=p;out.push(p);}if (s-1.0).abs()>1e-9{return Err("DOMAIN");}Ok(out)}
fn prob_scalar(p:f64)->Result<(),&'static str>{if (0.0..=1.0).contains(&p){Ok(())}else{Err("DOMAIN")}}
fn bin_entropy(p:f64)->f64{if p==0.0||p==1.0{0.0}else{-p*p.log2()-(1.0-p)*(1.0-p).log2()}}
fn entropy(p:&[f64])->f64{p.iter().filter(|x|**x>0.0).map(|x|-x*x.log2()).sum()}
fn entropy_op(args:&[Value])->Result<Value,&'static str>{need(args,1)?;finite(entropy(&probs(&args[0])?))}
fn cross_entropy(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let p=probs(&args[0])?;let q=probs(&args[1])?;if p.len()!=q.len(){return Err("SHAPE");}let mut s=0.0;for(i,x)in p.iter().enumerate(){if *x==0.0{continue;}if q[i]==0.0{return Err("DOMAIN");}s-=x*q[i].log2();}finite(s)}
fn kl(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let p=probs(&args[0])?;let q=probs(&args[1])?;if p.len()!=q.len(){return Err("SHAPE");}let mut s=0.0;for(i,x)in p.iter().enumerate(){if *x==0.0{continue;}if q[i]==0.0{return Err("DOMAIN");}s+=x*(x/q[i]).log2();}finite(s)}
fn js(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let p=probs(&args[0])?;let q=probs(&args[1])?;if p.len()!=q.len(){return Err("SHAPE");}let m:Vec<f64>=p.iter().zip(&q).map(|(a,b)|(a+b)/2.0).collect();let d=|a:&[f64]|->f64{a.iter().zip(&m).filter(|(x,_)|**x>0.0).map(|(x,y)|x*(x/y).log2()).sum()};finite((d(&p)+d(&q))/2.0)}
fn strings(args:&[Value])->Result<(&str,&str),&'static str>{need(args,2)?;Ok((args[0].as_str().ok_or("TYPE")?,args[1].as_str().ok_or("TYPE")?))}
fn hamming_string(args:&[Value])->Result<Value,&'static str>{let(a,b)=strings(args)?;let ac:Vec<char>=a.chars().collect();let bc:Vec<char>=b.chars().collect();if ac.len()!=bc.len(){return Err("SHAPE");}Ok(json!(ac.iter().zip(&bc).filter(|(x,y)|x!=y).count()))}
fn levenshtein_op(args:&[Value])->Result<Value,&'static str>{let(a,b)=strings(args)?;if a.len()>100_000||b.len()>100_000{return Err("LIMIT");}let aa:Vec<char>=a.chars().collect();let bb:Vec<char>=b.chars().collect();if aa.len().saturating_mul(bb.len())>4_000_000{return Err("LIMIT");}let mut prev:Vec<usize>=(0..=bb.len()).collect();let mut cur=vec![0;bb.len()+1];for(i,ca)in aa.iter().enumerate(){cur[0]=i+1;for(j,cb)in bb.iter().enumerate(){cur[j+1]=(prev[j+1]+1).min(cur[j]+1).min(prev[j]+usize::from(ca!=cb));}std::mem::swap(&mut prev,&mut cur);}Ok(json!(prev[bb.len()]))}
fn set_chars(s:&str)->std::collections::BTreeSet<char>{s.chars().collect()}
fn jaccard_chars(args:&[Value])->Result<Value,&'static str>{let(a,b)=strings(args)?;let x=set_chars(a);let y=set_chars(b);let u=x.union(&y).count();if u==0{return finite(1.0);}finite(x.intersection(&y).count() as f64/u as f64)}
fn dice_chars(args:&[Value])->Result<Value,&'static str>{let(a,b)=strings(args)?;let x=set_chars(a);let y=set_chars(b);let d=x.len()+y.len();if d==0{return finite(1.0);}finite(2.0*x.intersection(&y).count() as f64/d as f64)}
fn bytes(args:&[Value])->Result<Vec<u8>,&'static str>{need(args,1)?;let a=args[0].as_array().ok_or("TYPE")?;if a.len()>1_000_000{return Err("LIMIT");}a.iter().map(|v|{let n=v.as_u64().ok_or("TYPE")?;u8::try_from(n).map_err(|_|"DOMAIN")}).collect()}
fn checksum8(args:&[Value])->Result<Value,&'static str>{let a=bytes(args)?;let s=a.iter().fold(0u8,|acc,x|acc.wrapping_add(*x));Ok(json!(s))}
fn xor_checksum(args:&[Value])->Result<Value,&'static str>{let a=bytes(args)?;Ok(json!(a.iter().fold(0u8,|acc,x|acc^x)))}
fn parity(args:&[Value],odd:bool)->Result<Value,&'static str>{need(args,1)?;let n=args[0].as_u64().ok_or("TYPE")?;let ones=n.count_ones()%2==1;Ok(json!(if odd{ones}else{!ones}))}
