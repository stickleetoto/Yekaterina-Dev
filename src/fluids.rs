use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("fluid."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"fluid.density"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"fluid.specific_weight"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"fluid.pressure_head"=>{let v=nums(args,3)?;finite(v[0]/(positive(v[1])?*positive(v[2])?))},
"fluid.hydrostatic_pressure"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2])},
"fluid.buoyant_force"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2])},
"fluid.flow_rate"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"fluid.velocity"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"fluid.area"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"fluid.mass_flow"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"fluid.reynolds"=>{let v=nums(args,4)?;finite(v[0]*v[1]*v[2]/positive(v[3])?)},
"fluid.kinematic_viscosity"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"fluid.dynamic_viscosity"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"fluid.dynamic_pressure"=>{let v=nums(args,2)?;finite(0.5*v[0]*v[1]*v[1])},
"fluid.bernoulli_pressure2"=>{let v=nums(args,7)?;finite(v[0]+0.5*v[1]*(v[2]*v[2]-v[3]*v[3])+v[1]*v[4]*(v[5]-v[6]))},
"fluid.torricelli_velocity"=>{let v=nums(args,2)?;if v[0]<0.0||v[1]<0.0{return Err("DOMAIN");}finite((2.0*v[0]*v[1]).sqrt())},
"fluid.continuity_velocity2"=>{let v=nums(args,3)?;finite(v[0]*v[1]/positive(v[2])?)},
"fluid.pipe_area"=>{let x=num1(args)?;nonneg(x)?;finite(std::f64::consts::PI*x*x/4.0)},
"fluid.hydraulic_diameter_rect"=>{let v=nums(args,2)?;nonneg(v[0])?;nonneg(v[1])?;finite(2.0*v[0]*v[1]/positive(v[0]+v[1])?)},
"fluid.darcy_loss"=>{let v=nums(args,5)?;finite(v[0]*v[1]/positive(v[2])?*v[3]*v[3]/(2.0*positive(v[4])?))},
"fluid.darcy_pressure_drop"=>{let v=nums(args,5)?;finite(v[0]*v[1]/positive(v[2])?*v[3]*v[4]*v[4]/2.0)},
"fluid.power_hydraulic"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"fluid.pump_power"=>{let v=nums(args,5)?;finite(v[0]*v[1]*v[2]*v[3]/positive(v[4])?)},
"fluid.weber"=>{let v=nums(args,4)?;finite(v[0]*v[1]*v[1]*v[2]/positive(v[3])?)},
"fluid.froude"=>{let v=nums(args,3)?;finite(v[0]/(positive(v[1])?*positive(v[2])?).sqrt())},
"fluid.mach"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"fluid.capillary_rise"=>{let v=nums(args,5)?;finite(2.0*v[0]*v[1].cos()/(positive(v[2])?*positive(v[3])?*positive(v[4])?))},
"fluid.stokes_drag"=>{let v=nums(args,3)?;finite(6.0*std::f64::consts::PI*v[0]*v[1]*v[2])},
"fluid.terminal_velocity_stokes"=>{let v=nums(args,5)?;finite(2.0*v[0]*v[0]*(v[1]-v[2])*v[3]/(9.0*positive(v[4])?))},
"fluid.poiseuille_flow"=>{let v=nums(args,4)?;finite(std::f64::consts::PI*v[1].powi(4)*v[0]/(8.0*positive(v[2])?*positive(v[3])?))},
"fluid.poiseuille_resistance"=>{let v=nums(args,3)?;finite(8.0*v[0]*v[1]/(std::f64::consts::PI*positive(v[2])?.powi(4)))},
"fluid.ideal_gas_density"=>{let v=nums(args,3)?;finite(v[0]*v[1]/(8.31446261815324*positive(v[2])?))},
"fluid.bulk_velocity"=>{let v=nums(args,3)?;finite(v[0]/(positive(v[1])?*positive(v[2])?))},
"fluid.orifice_flow"=>{let v=nums(args,4)?;finite(v[0]*v[1]*(2.0*v[2]*v[3]).sqrt())},
"fluid.specific_gravity"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"fluid.compressibility"=>{let x=num1(args)?;finite(1.0/positive(x)?)},
_=>Err("OP")}}
fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
