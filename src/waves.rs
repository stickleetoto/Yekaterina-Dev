use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("wave."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"wave.speed"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"wave.frequency"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"wave.wavelength"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"wave.period"=>{let x=num1(args)?;finite(1.0/positive(x)?)},
"wave.frequency_from_period"=>{let x=num1(args)?;finite(1.0/positive(x)?)},
"wave.angular_frequency"=>{let x=num1(args)?;finite(std::f64::consts::TAU*x)},
"wave.wave_number"=>{let x=num1(args)?;finite(std::f64::consts::TAU/positive(x)?)},
"wave.phase_velocity"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?)},
"wave.intensity"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"wave.db_intensity"=>{let v=nums(args,2)?;finite(10.0*(positive(v[0])?/positive(v[1])?).log10())},
"wave.db_amplitude"=>{let v=nums(args,2)?;finite(20.0*(positive(v[0])?/positive(v[1])?).log10())},
"wave.intensity_ratio_db"=>{let x=num1(args)?;finite(10_f64.powf(x/10.0))},
"wave.amplitude_ratio_db"=>{let x=num1(args)?;finite(10_f64.powf(x/20.0))},
"wave.sound_intensity_sphere"=>{let v=nums(args,2)?;finite(v[0]/(4.0*std::f64::consts::PI*positive(v[1])?.powi(2)))},
"wave.sound_pressure_level"=>{let v=nums(args,2)?;finite(20.0*(positive(v[0])?/positive(v[1])?).log10())},
"wave.pressure_from_spl"=>{let v=nums(args,2)?;finite(v[1]*10_f64.powf(v[0]/20.0))},
"wave.doppler_observer"=>{let v=nums(args,3)?;finite(v[0]*(v[1]+v[2])/positive(v[1])?)},
"wave.doppler_source"=>{let v=nums(args,3)?;finite(v[0]*v[1]/positive(v[1]-v[2])?)},
"wave.doppler_full"=>{let v=nums(args,4)?;finite(v[0]*(v[1]+v[2])/positive(v[1]-v[3])?)},
"wave.string_speed"=>{let v=nums(args,2)?;finite((positive(v[0])?/positive(v[1])?).sqrt())},
"wave.string_fundamental"=>{let v=nums(args,3)?;finite(0.5/positive(v[0])?*(positive(v[1])?/positive(v[2])?).sqrt())},
"wave.open_pipe_fundamental"=>{let v=nums(args,2)?;finite(v[1]/(2.0*positive(v[0])?))},
"wave.closed_pipe_fundamental"=>{let v=nums(args,2)?;finite(v[1]/(4.0*positive(v[0])?))},
"wave.beat_frequency"=>{let v=nums(args,2)?;finite((v[0]-v[1]).abs())},
"wave.standing_nodes"=>{need(args,1)?;let n=args[0].as_u64().ok_or("TYPE")?;Ok(json!(n.checked_add(1).ok_or("OUT_LIMIT")?))},
"wave.standing_antinodes"=>{need(args,1)?;let n=args[0].as_u64().ok_or("TYPE")?;Ok(json!(n))},
"wave.rms_sine"=>{let x=num1(args)?;finite(x/2.0_f64.sqrt())},
"wave.peak_sine"=>{let x=num1(args)?;finite(x*2.0_f64.sqrt())},
"wave.mean_power_resistive"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"wave.acoustic_impedance"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"wave.reflection_pressure"=>{let v=nums(args,2)?;finite((v[1]-v[0])/nonzero(v[1]+v[0])?)},
"wave.transmission_intensity"=>{let v=nums(args,2)?;finite(4.0*v[0]*v[1]/positive((v[0]+v[1]).powi(2))?)},
"wave.mach"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"wave.wavelength_temperature_air"=>{let v=nums(args,2)?;finite((331.3+0.606*v[1])/positive(v[0])?)},
"wave.sound_speed_air"=>{let x=num1(args)?;finite(331.3+0.606*x)},
_=>Err("OP")}}
fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
