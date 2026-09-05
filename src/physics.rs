use serde_json::{json, Value};

const G: f64 = 6.67430e-11;
const C: f64 = 299_792_458.0;
const H: f64 = 6.62607015e-34;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("phys."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "phys.kinetic_energy"=>{let(m,v)=two(args)?;domain_pos(m)?;finite(0.5*m*v*v)},
    "phys.potential_energy"=>{need(args,3)?;let m=num(&args[0])?;let g=num(&args[1])?;let h=num(&args[2])?;domain_pos(m)?;finite(m*g*h)},
    "phys.momentum"=>{let(m,v)=two(args)?;domain_pos(m)?;finite(m*v)},
    "phys.force"=>{let(m,a)=two(args)?;domain_pos(m)?;finite(m*a)},
    "phys.weight"=>{let(m,g)=two(args)?;domain_pos(m)?;finite(m*g)},
    "phys.work"=>{need(args,3)?;let f=num(&args[0])?;let d=num(&args[1])?;let angle=num(&args[2])?;finite(f*d*angle.cos())},
    "phys.power"=>{let(w,t)=two(args)?;if t==0.0{return Err("DIV0");}finite(w/t)},
    "phys.pressure"=>{let(f,a)=two(args)?;if a<=0.0{return Err("DOMAIN");}finite(f/a)},
    "phys.density"=>{let(m,v)=two(args)?;if v<=0.0{return Err("DOMAIN");}finite(m/v)},
    "phys.speed"=>{let(d,t)=two(args)?;if t==0.0{return Err("DIV0");}finite(d/t)},
    "phys.acceleration"=>{let(dv,t)=two(args)?;if t==0.0{return Err("DIV0");}finite(dv/t)},
    "phys.kinematic_v"=>{need(args,3)?;finite(num(&args[0])?+num(&args[1])?*num(&args[2])?)},
    "phys.kinematic_s"=>{need(args,3)?;let u=num(&args[0])?;let a=num(&args[1])?;let t=num(&args[2])?;finite(u*t+0.5*a*t*t)},
    "phys.kinematic_v2"=>{need(args,3)?;let u=num(&args[0])?;let a=num(&args[1])?;let s=num(&args[2])?;let q=u*u+2.0*a*s;if q<0.0{return Err("DOMAIN");}finite(q.sqrt())},
    "phys.projectile_time"=>projectile(args,"time"),
    "phys.projectile_range"=>projectile(args,"range"),
    "phys.projectile_max_height"=>projectile(args,"height"),
    "phys.centripetal_accel"=>{let(v,r)=two(args)?;if r<=0.0{return Err("DOMAIN");}finite(v*v/r)},
    "phys.centripetal_force"=>{need(args,3)?;let m=num(&args[0])?;let v=num(&args[1])?;let r=num(&args[2])?;if m<0.0||r<=0.0{return Err("DOMAIN");}finite(m*v*v/r)},
    "phys.grav_force"=>{need(args,3)?;let m1=num(&args[0])?;let m2=num(&args[1])?;let r=num(&args[2])?;if m1<0.0||m2<0.0||r<=0.0{return Err("DOMAIN");}finite(G*m1*m2/(r*r))},
    "phys.escape_velocity"=>{let(m,r)=two(args)?;if m<0.0||r<=0.0{return Err("DOMAIN");}finite((2.0*G*m/r).sqrt())},
    "phys.orbital_velocity"=>{let(m,r)=two(args)?;if m<0.0||r<=0.0{return Err("DOMAIN");}finite((G*m/r).sqrt())},
    "phys.orbital_period"=>{let(m,r)=two(args)?;if m<=0.0||r<=0.0{return Err("DOMAIN");}finite(std::f64::consts::TAU*(r.powi(3)/(G*m)).sqrt())},
    "phys.spring_energy"=>{let(k,x)=two(args)?;if k<0.0{return Err("DOMAIN");}finite(0.5*k*x*x)},
    "phys.hooke_force"=>{let(k,x)=two(args)?;if k<0.0{return Err("DOMAIN");}finite(-k*x)},
    "phys.frequency_from_period"=>{need(args,1)?;let t=num(&args[0])?;if t<=0.0{return Err("DOMAIN");}finite(1.0/t)},
    "phys.period_from_frequency"=>{need(args,1)?;let f=num(&args[0])?;if f<=0.0{return Err("DOMAIN");}finite(1.0/f)},
    "phys.wave_speed"=>{let(f,l)=two(args)?;finite(f*l)},
    "phys.photon_energy"=>{need(args,1)?;let f=num(&args[0])?;if f<0.0{return Err("DOMAIN");}finite(H*f)},
    "phys.mass_energy"=>{need(args,1)?;let m=num(&args[0])?;if m<0.0{return Err("DOMAIN");}finite(m*C*C)},
    "phys.de_broglie"=>{need(args,1)?;let p=num(&args[0])?;if p==0.0{return Err("DIV0");}finite(H/p.abs())},
    "phys.impulse"=>{let(f,t)=two(args)?;finite(f*t)},
    "phys.efficiency"=>{let(outp,inp)=two(args)?;if inp==0.0{return Err("DIV0");}finite(outp/inp*100.0)},
    "phys.torque"=>{need(args,3)?;let r=num(&args[0])?;let f=num(&args[1])?;let angle=num(&args[2])?;finite(r*f*angle.sin())},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{need(args,2)?;Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn domain_pos(x:f64)->Result<(),&'static str>{if x>=0.0{Ok(())}else{Err("DOMAIN")}}
fn projectile(args:&[Value],mode:&str)->Result<Value,&'static str>{if args.len()<2||args.len()>3{return Err("ARG");}let v=num(&args[0])?;let th=num(&args[1])?;let g=if args.len()==3{num(&args[2])?}else{9.80665};if v<0.0||g<=0.0{return Err("DOMAIN");}finite(match mode{"time"=>2.0*v*th.sin()/g,"range"=>v*v*(2.0*th).sin()/g,"height"=>v*v*th.sin().powi(2)/(2.0*g),_=>return Err("OP")})}
