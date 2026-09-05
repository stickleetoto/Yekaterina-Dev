use serde_json::{json, Value};

#[derive(Clone, Copy, Debug)]
struct C { re: f64, im: f64 }
impl C {
    fn add(self,o:C)->C{C{re:self.re+o.re,im:self.im+o.im}}
    fn sub(self,o:C)->C{C{re:self.re-o.re,im:self.im-o.im}}
    fn mul(self,o:C)->C{C{re:self.re*o.re-self.im*o.im,im:self.re*o.im+self.im*o.re}}
    fn div(self,o:C)->Result<C,&'static str>{let d=o.re*o.re+o.im*o.im;if d==0.0{return Err("DIV0");}Ok(C{re:(self.re*o.re+self.im*o.im)/d,im:(self.im*o.re-self.re*o.im)/d})}
    fn conj(self)->C{C{re:self.re,im:-self.im}}
    fn abs(self)->f64{self.re.hypot(self.im)}
    fn arg(self)->f64{self.im.atan2(self.re)}
    fn exp(self)->C{let e=self.re.exp();C{re:e*self.im.cos(),im:e*self.im.sin()}}
    fn ln(self)->Result<C,&'static str>{let r=self.abs();if r==0.0{return Err("DOMAIN");}Ok(C{re:r.ln(),im:self.arg()})}
    fn sqrt(self)->C{let r=self.abs();let re=((r+self.re)/2.0).max(0.0).sqrt();let im=((r-self.re)/2.0).max(0.0).sqrt().copysign(self.im);C{re,im}}
    fn sin(self)->C{C{re:self.re.sin()*self.im.cosh(),im:self.re.cos()*self.im.sinh()}}
    fn cos(self)->C{C{re:self.re.cos()*self.im.cosh(),im:-self.re.sin()*self.im.sinh()}}
    fn sinh(self)->C{C{re:self.re.sinh()*self.im.cos(),im:self.re.cosh()*self.im.sin()}}
    fn cosh(self)->C{C{re:self.re.cosh()*self.im.cos(),im:self.re.sinh()*self.im.sin()}}
}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("cplx."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "cplx.add"=>binary(args,|a,b|Ok(a.add(b))),
    "cplx.sub"=>binary(args,|a,b|Ok(a.sub(b))),
    "cplx.mul"=>binary(args,|a,b|Ok(a.mul(b))),
    "cplx.div"=>binary(args,|a,b|a.div(b)),
    "cplx.conj"=>unary(args,|a|Ok(a.conj())),
    "cplx.abs"=>{let a=one(args)?;finite(a.abs())},
    "cplx.norm_sq"=>{let a=one(args)?;finite(a.re*a.re+a.im*a.im)},
    "cplx.arg"=>{let a=one(args)?;finite(a.arg())},
    "cplx.phase_deg"=>{let a=one(args)?;finite(a.arg().to_degrees())},
    "cplx.from_polar"=>from_polar(args,false),
    "cplx.from_polar_deg"=>from_polar(args,true),
    "cplx.to_polar"=>{let a=one(args)?;Ok(json!([a.abs(),a.arg()]))},
    "cplx.to_polar_deg"=>{let a=one(args)?;Ok(json!([a.abs(),a.arg().to_degrees()]))},
    "cplx.recip"=>unary(args,|a|C{re:1.0,im:0.0}.div(a)),
    "cplx.exp"=>unary(args,|a|Ok(a.exp())),
    "cplx.ln"=>unary(args,|a|a.ln()),
    "cplx.sqrt"=>unary(args,|a|Ok(a.sqrt())),
    "cplx.pow_int"=>pow_int(args),
    "cplx.pow_real"=>pow_real(args),
    "cplx.sin"=>unary(args,|a|Ok(a.sin())),
    "cplx.cos"=>unary(args,|a|Ok(a.cos())),
    "cplx.tan"=>unary(args,|a|a.sin().div(a.cos())),
    "cplx.sinh"=>unary(args,|a|Ok(a.sinh())),
    "cplx.cosh"=>unary(args,|a|Ok(a.cosh())),
    "cplx.tanh"=>unary(args,|a|a.sinh().div(a.cosh())),
    "cplx.scale"=>scale(args),
    "cplx.rotate"=>rotate(args,false),
    "cplx.rotate_deg"=>rotate(args,true),
    "cplx.normalize"=>{let a=one(args)?;let r=a.abs();if r==0.0{Err("DOMAIN")}else{out(C{re:a.re/r,im:a.im/r})}},
    "cplx.approx_eq"=>approx_eq(args),
    "cplx.is_real"=>{let a=one(args)?;Ok(json!(a.im==0.0))},
    "cplx.is_imag"=>{let a=one(args)?;Ok(json!(a.re==0.0&&a.im!=0.0))},
    "cplx.from_real"=>{need(args,1)?;out(C{re:num(&args[0])?,im:0.0})},
    "cplx.from_imag"=>{need(args,1)?;out(C{re:0.0,im:num(&args[0])?})},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn c(v:&Value)->Result<C,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=2{return Err("SHAPE");}Ok(C{re:num(&a[0])?,im:num(&a[1])?})}
fn one(args:&[Value])->Result<C,&'static str>{need(args,1)?;c(&args[0])}
fn out(z:C)->Result<Value,&'static str>{if z.re.is_finite()&&z.im.is_finite(){Ok(json!([z.re,z.im]))}else{Err("NONFINITE")}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn unary<F>(args:&[Value],f:F)->Result<Value,&'static str>where F:FnOnce(C)->Result<C,&'static str>{out(f(one(args)?)?)}
fn binary<F>(args:&[Value],f:F)->Result<Value,&'static str>where F:FnOnce(C,C)->Result<C,&'static str>{need(args,2)?;out(f(c(&args[0])?,c(&args[1])?)?)}
fn from_polar(args:&[Value],deg:bool)->Result<Value,&'static str>{need(args,2)?;let r=num(&args[0])?;let mut t=num(&args[1])?;if r<0.0{return Err("DOMAIN");}if deg{t=t.to_radians();}out(C{re:r*t.cos(),im:r*t.sin()})}
fn pow_int(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let mut base=c(&args[0])?;let n=args[1].as_i64().ok_or("TYPE")?;if n==0{return out(C{re:1.0,im:0.0});}let neg=n<0;let mut e=n.unsigned_abs();let mut r=C{re:1.0,im:0.0};while e>0{if e&1==1{r=r.mul(base);}base=base.mul(base);e>>=1;}if neg{r=C{re:1.0,im:0.0}.div(r)?;}out(r)}
fn pow_real(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let z=c(&args[0])?;let p=num(&args[1])?;let l=z.ln()?;out(C{re:l.re*p,im:l.im*p}.exp())}
fn scale(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let z=c(&args[0])?;let s=num(&args[1])?;out(C{re:z.re*s,im:z.im*s})}
fn rotate(args:&[Value],deg:bool)->Result<Value,&'static str>{need(args,2)?;let z=c(&args[0])?;let mut t=num(&args[1])?;if deg{t=t.to_radians();}out(z.mul(C{re:t.cos(),im:t.sin()}))}
fn approx_eq(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let a=c(&args[0])?;let b=c(&args[1])?;let e=num(&args[2])?;if e<0.0{return Err("DOMAIN");}Ok(json!((a.sub(b)).abs()<=e))}
