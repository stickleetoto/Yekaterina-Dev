use serde_json::{Value,json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "pct.of" | "pct.change" | "pct.diff" |
        "fin.simple_interest" | "fin.compound" | "fin.present_value" | "fin.future_value" |
        "fin.loan_payment" | "fin.roi" | "fin.cagr" | "fin.discount" |
        "unit.length" | "unit.mass" | "unit.data" | "unit.temp" | "unit.time" |
        "unit.area" | "unit.volume" | "unit.speed" | "unit.angle" | "unit.frequency" |
        "unit.pressure" | "unit.energy" | "unit.power"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "pct.of"=>{let(a,p)=two(args)?;finite(a*p/100.0)},
        "pct.change"=>{let(old,new)=two(args)?;if old==0.0{Err("DIV0")}else{finite((new-old)/old*100.0)}},
        "pct.diff"=>{let(a,b)=two(args)?;let avg=(a.abs()+b.abs())/2.0;if avg==0.0{Err("DIV0")}else{finite((a-b).abs()/avg*100.0)}},
        "fin.simple_interest"=>{if args.len()!=3{return Err("ARG");}let p=num(&args[0])?;let r=num(&args[1])?;let t=num(&args[2])?;finite(p*(1.0+r/100.0*t))},
        "fin.compound"=>compound(args),
        "fin.present_value"=>present_value(args),
        "fin.future_value"=>future_value(args),
        "fin.loan_payment"=>loan_payment(args),
        "fin.roi"=>{let(a,b)=two(args)?;if a==0.0{Err("DIV0")}else{finite((b-a)/a*100.0)}},
        "fin.cagr"=>cagr(args),
        "fin.discount"=>discount(args),
        "unit.length"=>convert(args,LENGTH),
        "unit.mass"=>convert(args,MASS),
        "unit.data"=>convert(args,DATA),
        "unit.temp"=>temp(args),
        "unit.time"=>convert(args,TIME),
        "unit.area"=>convert(args,AREA),
        "unit.volume"=>convert(args,VOLUME),
        "unit.speed"=>convert(args,SPEED),
        "unit.angle"=>convert(args,ANGLE),
        "unit.frequency"=>convert(args,FREQUENCY),
        "unit.pressure"=>convert(args,PRESSURE),
        "unit.energy"=>convert(args,ENERGY),
        "unit.power"=>convert(args,POWER),
        _ => Err("OP"),
    }
}

const LENGTH:&[(&str,f64)]=&[("m",1.0),("km",1000.0),("cm",0.01),("mm",0.001),("in",0.0254),("ft",0.3048),("yd",0.9144),("mi",1609.344)];
const MASS:&[(&str,f64)]=&[("kg",1.0),("g",0.001),("mg",0.000001),("lb",0.45359237),("oz",0.028349523125)];
const DATA:&[(&str,f64)]=&[("B",1.0),("KB",1000.0),("MB",1_000_000.0),("GB",1_000_000_000.0),("TB",1_000_000_000_000.0),("KiB",1024.0),("MiB",1_048_576.0),("GiB",1_073_741_824.0),("TiB",1_099_511_627_776.0),("bit",0.125)];
const TIME:&[(&str,f64)]=&[("s",1.0),("ms",0.001),("us",0.000001),("ns",0.000000001),("min",60.0),("h",3600.0),("day",86400.0),("week",604800.0)];
const AREA:&[(&str,f64)]=&[("m2",1.0),("km2",1_000_000.0),("cm2",0.0001),("mm2",0.000001),("ft2",0.09290304),("acre",4046.8564224),("ha",10000.0)];
const VOLUME:&[(&str,f64)]=&[("m3",1.0),("L",0.001),("mL",0.000001),("cm3",0.000001),("ft3",0.028316846592),("gal_us",0.003785411784)];
const SPEED:&[(&str,f64)]=&[("mps",1.0),("kmh",1.0/3.6),("mph",0.44704),("knot",0.5144444444444445),("fps",0.3048)];
const ANGLE:&[(&str,f64)]=&[("rad",1.0),("deg",std::f64::consts::PI/180.0),("turn",std::f64::consts::TAU)];
const FREQUENCY:&[(&str,f64)]=&[("Hz",1.0),("kHz",1000.0),("MHz",1_000_000.0),("GHz",1_000_000_000.0)];
const PRESSURE:&[(&str,f64)]=&[("Pa",1.0),("kPa",1000.0),("MPa",1_000_000.0),("bar",100_000.0),("atm",101_325.0),("psi",6894.757293168)];
const ENERGY:&[(&str,f64)]=&[("J",1.0),("kJ",1000.0),("MJ",1_000_000.0),("Wh",3600.0),("kWh",3_600_000.0),("cal",4.184),("kcal",4184.0)];
const POWER:&[(&str,f64)]=&[("W",1.0),("kW",1000.0),("MW",1_000_000.0),("hp",745.6998715822702)];
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{if args.len()!=2{return Err("ARG");}Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn compound(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let p=num(&args[0])?;let r=num(&args[1])?;let n=num(&args[2])?;let t=num(&args[3])?;if n<=0.0{return Err("DOMAIN");}finite(p*(1.0+r/100.0/n).powf(n*t))}
fn present_value(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let fv=num(&args[0])?;let r=num(&args[1])?/100.0;let n=num(&args[2])?;if 1.0+r<=0.0{return Err("DOMAIN");}finite(fv/(1.0+r).powf(n))}
fn future_value(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let pv=num(&args[0])?;let r=num(&args[1])?/100.0;let n=num(&args[2])?;if 1.0+r<0.0{return Err("DOMAIN");}finite(pv*(1.0+r).powf(n))}
fn loan_payment(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let p=num(&args[0])?;let annual=num(&args[1])?/100.0;let months=args[2].as_u64().ok_or("TYPE")? as usize;if months==0{return Err("DOMAIN");}let r=annual/12.0;if r==0.0{return finite(p/months as f64);}let q=(1.0+r).powf(months as f64);finite(p*r*q/(q-1.0))}
fn cagr(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let begin=num(&args[0])?;let end=num(&args[1])?;let years=num(&args[2])?;if begin<=0.0||end<0.0||years<=0.0{return Err("DOMAIN");}finite(((end/begin).powf(1.0/years)-1.0)*100.0)}
fn discount(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let price=num(&args[0])?;let pct=num(&args[1])?;finite(price*(1.0-pct/100.0))}
fn convert(args:&[Value],table:&[(&str,f64)])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let from=args[1].as_str().ok_or("TYPE")?;let to=args[2].as_str().ok_or("TYPE")?;let f=table.iter().find(|(u,_)|u.eq_ignore_ascii_case(from)).map(|x|x.1).ok_or("UNIT")?;let t=table.iter().find(|(u,_)|u.eq_ignore_ascii_case(to)).map(|x|x.1).ok_or("UNIT")?;finite(x*f/t)}
fn temp(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let from=args[1].as_str().ok_or("TYPE")?;let to=args[2].as_str().ok_or("TYPE")?;let c=match from{"C"|"c"=>x,"F"|"f"=>(x-32.0)*5.0/9.0,"K"|"k"=>x-273.15,_=>return Err("UNIT")};let out=match to{"C"|"c"=>c,"F"|"f"=>c*9.0/5.0+32.0,"K"|"k"=>c+273.15,_=>return Err("UNIT")};if (to.eq_ignore_ascii_case("k")&&out<0.0)||(from.eq_ignore_ascii_case("k")&&x<0.0){Err("DOMAIN")}else{finite(out)}}
