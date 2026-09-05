use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

const C:f64=299_792_458.0; const H:f64=6.62607015e-34;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("optics."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"optics.frequency"=>{let x=num1(args)?;finite(C/positive(x)?)},
"optics.wavelength"=>{let x=num1(args)?;finite(C/positive(x)?)},
"optics.photon_energy_frequency"=>{let x=num1(args)?;finite(H*nonneg(x)?)},
"optics.photon_energy_wavelength"=>{let x=num1(args)?;finite(H*C/positive(x)?)},
"optics.snell_sin2"=>{let v=nums(args,3)?;finite(v[0]*v[1].sin()/positive(v[2])?)},
"optics.critical_angle"=>{let v=nums(args,2)?;if v[0]<=v[1]||v[1]<=0.0{return Err("DOMAIN");}finite((v[1]/positive(v[0])?).asin())},
"optics.brewster_angle"=>{let v=nums(args,2)?;finite((v[1]/positive(v[0])?).atan())},
"optics.thin_lens_f"=>{let v=nums(args,2)?;finite(1.0/(1.0/nonzero(v[0])?+1.0/nonzero(v[1])?))},
"optics.thin_lens_image"=>{let v=nums(args,2)?;finite(1.0/(1.0/nonzero(v[0])?-1.0/nonzero(v[1])?))},
"optics.thin_lens_object"=>{let v=nums(args,2)?;finite(1.0/(1.0/nonzero(v[0])?-1.0/nonzero(v[1])?))},
"optics.magnification"=>{let v=nums(args,2)?;finite(-v[0]/nonzero(v[1])?)},
"optics.lens_power_diopter"=>{let x=num1(args)?;finite(1.0/nonzero(x)?)},
"optics.focal_length_from_power"=>{let x=num1(args)?;finite(1.0/nonzero(x)?)},
"optics.mirror_f"=>{let v=nums(args,2)?;finite(1.0/(1.0/nonzero(v[0])?+1.0/nonzero(v[1])?))},
"optics.f_number"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"optics.aperture_diameter"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"optics.airy_angle"=>{let v=nums(args,2)?;finite(1.22*v[0]/positive(v[1])?)},
"optics.airy_radius"=>{let v=nums(args,2)?;finite(1.22*v[0]*v[1])},
"optics.diffraction_grating_angle"=>{let v=nums(args,3)?;finite((v[0]*v[1]/positive(v[2])?).asin())},
"optics.grating_spacing"=>{let x=num1(args)?;finite(1.0/positive(x)?)},
"optics.malus_intensity"=>{let v=nums(args,2)?;finite(v[0]*v[1].cos().powi(2))},
"optics.inverse_square_intensity"=>{let v=nums(args,2)?;finite(v[0]/(4.0*std::f64::consts::PI*positive(v[1])?.powi(2)))},
"optics.illuminance"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"optics.luminous_intensity"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"optics.solid_angle_cone"=>{let x=num1(args)?;finite(2.0*std::f64::consts::PI*(1.0-x.cos()))},
"optics.refractive_speed"=>{let x=num1(args)?;finite(C/positive(x)?)},
"optics.refractive_index"=>{let x=num1(args)?;finite(C/positive(x)?)},
"optics.optical_path"=>{let v=nums(args,2)?;finite(v[0]*v[1])},
"optics.absorbance"=>{let v=nums(args,2)?;finite(-(positive(v[1])?/positive(v[0])?).log10())},
"optics.transmittance"=>{let v=nums(args,2)?;finite(v[1]/positive(v[0])?)},
"optics.beer_lambert"=>{let v=nums(args,3)?;finite(v[0]*v[1]*v[2])},
"optics.reflection_normal"=>{let v=nums(args,2)?;if v[0]+v[1]==0.0{return Err("DIV0");}finite(((v[0]-v[1])/(v[0]+v[1])).powi(2))},
"optics.doppler_light_approx"=>{let v=nums(args,2)?;finite(v[0]*(1.0-v[1]/C))},
"optics.resolution_rayleigh"=>{let v=nums(args,2)?;finite(1.22*v[0]/positive(v[1])?)},
"optics.depth_of_field_hyperfocal"=>{let v=nums(args,3)?;finite(v[0]*v[0]/(positive(v[1])?*positive(v[2])?)+v[0])},
_=>Err("OP")}}
fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
