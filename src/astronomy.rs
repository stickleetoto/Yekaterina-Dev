use serde_json::{json, Value};

const G:f64=6.67430e-11;
const C:f64=299_792_458.0;
const SIGMA:f64=5.670_374_419e-8;
const H:f64=6.626_070_15e-34;
const PC:f64=3.085_677_581_491_367e16;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("astro."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "astro.escape_velocity"=>{let(m,r)=two(args)?;nonneg(m)?;positive(r)?;finite((2.0*G*m/r).sqrt())},
    "astro.orbital_velocity"=>{let(m,r)=two(args)?;nonneg(m)?;positive(r)?;finite((G*m/r).sqrt())},
    "astro.orbital_period"=>{let(m,r)=two(args)?;positive(m)?;positive(r)?;finite(std::f64::consts::TAU*(r.powi(3)/(G*m)).sqrt())},
    "astro.kepler_period"=>{let(a,m)=two(args)?;positive(a)?;positive(m)?;finite(std::f64::consts::TAU*(a.powi(3)/(G*m)).sqrt())},
    "astro.surface_gravity"=>{let(m,r)=two(args)?;nonneg(m)?;positive(r)?;finite(G*m/(r*r))},
    "astro.schwarzschild_radius"=>{let m=one(args)?;nonneg(m)?;finite(2.0*G*m/(C*C))},
    "astro.gravitational_force"=>{need(args,3)?;let m1=num(&args[0])?;let m2=num(&args[1])?;let r=num(&args[2])?;nonneg(m1)?;nonneg(m2)?;positive(r)?;finite(G*m1*m2/(r*r))},
    "astro.sphere_density"=>{let(m,r)=two(args)?;nonneg(m)?;positive(r)?;finite(m/(4.0/3.0*std::f64::consts::PI*r.powi(3)))},
    "astro.luminosity_flux"=>{let(l,d)=two(args)?;nonneg(l)?;positive(d)?;finite(l/(4.0*std::f64::consts::PI*d*d))},
    "astro.luminosity_from_flux"=>{let(f,d)=two(args)?;nonneg(f)?;nonneg(d)?;finite(f*4.0*std::f64::consts::PI*d*d)},
    "astro.distance_modulus"=>{let d=one(args)?;positive(d)?;finite(5.0*d.log10()-5.0)},
    "astro.distance_pc_from_modulus"=>finite(10f64.powf((one(args)?+5.0)/5.0)),
    "astro.parallax_distance_pc"=>{let p=one(args)?;positive(p)?;finite(1.0/p)},
    "astro.parallax_arcsec"=>{let d=one(args)?;positive(d)?;finite(1.0/d)},
    "astro.redshift_velocity_approx"=>{let z=one(args)?;if z<0.0{return Err("DOMAIN");}finite(z*C)},
    "astro.hubble_distance_mpc"=>{let(v,h0)=two(args)?;nonneg(v)?;positive(h0)?;finite(v/h0)},
    "astro.light_travel_time"=>{let d=one(args)?;nonneg(d)?;finite(d/C)},
    "astro.wien_peak"=>{let t=one(args)?;positive(t)?;finite(2.897_771_955e-3/t)},
    "astro.stefan_boltzmann_luminosity"=>{let(r,t)=two(args)?;nonneg(r)?;nonneg(t)?;finite(4.0*std::f64::consts::PI*r*r*SIGMA*t.powi(4))},
    "astro.equilibrium_temperature"=>equilibrium(args),
    "astro.rocket_delta_v"=>rocket(args),
    "astro.hohmann_delta_v1"=>hohmann(args,0),
    "astro.hohmann_delta_v2"=>hohmann(args,1),
    "astro.hohmann_transfer_time"=>hohmann_time(args),
    "astro.synodic_period"=>{let(p1,p2)=two(args)?;positive(p1)?;positive(p2)?;let d=(1.0/p1-1.0/p2).abs();if d==0.0{return Err("DIV0");}finite(1.0/d)},
    "astro.angular_size"=>{let(r,d)=two(args)?;nonneg(r)?;positive(d)?;finite(2.0*(r/d).atan())},
    "astro.small_angle_size"=>{let(a,d)=two(args)?;nonneg(a)?;nonneg(d)?;finite(a*d)},
    "astro.magnitude_flux_ratio"=>{let(m1,m2)=two(args)?;finite(10f64.powf(0.4*(m2-m1)))},
    "astro.flux_magnitude_diff"=>{let(f1,f2)=two(args)?;positive(f1)?;positive(f2)?;finite(-2.5*(f1/f2).log10())},
    "astro.photon_energy_wavelength"=>{let l=one(args)?;positive(l)?;finite(H*C/l)},
    "astro.wavelength_from_photon_energy"=>{let e=one(args)?;positive(e)?;finite(H*C/e)},
    "astro.parsec_to_m"=>{let p=one(args)?;finite(p*PC)},
    "astro.m_to_parsec"=>{let m=one(args)?;finite(m/PC)},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{need(args,2)?;Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn positive(x:f64)->Result<(),&'static str>{if x>0.0{Ok(())}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<(),&'static str>{if x>=0.0{Ok(())}else{Err("DOMAIN")}}
fn equilibrium(args:&[Value])->Result<Value,&'static str>{need(args,4)?;let ts=num(&args[0])?;let rs=num(&args[1])?;let d=num(&args[2])?;let a=num(&args[3])?;positive(ts)?;positive(rs)?;positive(d)?;if !(0.0..=1.0).contains(&a){return Err("DOMAIN");}finite(ts*((1.0-a)*rs*rs/(4.0*d*d)).powf(0.25))}
fn rocket(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let ve=num(&args[0])?;let m0=num(&args[1])?;let m1=num(&args[2])?;positive(ve)?;positive(m0)?;positive(m1)?;if m1>m0{return Err("DOMAIN");}finite(ve*(m0/m1).ln())}
fn hohmann(args:&[Value],which:u8)->Result<Value,&'static str>{need(args,3)?;let mu=num(&args[0])?;let r1=num(&args[1])?;let r2=num(&args[2])?;positive(mu)?;positive(r1)?;positive(r2)?;let a=(r1+r2)/2.0;let v1=(mu/r1).sqrt();let v2=(mu/r2).sqrt();let vt1=(mu*(2.0/r1-1.0/a)).sqrt();let vt2=(mu*(2.0/r2-1.0/a)).sqrt();finite(if which==0{vt1-v1}else{v2-vt2})}
fn hohmann_time(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let mu=num(&args[0])?;let r1=num(&args[1])?;let r2=num(&args[2])?;positive(mu)?;positive(r1)?;positive(r2)?;let a=(r1+r2)/2.0;finite(std::f64::consts::PI*(a.powi(3)/mu).sqrt())}
