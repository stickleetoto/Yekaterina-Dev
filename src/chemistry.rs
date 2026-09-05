use serde_json::{json, Value};

const AVOGADRO:f64=6.022_140_76e23;
const GAS_R:f64=8.314_462_618_153_24;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("chem."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "chem.moles_from_mass"=>{let(m,mm)=two(args)?;positive(mm)?;finite(m/mm)},
    "chem.mass_from_moles"=>{let(n,mm)=two(args)?;nonneg(n)?;positive(mm)?;finite(n*mm)},
    "chem.particles_from_moles"=>{let n=one(args)?;nonneg(n)?;finite(n*AVOGADRO)},
    "chem.moles_from_particles"=>{let p=one(args)?;nonneg(p)?;finite(p/AVOGADRO)},
    "chem.molarity"=>{let(n,l)=two(args)?;nonneg(n)?;positive(l)?;finite(n/l)},
    "chem.moles_from_molarity"=>{let(m,l)=two(args)?;nonneg(m)?;nonneg(l)?;finite(m*l)},
    "chem.molality"=>{let(n,kg)=two(args)?;nonneg(n)?;positive(kg)?;finite(n/kg)},
    "chem.mass_percent"=>percent(args,1e2),
    "chem.volume_percent"=>percent(args,1e2),
    "chem.ppm"=>percent(args,1e6),
    "chem.ppb"=>percent(args,1e9),
    "chem.dilution_c2"=>{need(args,3)?;let c1=num(&args[0])?;let v1=num(&args[1])?;let v2=num(&args[2])?;positive(v2)?;finite(c1*v1/v2)},
    "chem.dilution_v2"=>{need(args,3)?;let c1=num(&args[0])?;let v1=num(&args[1])?;let c2=num(&args[2])?;positive(c2)?;finite(c1*v1/c2)},
    "chem.ideal_gas_pressure"=>{need(args,3)?;let n=num(&args[0])?;let t=num(&args[1])?;let v=num(&args[2])?;nonneg(n)?;nonneg(t)?;positive(v)?;finite(n*GAS_R*t/v)},
    "chem.ideal_gas_volume"=>{need(args,3)?;let n=num(&args[0])?;let t=num(&args[1])?;let p=num(&args[2])?;nonneg(n)?;nonneg(t)?;positive(p)?;finite(n*GAS_R*t/p)},
    "chem.ideal_gas_moles"=>{need(args,3)?;let p=num(&args[0])?;let v=num(&args[1])?;let t=num(&args[2])?;nonneg(p)?;nonneg(v)?;positive(t)?;finite(p*v/(GAS_R*t))},
    "chem.ideal_gas_temperature"=>{need(args,3)?;let p=num(&args[0])?;let v=num(&args[1])?;let n=num(&args[2])?;nonneg(p)?;nonneg(v)?;positive(n)?;finite(p*v/(n*GAS_R))},
    "chem.ph_from_h"=>{let h=one(args)?;positive(h)?;finite(-h.log10())},
    "chem.h_from_ph"=>finite(10f64.powf(-one(args)?)),
    "chem.poh_from_oh"=>{let oh=one(args)?;positive(oh)?;finite(-oh.log10())},
    "chem.oh_from_poh"=>finite(10f64.powf(-one(args)?)),
    "chem.ph_from_poh"=>finite(14.0-one(args)?),
    "chem.poh_from_ph"=>finite(14.0-one(args)?),
    "chem.henderson_hasselbalch"=>{need(args,3)?;let pka=num(&args[0])?;let base=num(&args[1])?;let acid=num(&args[2])?;positive(base)?;positive(acid)?;finite(pka+(base/acid).log10())},
    "chem.beer_lambert"=>{need(args,3)?;let e=num(&args[0])?;let l=num(&args[1])?;let c=num(&args[2])?;nonneg(e)?;nonneg(l)?;nonneg(c)?;finite(e*l*c)},
    "chem.osmotic_pressure"=>{need(args,3)?;let m=num(&args[0])?;let t=num(&args[1])?;let i=num(&args[2])?;nonneg(m)?;nonneg(t)?;nonneg(i)?;finite(i*m*GAS_R*t)},
    "chem.freezing_point_depression"=>colligative(args),
    "chem.boiling_point_elevation"=>colligative(args),
    "chem.equivalent_weight"=>{let(mm,nf)=two(args)?;positive(mm)?;positive(nf)?;finite(mm/nf)},
    "chem.normality"=>{let(m,nf)=two(args)?;nonneg(m)?;nonneg(nf)?;finite(m*nf)},
    "chem.density"=>{let(m,v)=two(args)?;nonneg(m)?;positive(v)?;finite(m/v)},
    "chem.specific_gravity"=>{let(d,r)=two(args)?;nonneg(d)?;positive(r)?;finite(d/r)},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{need(args,2)?;Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn positive(x:f64)->Result<(),&'static str>{if x>0.0{Ok(())}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<(),&'static str>{if x>=0.0{Ok(())}else{Err("DOMAIN")}}
fn percent(args:&[Value],scale:f64)->Result<Value,&'static str>{let(part,total)=two(args)?;nonneg(part)?;positive(total)?;finite(part/total*scale)}
fn colligative(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let i=num(&args[0])?;let k=num(&args[1])?;let m=num(&args[2])?;nonneg(i)?;nonneg(k)?;nonneg(m)?;finite(i*k*m)}
