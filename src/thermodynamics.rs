use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

const R:f64=8.31446261815324; const SIGMA:f64=5.670374419e-8;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("thermo."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"thermo.heat"=>{let v=nums(args,3)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*v[1]*v[2])},
"thermo.specific_heat"=>{let v=nums(args,3)?;finite(v[0]/(positive(v[1])?*nonzero(v[2])?))},
"thermo.heat_capacity"=>{let v=nums(args,2)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*v[1])},
"thermo.delta_t"=>{let v=nums(args,3)?;finite(v[0]/(positive(v[1])?*positive(v[2])?))},
"thermo.latent_heat"=>{let v=nums(args,2)?;nonneg(v[0])?;finite(v[0]*v[1])},
"thermo.mass_from_latent"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"thermo.conduction"=>{let v=nums(args,4)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*v[1]*v[2]/positive(v[3])?)},
"thermo.thermal_resistance"=>{let v=nums(args,3)?;finite(v[0]/(positive(v[1])?*positive(v[2])?))},
"thermo.convection"=>{let v=nums(args,3)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*v[1]*v[2])},
"thermo.radiation"=>{let v=nums(args,4)?;if !(0.0..=1.0).contains(&v[0])||v[1]<0.0||v[2]<0.0||v[3]<0.0{return Err("DOMAIN");}finite(v[0]*SIGMA*v[1]*(v[2].powi(4)-v[3].powi(4)))},
"thermo.stefan_flux"=>{let v=nums(args,2)?;if !(0.0..=1.0).contains(&v[0])||v[1]<0.0{return Err("DOMAIN");}finite(v[0]*SIGMA*v[1].powi(4))},
"thermo.wien_peak"=>{let x=num1(args)?;finite(2.897771955e-3/positive(x)?)},
"thermo.carnot_efficiency"=>{let v=nums(args,2)?;if v[1]<0.0||v[1]>v[0]{return Err("DOMAIN");}finite(1.0-v[1]/positive(v[0])?)},
"thermo.carnot_cop_refrigerator"=>{let v=nums(args,2)?;if v[1]<0.0||v[1]>=v[0]{return Err("DOMAIN");}finite(v[1]/positive(v[0]-v[1])?)},
"thermo.carnot_cop_heatpump"=>{let v=nums(args,2)?;if v[1]<0.0||v[1]>=v[0]{return Err("DOMAIN");}finite(v[0]/positive(v[0]-v[1])?)},
"thermo.ideal_gas_pressure"=>{let v=nums(args,3)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*R*v[1]/positive(v[2])?)},
"thermo.ideal_gas_volume"=>{let v=nums(args,3)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*R*v[1]/positive(v[2])?)},
"thermo.ideal_gas_moles"=>{let v=nums(args,3)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*v[1]/(R*positive(v[2])?))},
"thermo.ideal_gas_temp"=>{let v=nums(args,3)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*v[1]/(R*positive(v[2])?))},
"thermo.isothermal_work"=>{let v=nums(args,4)?;nonneg(v[0])?;positive(v[1])?;finite(v[0]*R*v[1]*(positive(v[3])?/positive(v[2])?).ln())},
"thermo.entropy_isothermal"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"thermo.entropy_ideal_gas_volume"=>{let v=nums(args,3)?;nonneg(v[0])?;finite(v[0]*R*(positive(v[2])?/positive(v[1])?).ln())},
"thermo.linear_expansion"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2])},
"thermo.area_expansion"=>{let v=nums(args,3)?;finite(2.0*v[0]*v[1]*v[2])},
"thermo.volume_expansion"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2])},
"thermo.final_length"=>{let v=nums(args,3)?;finite(v[0]*(1.0+v[1]*v[2]))},
"thermo.final_volume"=>{let v=nums(args,3)?;finite(v[0]*(1.0+v[1]*v[2]))},
"thermo.thermal_stress"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2])},
"thermo.mix_temperature"=>{let v=nums(args,6)?;let d=v[0]*v[1]+v[3]*v[4];if d==0.0{return Err("DIV0");}finite((v[0]*v[1]*v[2]+v[3]*v[4]*v[5])/d)},
"thermo.blackbody_power"=>{let v=nums(args,2)?;nonneg(v[0])?;nonneg(v[1])?;finite(SIGMA*v[0]*v[1].powi(4))},
"thermo.heat_flux"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"thermo.u_value"=>{let x=num1(args)?;finite(1.0/positive(x)?)},
"thermo.fahrenheit_to_rankine"=>{let x=num1(args)?;if x < -459.67{return Err("DOMAIN");}finite(x+459.67)},
"thermo.rankine_to_fahrenheit"=>{let x=num1(args)?;if x<0.0{return Err("DOMAIN");}finite(x-459.67)},
"thermo.celsius_to_rankine"=>{let x=num1(args)?;if x < -273.15{return Err("DOMAIN");}finite((x+273.15)*9.0/5.0)},
"thermo.rankine_to_celsius"=>{let x=num1(args)?;if x<0.0{return Err("DOMAIN");}finite(x*5.0/9.0-273.15)},
"thermo.kelvin_to_celsius"=>{let x=num1(args)?;if x<0.0{return Err("DOMAIN");}finite(x-273.15)},
"thermo.celsius_to_kelvin"=>{let x=num1(args)?;if x < -273.15{return Err("DOMAIN");}finite(x+273.15)},
"thermo.thermal_diffusivity"=>{let v=nums(args,3)?;nonneg(v[0])?;finite(v[0]/(positive(v[1])?*positive(v[2])?))},
"thermo.fourier_number"=>{let v=nums(args,3)?;nonneg(v[0])?;nonneg(v[1])?;finite(v[0]*v[1]/positive(v[2]*v[2])?)},
_=>Err("OP")}}
fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
