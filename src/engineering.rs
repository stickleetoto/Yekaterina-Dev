use serde_json::{json, Value};

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("eng."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "eng.ohm_voltage"=>{let(i,r)=two(args)?;finite(i*r)},
    "eng.ohm_current"=>{let(v,r)=two(args)?;if r==0.0{return Err("DIV0");}finite(v/r)},
    "eng.ohm_resistance"=>{let(v,i)=two(args)?;if i==0.0{return Err("DIV0");}finite(v/i)},
    "eng.power_vi"=>{let(v,i)=two(args)?;finite(v*i)},
    "eng.power_ir"=>{let(i,r)=two(args)?;finite(i*i*r)},
    "eng.power_vr"=>{let(v,r)=two(args)?;if r==0.0{return Err("DIV0");}finite(v*v/r)},
    "eng.resistance_series"=>series(args),
    "eng.resistance_parallel"=>parallel(args),
    "eng.capacitance_series"=>parallel(args),
    "eng.capacitance_parallel"=>series(args),
    "eng.rc_tau"=>{let(r,c)=two(args)?;if r<0.0||c<0.0{return Err("DOMAIN");}finite(r*c)},
    "eng.rl_tau"=>{let(l,r)=two(args)?;if l<0.0||r<=0.0{return Err("DOMAIN");}finite(l/r)},
    "eng.reactance_capacitive"=>{let(f,c)=two(args)?;if f<=0.0||c<=0.0{return Err("DOMAIN");}finite(1.0/(std::f64::consts::TAU*f*c))},
    "eng.reactance_inductive"=>{let(f,l)=two(args)?;if f<0.0||l<0.0{return Err("DOMAIN");}finite(std::f64::consts::TAU*f*l)},
    "eng.impedance_rl"=>{let(r,x)=two(args)?;finite(r.hypot(x))},
    "eng.impedance_rc"=>{let(r,x)=two(args)?;finite(r.hypot(x))},
    "eng.voltage_divider"=>voltage_divider(args),
    "eng.current_divider_two"=>current_divider(args),
    "eng.energy_capacitor"=>{let(c,v)=two(args)?;if c<0.0{return Err("DOMAIN");}finite(0.5*c*v*v)},
    "eng.energy_inductor"=>{let(l,i)=two(args)?;if l<0.0{return Err("DOMAIN");}finite(0.5*l*i*i)},
    "eng.thermal_energy"=>{need(args,3)?;let m=num(&args[0])?;let c=num(&args[1])?;let dt=num(&args[2])?;if m<0.0||c<0.0{return Err("DOMAIN");}finite(m*c*dt)},
    "eng.heat_power"=>{let(q,t)=two(args)?;if t==0.0{return Err("DIV0");}finite(q/t)},
    "eng.ideal_gas_pressure"=>{need(args,3)?;let n=num(&args[0])?;let t=num(&args[1])?;let v=num(&args[2])?;if n<0.0||t<0.0||v<=0.0{return Err("DOMAIN");}finite(n*8.31446261815324*t/v)},
    "eng.ideal_gas_volume"=>{need(args,3)?;let n=num(&args[0])?;let t=num(&args[1])?;let p=num(&args[2])?;if n<0.0||t<0.0||p<=0.0{return Err("DOMAIN");}finite(n*8.31446261815324*t/p)},
    "eng.reynolds"=>{need(args,4)?;let rho=num(&args[0])?;let v=num(&args[1])?;let l=num(&args[2])?;let mu=num(&args[3])?;if rho<0.0||l<0.0||mu<=0.0{return Err("DOMAIN");}finite(rho*v*l/mu)},
    "eng.flow_rate"=>{let(a,v)=two(args)?;if a<0.0{return Err("DOMAIN");}finite(a*v)},
    "eng.dynamic_pressure"=>{let(rho,v)=two(args)?;if rho<0.0{return Err("DOMAIN");}finite(0.5*rho*v*v)},
    "eng.stress"=>{let(f,a)=two(args)?;if a<=0.0{return Err("DOMAIN");}finite(f/a)},
    "eng.strain"=>{let(dl,l)=two(args)?;if l==0.0{return Err("DIV0");}finite(dl/l)},
    "eng.young_modulus"=>{let(s,e)=two(args)?;if e==0.0{return Err("DIV0");}finite(s/e)},
    "eng.hydraulic_power"=>{need(args,4)?;let rho=num(&args[0])?;let g=num(&args[1])?;let q=num(&args[2])?;let h=num(&args[3])?;if rho<0.0{return Err("DOMAIN");}finite(rho*g*q*h)},
    "eng.efficiency"=>{let(o,i)=two(args)?;if i==0.0{return Err("DIV0");}finite(o/i*100.0)},
    "eng.gear_ratio"=>{let(driven,driver)=two(args)?;if driver==0.0{return Err("DIV0");}finite(driven/driver)},
    "eng.mechanical_advantage"=>{let(outf,inf)=two(args)?;if inf==0.0{return Err("DIV0");}finite(outf/inf)},
    "eng.spring_constant"=>{let(f,x)=two(args)?;if x==0.0{return Err("DIV0");}finite(f/x)},
    "eng.conductance"=>{need(args,1)?;let r=num(&args[0])?;if r==0.0{return Err("DIV0");}finite(1.0/r)},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{need(args,2)?;Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn array(args:&[Value])->Result<Vec<f64>,&'static str>{need(args,1)?;let a=args[0].as_array().ok_or("TYPE")?;if a.is_empty()||a.len()>100_000{return Err("EMPTY");}a.iter().map(num).collect()}
fn series(args:&[Value])->Result<Value,&'static str>{let x=array(args)?;if x.iter().any(|v|*v<0.0){return Err("DOMAIN");}finite(x.iter().sum())}
fn parallel(args:&[Value])->Result<Value,&'static str>{let x=array(args)?;if x.iter().any(|v|*v<=0.0){return Err("DOMAIN");}finite(1.0/x.iter().map(|v|1.0/v).sum::<f64>())}
fn voltage_divider(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let vin=num(&args[0])?;let r1=num(&args[1])?;let r2=num(&args[2])?;if r1<0.0||r2<0.0||r1+r2==0.0{return Err("DOMAIN");}finite(vin*r2/(r1+r2))}
fn current_divider(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let it=num(&args[0])?;let r1=num(&args[1])?;let r2=num(&args[2])?;if r1<=0.0||r2<=0.0{return Err("DOMAIN");}finite(it*r2/(r1+r2))}
