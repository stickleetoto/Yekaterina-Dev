use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

const K:f64=8.9875517923e9;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("elec."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"elec.voltage"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"elec.current"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"elec.resistance"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"elec.power_vi"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"elec.power_i2r"=>{let v=nums(args,2)?;finite(v[0]*v[0]*v[1])},
"elec.power_v2r"=>{let v=nums(args,2)?;finite(v[0]*v[0]/positive(v[1])?)},
"elec.energy"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"elec.charge"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"elec.current_from_charge"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"elec.capacitance"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"elec.charge_capacitor"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"elec.capacitor_energy"=>{let v=nums(args,2)?;finite(0.5*v[0]*v[1]*v[1])},
"elec.inductor_energy"=>{let v=nums(args,2)?;finite(0.5*v[0]*v[1]*v[1])},
"elec.rc_tau"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"elec.rl_tau"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"elec.xc"=>{let v=nums(args,2)?;finite(1.0/(std::f64::consts::TAU*positive(v[0])?*positive(v[1])?))},
"elec.xl"=>{let v=nums(args,2)?;finite(std::f64::consts::TAU*v[0]*v[1])},
"elec.impedance_rl"=>{let v=nums(args,2)?;finite(v[0].hypot(v[1]))},
"elec.impedance_rc"=>{let v=nums(args,2)?;finite(v[0].hypot(v[1]))},
"elec.resonant_frequency"=>{let v=nums(args,2)?;finite(1.0/(std::f64::consts::TAU*(positive(v[0])?*positive(v[1])?).sqrt()))},
"elec.conductance"=>{let x=num1(args)?;finite(1.0/nonzero(x)?)},
"elec.resistivity"=>{let v=nums(args,3)?;finite(v[0]*v[1]/positive(v[2])?)},
"elec.resistance_from_resistivity"=>{let v=nums(args,3)?;finite(v[0]*v[1]/positive(v[2])?)},
"elec.wire_area"=>{let x=num1(args)?;nonneg(x)?;finite(std::f64::consts::PI*x*x/4.0)},
"elec.current_density"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"elec.electric_field"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"elec.coulomb_force"=>{let v=nums(args,3)?;finite(K*v[0]*v[1]/positive(v[2]*v[2])?)},
"elec.electric_potential_point"=>{let v=nums(args,2)?;finite(K*v[0]/positive(v[1])?)},
"elec.electric_field_point"=>{let v=nums(args,2)?;finite(K*v[0]/positive(v[1]*v[1])?)},
"elec.parallel_two"=>{let v=nums(args,2)?;finite(v[0]*v[1]/positive(v[0]+v[1])?)},
"elec.series_two"=>{let v=nums(args,2)?;finite(v[0]+v[1])},
"elec.voltage_divider"=>{let v=nums(args,3)?;finite(v[0]*v[2]/positive(v[1]+v[2])?)},
"elec.current_divider_r1"=>{let v=nums(args,3)?;finite(v[0]*v[2]/positive(v[1]+v[2])?)},
"elec.transformer_voltage"=>{let v=nums(args,3)?;finite(v[0]*v[2]/positive(v[1])?)},
"elec.transformer_current"=>{let v=nums(args,3)?;finite(v[0]*v[1]/positive(v[2])?)},
"elec.transformer_impedance"=>{let v=nums(args,3)?;finite(v[0]*(v[2]/positive(v[1])?).powi(2))},
"elec.battery_runtime_hours"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"elec.battery_energy_wh"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"elec.db_power_ratio"=>{let x=num1(args)?;finite(10.0*positive(x)?.log10())},
"elec.db_voltage_ratio"=>{let x=num1(args)?;finite(20.0*positive(x)?.log10())},
_=>Err("OP")}}
fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
