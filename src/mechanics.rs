use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("mech."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"mech.velocity"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.distance"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"mech.time"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.acceleration"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.force"=>{let v=nums(args,2)?;nonneg(v[0])?;finite(v[0]*v[1])},
"mech.mass_from_force"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.work"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"mech.power"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.kinetic_energy"=>{let v=nums(args,2)?;nonneg(v[0])?;finite(0.5*v[0]*v[1]*v[1])},
"mech.momentum"=>{let v=nums(args,2)?;nonneg(v[0])?;finite(v[0]*v[1])},
"mech.impulse"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"mech.torque"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"mech.angular_speed"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.rpm_to_rad_s"=>{let x=num1(args)?;finite(x*std::f64::consts::TAU/60.0)},
"mech.rad_s_to_rpm"=>{let x=num1(args)?;finite(x*60.0/std::f64::consts::TAU)},
"mech.rotational_energy"=>{let v=nums(args,2)?;nonneg(v[0])?;finite(0.5*v[0]*v[1]*v[1])},
"mech.angular_momentum"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"mech.centripetal_accel"=>{let v=nums(args,2)?;finite(v[0]*v[0]/positive(v[1])?)},
"mech.centripetal_force"=>{let v=nums(args,3)?;nonneg(v[0])?;finite(v[0]*v[1]*v[1]/positive(v[2])?)},
"mech.spring_force"=>{let v=nums(args,2)?;finite(-v[0]*v[1])},
"mech.spring_energy"=>{let v=nums(args,2)?;nonneg(v[0])?;finite(0.5*v[0]*v[1]*v[1])},
"mech.spring_period"=>{let v=nums(args,2)?;finite(std::f64::consts::TAU*(positive(v[0])?/positive(v[1])?).sqrt())},
"mech.pendulum_period"=>{let v=nums(args,2)?;finite(std::f64::consts::TAU*(positive(v[0])?/positive(v[1])?).sqrt())},
"mech.pressure"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"mech.stress"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"mech.strain"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.young_modulus"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.shear_modulus"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.bulk_modulus"=>{let v=nums(args,2)?;finite(-v[0]/nonzero(v[1])?)},
"mech.poisson_ratio"=>{let v=nums(args,2)?;finite(-v[0]/nonzero(v[1])?)},
"mech.mechanical_advantage"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.efficiency"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?*100.0)},
"mech.gear_ratio"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.output_rpm"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"mech.output_torque"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2])},
"mech.wheel_linear_speed"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"mech.rolling_rpm"=>{let v=nums(args,2)?;finite(v[0]/(std::f64::consts::TAU*positive(v[1])?)*60.0)},
"mech.friction_force"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"mech.incline_force"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2].sin())},
"mech.projectile_range"=>{let v=nums(args,3)?;finite(v[0]*v[0]*(2.0*v[1]).sin()/positive(v[2])?)},
_=>Err("OP")}}
fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
