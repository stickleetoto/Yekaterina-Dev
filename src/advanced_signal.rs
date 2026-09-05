use serde_json::{json, Value};

#[derive(Clone,Copy)]struct C{re:f64,im:f64}
impl C{fn add(self,o:C)->C{C{re:self.re+o.re,im:self.im+o.im}}fn sub(self,o:C)->C{C{re:self.re-o.re,im:self.im-o.im}}fn mul(self,o:C)->C{C{re:self.re*o.re-self.im*o.im,im:self.re*o.im+self.im*o.re}}}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    if matches!(op,
        "signal.dft"|"signal.idft"|"signal.fft"|"signal.ifft"|"signal.rfft"|
        "signal.magnitude_spectrum"|"signal.power_spectrum"|"signal.phase_spectrum"|
        "signal.window_hann"|"signal.window_hamming"|"signal.window_blackman"|"signal.window_bartlett"|
        "signal.apply_hann"|"signal.apply_hamming"|"signal.apply_blackman"|"signal.apply_bartlett"|
        "signal.rms"|"signal.mean"|"signal.variance"|"signal.std"|"signal.snr"|
        "signal.db20"|"signal.db10"|"signal.from_db20"|"signal.from_db10"|
        "signal.peak_to_peak"|"signal.crest_factor"|"signal.zcr_rate"|"signal.pad_zero"|
        "signal.resample_linear"|"signal.reverse"|"signal.rectify"|"signal.clip"|"signal.dc_remove"
    ){Some(run(op,args))}else{None}
}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "signal.dft"=>dft_op(args,false),
    "signal.idft"=>idft_op(args),
    "signal.fft"=>fft_op(args,false),
    "signal.ifft"=>fft_op(args,true),
    "signal.rfft"=>rfft(args),
    "signal.magnitude_spectrum"=>spectrum(args,"mag"),
    "signal.power_spectrum"=>spectrum(args,"power"),
    "signal.phase_spectrum"=>spectrum(args,"phase"),
    "signal.window_hann"=>window(args,"hann"),
    "signal.window_hamming"=>window(args,"hamming"),
    "signal.window_blackman"=>window(args,"blackman"),
    "signal.window_bartlett"=>window(args,"bartlett"),
    "signal.apply_hann"=>apply_window(args,"hann"),
    "signal.apply_hamming"=>apply_window(args,"hamming"),
    "signal.apply_blackman"=>apply_window(args,"blackman"),
    "signal.apply_bartlett"=>apply_window(args,"bartlett"),
    "signal.rms"=>{let x=one_real(args)?;if x.is_empty(){return Err("EMPTY");}finite((x.iter().map(|v|v*v).sum::<f64>()/x.len() as f64).sqrt())},
    "signal.mean"=>{let x=one_real(args)?;if x.is_empty(){return Err("EMPTY");}finite(x.iter().sum::<f64>()/x.len() as f64)},
    "signal.variance"=>sig_variance(args,false),
    "signal.std"=>sig_variance(args,true),
    "signal.snr"=>snr(args),
    "signal.db20"=>{need(args,1)?;let x=num(&args[0])?;if x<=0.0{return Err("DOMAIN");}finite(20.0*x.log10())},
    "signal.db10"=>{need(args,1)?;let x=num(&args[0])?;if x<=0.0{return Err("DOMAIN");}finite(10.0*x.log10())},
    "signal.from_db20"=>{need(args,1)?;finite(10f64.powf(num(&args[0])?/20.0))},
    "signal.from_db10"=>{need(args,1)?;finite(10f64.powf(num(&args[0])?/10.0))},
    "signal.peak_to_peak"=>{let x=one_real(args)?;if x.is_empty(){return Err("EMPTY");}finite(x.iter().copied().reduce(f64::max).unwrap()-x.iter().copied().reduce(f64::min).unwrap())},
    "signal.crest_factor"=>crest(args),
    "signal.zcr_rate"=>zcr_rate(args),
    "signal.pad_zero"=>pad_zero(args),
    "signal.resample_linear"=>resample(args),
    "signal.reverse"=>{let mut x=one_real(args)?;x.reverse();Ok(json!(x))},
    "signal.rectify"=>{let x=one_real(args)?;Ok(json!(x.into_iter().map(f64::abs).collect::<Vec<_>>()))},
    "signal.clip"=>clip(args),
    "signal.dc_remove"=>dc_remove(args),
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn real_vec(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>4096{return Err("LIMIT");}a.iter().map(num).collect()}
fn one_real(args:&[Value])->Result<Vec<f64>,&'static str>{need(args,1)?;real_vec(&args[0])}
fn complex_vec(v:&Value)->Result<Vec<C>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>4096{return Err("LIMIT");}a.iter().map(|z|{let q=z.as_array().ok_or("TYPE")?;if q.len()!=2{return Err("SHAPE");}Ok(C{re:num(&q[0])?,im:num(&q[1])?})}).collect()}
fn out_c(v:Vec<C>)->Result<Value,&'static str>{if v.iter().all(|z|z.re.is_finite()&&z.im.is_finite()){Ok(json!(v.into_iter().map(|z|vec![z.re,z.im]).collect::<Vec<_>>()))}else{Err("NONFINITE")}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn dft(x:&[C],inverse:bool)->Vec<C>{let n=x.len();let sign=if inverse{1.0}else{-1.0};let mut out=vec![C{re:0.0,im:0.0};n];for k in 0..n{let mut s=C{re:0.0,im:0.0};for(t,z)in x.iter().enumerate(){let ang=sign*std::f64::consts::TAU*(k*t) as f64/n as f64;let w=C{re:ang.cos(),im:ang.sin()};s=s.add(z.mul(w));}if inverse{s.re/=n as f64;s.im/=n as f64;}out[k]=s;}out}
fn fft_inplace(a:&mut[C],inverse:bool){let n=a.len();let mut j=0usize;for i in 1..n{let mut bit=n>>1;while j&bit!=0{j^=bit;bit>>=1;}j^=bit;if i<j{a.swap(i,j);}}let mut len=2usize;while len<=n{let ang=(if inverse{1.0}else{-1.0})*std::f64::consts::TAU/len as f64;let wlen=C{re:ang.cos(),im:ang.sin()};for i in (0..n).step_by(len){let mut w=C{re:1.0,im:0.0};for j in 0..len/2{let u=a[i+j];let v=a[i+j+len/2].mul(w);a[i+j]=u.add(v);a[i+j+len/2]=u.sub(v);w=w.mul(wlen);}}len<<=1;}if inverse{for z in a{z.re/=n as f64;z.im/=n as f64;}}}
fn dft_op(args:&[Value],inverse:bool)->Result<Value,&'static str>{need(args,1)?;let x=if inverse{complex_vec(&args[0])?}else{real_vec(&args[0])?.into_iter().map(|v|C{re:v,im:0.0}).collect()};if x.is_empty(){return Err("EMPTY");}if x.len()>2048{return Err("LIMIT");}out_c(dft(&x,inverse))}
fn idft_op(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let x=complex_vec(&args[0])?;if x.is_empty(){return Err("EMPTY");}if x.len()>2048{return Err("LIMIT");}out_c(dft(&x,true))}
fn fft_op(args:&[Value],inverse:bool)->Result<Value,&'static str>{need(args,1)?;let mut x=if inverse{complex_vec(&args[0])?}else{real_vec(&args[0])?.into_iter().map(|v|C{re:v,im:0.0}).collect()};if x.is_empty(){return Err("EMPTY");}if !x.len().is_power_of_two(){return Err("SHAPE");}fft_inplace(&mut x,inverse);out_c(x)}
fn rfft(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let mut x=real_vec(&args[0])?.into_iter().map(|v|C{re:v,im:0.0}).collect::<Vec<_>>();if x.is_empty()||!x.len().is_power_of_two(){return Err("SHAPE");}fft_inplace(&mut x,false);let keep=x.len()/2+1;x.truncate(keep);out_c(x)}
fn spectrum(args:&[Value],mode:&str)->Result<Value,&'static str>{need(args,1)?;let x=complex_vec(&args[0])?;let out=x.into_iter().map(|z|match mode{"mag"=>z.re.hypot(z.im),"power"=>z.re*z.re+z.im*z.im,"phase"=>z.im.atan2(z.re),_=>0.0}).collect::<Vec<_>>();if out.iter().all(|v|v.is_finite()){Ok(json!(out))}else{Err("NONFINITE")}}
fn window_value(kind:&str,i:usize,n:usize)->f64{if n<=1{return 1.0;}let x=i as f64/(n-1) as f64;match kind{"hann"=>0.5-0.5*(std::f64::consts::TAU*x).cos(),"hamming"=>0.54-0.46*(std::f64::consts::TAU*x).cos(),"blackman"=>0.42-0.5*(std::f64::consts::TAU*x).cos()+0.08*(2.0*std::f64::consts::TAU*x).cos(),"bartlett"=>1.0-(2.0*(i as f64-(n-1) as f64/2.0)/(n-1) as f64).abs(),_=>1.0}}
fn window(args:&[Value],kind:&str)->Result<Value,&'static str>{need(args,1)?;let n=args[0].as_u64().ok_or("TYPE")? as usize;if n==0||n>100_000{return Err("LIMIT");}Ok(json!((0..n).map(|i|window_value(kind,i,n)).collect::<Vec<_>>()))}
fn apply_window(args:&[Value],kind:&str)->Result<Value,&'static str>{need(args,1)?;let x=real_vec(&args[0])?;let n=x.len();Ok(json!(x.into_iter().enumerate().map(|(i,v)|v*window_value(kind,i,n)).collect::<Vec<_>>()))}
fn sig_variance(args:&[Value],std:bool)->Result<Value,&'static str>{let x=one_real(args)?;if x.is_empty(){return Err("EMPTY");}let m=x.iter().sum::<f64>()/x.len() as f64;let v=x.iter().map(|a|{let d=*a-m;d*d}).sum::<f64>()/x.len() as f64;finite(if std{v.sqrt()}else{v})}
fn snr(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let s=real_vec(&args[0])?;let n=real_vec(&args[1])?;if s.is_empty()||s.len()!=n.len(){return Err("SHAPE");}let ps=s.iter().map(|x|x*x).sum::<f64>()/s.len() as f64;let pn=n.iter().map(|x|x*x).sum::<f64>()/n.len() as f64;if pn==0.0{return Err("DIV0");}if ps==0.0{return Err("DOMAIN");}finite(10.0*(ps/pn).log10())}
fn crest(args:&[Value])->Result<Value,&'static str>{let x=one_real(args)?;if x.is_empty(){return Err("EMPTY");}let peak=x.iter().map(|v|v.abs()).fold(0.0,f64::max);let rms=(x.iter().map(|v|v*v).sum::<f64>()/x.len() as f64).sqrt();if rms==0.0{return Err("DOMAIN");}finite(peak/rms)}
fn zcr_rate(args:&[Value])->Result<Value,&'static str>{let x=one_real(args)?;if x.len()<2{return Ok(json!(0.0));}let mut n=0usize;for w in x.windows(2){if (w[0]<0.0&&w[1]>=0.0)||(w[0]>0.0&&w[1]<=0.0){n+=1;}}finite(n as f64/(x.len()-1) as f64)}
fn pad_zero(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let mut x=real_vec(&args[0])?;let n=args[1].as_u64().ok_or("TYPE")? as usize;if n<x.len()||n>100_000{return Err("DOMAIN");}x.resize(n,0.0);Ok(json!(x))}
fn resample(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let x=real_vec(&args[0])?;let n=args[1].as_u64().ok_or("TYPE")? as usize;if x.is_empty()||n==0||n>100_000{return Err("DOMAIN");}if n==1{return Ok(json!([x[0]]));}if x.len()==1{return Ok(json!(vec![x[0];n]));}let scale=(x.len()-1) as f64/(n-1) as f64;let mut o=Vec::with_capacity(n);for i in 0..n{let p=i as f64*scale;let lo=p.floor() as usize;let hi=p.ceil() as usize;let t=p-lo as f64;o.push(x[lo]+(x[hi]-x[lo])*t);}Ok(json!(o))}
fn clip(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let x=real_vec(&args[0])?;let lo=num(&args[1])?;let hi=num(&args[2])?;if lo>hi{return Err("DOMAIN");}Ok(json!(x.into_iter().map(|v|v.clamp(lo,hi)).collect::<Vec<_>>()))}
fn dc_remove(args:&[Value])->Result<Value,&'static str>{let x=one_real(args)?;if x.is_empty(){return Err("EMPTY");}let m=x.iter().sum::<f64>()/x.len() as f64;Ok(json!(x.into_iter().map(|v|v-m).collect::<Vec<_>>()))}
