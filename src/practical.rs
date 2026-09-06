use serde_json::{Value,json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "pct.of" | "pct.change" | "pct.diff" |
        "pct.increase" | "pct.decrease" | "pct.ratio" | "pct.reverse" |
        "pct.point_change" | "pct.apply" |
        "fin.simple_interest" | "fin.compound" | "fin.present_value" | "fin.future_value" |
        "fin.loan_payment" | "fin.roi" | "fin.cagr" | "fin.discount" |
        "fin.npv" | "fin.irr" | "fin.annuity_pv" | "fin.annuity_fv" | "fin.perpetuity_pv" |
        "fin.loan_balance" | "fin.loan_total_interest" | "fin.amort_interest" |
        "fin.amort_principal" | "fin.depreciation_straight" | "fin.depreciation_declining" |
        "fin.depreciation_syd" | "fin.break_even_units" | "fin.margin" | "fin.markup" |
        "fin.effective_rate" | "fin.nominal_rate" | "fin.rule72" | "fin.payback_period" |
        "fin.bond_price" | "fin.real_rate" |
        "unit.length" | "unit.mass" | "unit.data" | "unit.temp" | "unit.time" |
        "unit.area" | "unit.volume" | "unit.speed" | "unit.angle" | "unit.frequency" |
        "unit.pressure" | "unit.energy" | "unit.power" |
        "unit.force" | "unit.torque" | "unit.density" | "unit.flow" |
        "unit.acceleration" | "unit.charge" | "unit.illuminance"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "pct.of"=>{let(a,p)=two(args)?;finite(a*p/100.0)},
        "pct.change"=>{let(old,new)=two(args)?;if old==0.0{Err("DIV0")}else{finite((new-old)/old*100.0)}},
        "pct.diff"=>{let(a,b)=two(args)?;let avg=(a.abs()+b.abs())/2.0;if avg==0.0{Err("DIV0")}else{finite((a-b).abs()/avg*100.0)}},
        "pct.increase"=>{let(v,p)=two(args)?;finite(v*(1.0+p/100.0))},
        "pct.decrease"=>{let(v,p)=two(args)?;finite(v*(1.0-p/100.0))},
        "pct.ratio"=>{let(part,whole)=two(args)?;if whole==0.0{Err("DIV0")}else{finite(part/whole*100.0)}},
        "pct.reverse"=>{let(final_,p)=two(args)?;let f=1.0+p/100.0;if f==0.0{Err("DIV0")}else{finite(final_/f)}},
        "pct.point_change"=>{let(a,b)=two(args)?;finite(b-a)},
        "pct.apply"=>pct_apply(args),
        "fin.simple_interest"=>{if args.len()!=3{return Err("ARG");}let p=num(&args[0])?;let r=num(&args[1])?;let t=num(&args[2])?;finite(p*(1.0+r/100.0*t))},
        "fin.compound"=>compound(args),
        "fin.present_value"=>present_value(args),
        "fin.future_value"=>future_value(args),
        "fin.loan_payment"=>loan_payment(args),
        "fin.roi"=>{let(a,b)=two(args)?;if a==0.0{Err("DIV0")}else{finite((b-a)/a*100.0)}},
        "fin.cagr"=>cagr(args),
        "fin.discount"=>discount(args),
        "fin.npv"=>npv_op(args),
        "fin.irr"=>irr(args),
        "fin.annuity_pv"=>annuity_pv(args),
        "fin.annuity_fv"=>annuity_fv(args),
        "fin.perpetuity_pv"=>{let(pmt,r)=two(args)?;let r=r/100.0;if r<=0.0{Err("DOMAIN")}else{finite(pmt/r)}},
        "fin.loan_balance"=>loan_balance(args),
        "fin.loan_total_interest"=>loan_total_interest(args),
        "fin.amort_interest"=>amort(args,true),
        "fin.amort_principal"=>amort(args,false),
        "fin.depreciation_straight"=>dep_straight(args),
        "fin.depreciation_declining"=>dep_declining(args),
        "fin.depreciation_syd"=>dep_syd(args),
        "fin.break_even_units"=>break_even_units(args),
        "fin.margin"=>{let(cost,price)=two(args)?;if price==0.0{Err("DIV0")}else{finite((price-cost)/price*100.0)}},
        "fin.markup"=>{let(cost,price)=two(args)?;if cost==0.0{Err("DIV0")}else{finite((price-cost)/cost*100.0)}},
        "fin.effective_rate"=>effective_rate(args),
        "fin.nominal_rate"=>nominal_rate(args),
        "fin.rule72"=>{let r=one(args)?;if r<=0.0{Err("DOMAIN")}else{finite(72.0/r)}},
        "fin.payback_period"=>payback_period(args),
        "fin.bond_price"=>bond_price(args),
        "fin.real_rate"=>{let(nominal,inflation)=two(args)?;let i=inflation/100.0;if 1.0+i<=0.0{return Err("DOMAIN");}finite(((1.0+nominal/100.0)/(1.0+i)-1.0)*100.0)},
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
        "unit.force"=>convert(args,FORCE),
        "unit.torque"=>convert(args,TORQUE),
        "unit.density"=>convert(args,DENSITY),
        "unit.flow"=>convert(args,FLOW),
        "unit.acceleration"=>convert(args,ACCELERATION),
        "unit.charge"=>convert(args,CHARGE),
        "unit.illuminance"=>convert(args,ILLUMINANCE),
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
// Unit keys are matched case-insensitively, so a table must never contain two
// keys that differ only by case (e.g. MN and mN): the first would silently win.
const FORCE:&[(&str,f64)]=&[("N",1.0),("kN",1000.0),("MN",1_000_000.0),("dyn",0.00001),("lbf",4.4482216152605),("kgf",9.80665)];
const TORQUE:&[(&str,f64)]=&[("Nm",1.0),("kNm",1000.0),("lbft",1.3558179483314003),("lbin",0.1129848290276167)];
const DENSITY:&[(&str,f64)]=&[("kgm3",1.0),("gcm3",1000.0),("gL",1.0),("lbft3",16.018463373960138),("lbin3",27679.90471020312)];
const FLOW:&[(&str,f64)]=&[("m3s",1.0),("m3h",1.0/3600.0),("Ls",0.001),("Lmin",1.0/60000.0),("gpm",0.0000630901964),("cfm",0.0004719474432)];
const ACCELERATION:&[(&str,f64)]=&[("mps2",1.0),("g",9.80665),("ftps2",0.3048),("gal",0.01)];
const CHARGE:&[(&str,f64)]=&[("C",1.0),("mC",0.001),("uC",0.000001),("Ah",3600.0),("mAh",3.6)];
const ILLUMINANCE:&[(&str,f64)]=&[("lux",1.0),("phot",10000.0),("footcandle",10.763910416709722),("nox",0.001)];

fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{if args.len()!=1{return Err("ARG");}num(&args[0])}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{if args.len()!=2{return Err("ARG");}Ok((num(&args[0])?,num(&args[1])?))}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn series(v:&Value)->Result<Vec<f64>,&'static str>{
    let a=v.as_array().ok_or("TYPE")?;
    if a.is_empty(){return Err("EMPTY");}
    if a.len()>100_000{return Err("LIMIT");}
    a.iter().map(num).collect()
}
/// Whole-number count argument. Rejects fractions rather than truncating them:
/// a period index of 2.5 is a caller mistake, not a request to round.
fn count(v:&Value)->Result<usize,&'static str>{let x=num(v)?;if x<0.0||x.fract()!=0.0||x>1e9{return Err("DOMAIN");}Ok(x as usize)}
fn compound(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let p=num(&args[0])?;let r=num(&args[1])?;let n=num(&args[2])?;let t=num(&args[3])?;if n<=0.0{return Err("DOMAIN");}finite(p*(1.0+r/100.0/n).powf(n*t))}
fn present_value(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let fv=num(&args[0])?;let r=num(&args[1])?/100.0;let n=num(&args[2])?;if 1.0+r<=0.0{return Err("DOMAIN");}finite(fv/(1.0+r).powf(n))}
fn future_value(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let pv=num(&args[0])?;let r=num(&args[1])?/100.0;let n=num(&args[2])?;if 1.0+r<0.0{return Err("DOMAIN");}finite(pv*(1.0+r).powf(n))}
fn loan_payment(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let p=num(&args[0])?;let annual=num(&args[1])?/100.0;let months=args[2].as_u64().ok_or("TYPE")? as usize;if months==0{return Err("DOMAIN");}let r=annual/12.0;if r==0.0{return finite(p/months as f64);}let q=(1.0+r).powf(months as f64);finite(p*r*q/(q-1.0))}
fn cagr(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let begin=num(&args[0])?;let end=num(&args[1])?;let years=num(&args[2])?;if begin<=0.0||end<0.0||years<=0.0{return Err("DOMAIN");}finite(((end/begin).powf(1.0/years)-1.0)*100.0)}
fn discount(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let price=num(&args[0])?;let pct=num(&args[1])?;finite(price*(1.0-pct/100.0))}
fn convert(args:&[Value],table:&[(&str,f64)])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let from=args[1].as_str().ok_or("TYPE")?;let to=args[2].as_str().ok_or("TYPE")?;let f=table.iter().find(|(u,_)|u.eq_ignore_ascii_case(from)).map(|x|x.1).ok_or("UNIT")?;let t=table.iter().find(|(u,_)|u.eq_ignore_ascii_case(to)).map(|x|x.1).ok_or("UNIT")?;finite(x*f/t)}
fn temp(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=num(&args[0])?;let from=args[1].as_str().ok_or("TYPE")?;let to=args[2].as_str().ok_or("TYPE")?;let c=match from{"C"|"c"=>x,"F"|"f"=>(x-32.0)*5.0/9.0,"K"|"k"=>x-273.15,_=>return Err("UNIT")};let out=match to{"C"|"c"=>c,"F"|"f"=>c*9.0/5.0+32.0,"K"|"k"=>c+273.15,_=>return Err("UNIT")};if (to.eq_ignore_ascii_case("k")&&out<0.0)||(from.eq_ignore_ascii_case("k")&&x<0.0){Err("DOMAIN")}else{finite(out)}}

/// Successive percentage changes, compounded in order. [10, -10] on 100 gives
/// 99, not 100: the second change applies to the already-changed value.
fn pct_apply(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let v=num(&args[0])?;let changes=series(&args[1])?;let mut out=v;for c in changes{out*=1.0+c/100.0;}finite(out)}

/// Net present value with the first cash flow at t=0, undiscounted.
fn npv_at(rate:f64,flows:&[f64])->f64{flows.iter().enumerate().map(|(t,cf)|cf/(1.0+rate).powi(t as i32)).sum()}
fn npv_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let r=num(&args[0])?/100.0;let flows=series(&args[1])?;if 1.0+r<=0.0{return Err("DOMAIN");}finite(npv_at(r,&flows))}

/// Internal rate of return by bisection: a fixed bracket and a fixed iteration
/// count, so the answer is a pure function of the input on every platform.
/// Newton would be faster and would make the result depend on the starting
/// guess and on floating-point path; that trade is not worth it here.
fn irr(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=1{return Err("ARG");}
    let flows=series(&args[0])?;
    if flows.len()<2{return Err("DOMAIN");}
    let (mut lo,mut hi)=(-0.9999_f64,10.0_f64);
    let (flo,fhi)=(npv_at(lo,&flows),npv_at(hi,&flows));
    if !flo.is_finite()||!fhi.is_finite(){return Err("NONFINITE");}
    // No sign change means no root in the bracket: report it rather than
    // returning whichever endpoint happens to be closer to zero.
    if flo*fhi>0.0{return Err("NO_CONVERGE");}
    for _ in 0..200{
        let mid=(lo+hi)/2.0;
        if npv_at(mid,&flows)*flo>0.0{lo=mid;}else{hi=mid;}
    }
    finite((lo+hi)/2.0*100.0)
}

fn annuity_pv(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let pmt=num(&args[0])?;let r=num(&args[1])?/100.0;let n=num(&args[2])?;
    if n<0.0{return Err("DOMAIN");}
    if r==0.0{return finite(pmt*n);}
    if 1.0+r<=0.0{return Err("DOMAIN");}
    finite(pmt*(1.0-(1.0+r).powf(-n))/r)
}
fn annuity_fv(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let pmt=num(&args[0])?;let r=num(&args[1])?/100.0;let n=num(&args[2])?;
    if n<0.0{return Err("DOMAIN");}
    if r==0.0{return finite(pmt*n);}
    if 1.0+r<=0.0{return Err("DOMAIN");}
    finite(pmt*((1.0+r).powf(n)-1.0)/r)
}

/// Level monthly payment for a fully amortising loan.
fn monthly_payment(principal:f64,annual_percent:f64,months:usize)->Result<f64,&'static str>{
    if months==0{return Err("DOMAIN");}
    let r=annual_percent/100.0/12.0;
    if 1.0+r<=0.0{return Err("DOMAIN");}
    if r==0.0{return Ok(principal/months as f64);}
    let q=(1.0+r).powi(months as i32);
    Ok(principal*r*q/(q-1.0))
}
/// Outstanding balance after `paid` payments.
fn balance_after(principal:f64,annual_percent:f64,months:usize,paid:usize)->Result<f64,&'static str>{
    let pmt=monthly_payment(principal,annual_percent,months)?;
    let r=annual_percent/100.0/12.0;
    if r==0.0{return Ok(principal-pmt*paid as f64);}
    let q=(1.0+r).powi(paid as i32);
    Ok(principal*q-pmt*(q-1.0)/r)
}
fn loan_balance(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=4{return Err("ARG");}
    let p=num(&args[0])?;let rate=num(&args[1])?;
    let months=count(&args[2])?;let paid=count(&args[3])?;
    if months==0||paid>months{return Err("DOMAIN");}
    finite(balance_after(p,rate,months,paid)?)
}
fn loan_total_interest(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let p=num(&args[0])?;let rate=num(&args[1])?;let months=count(&args[2])?;
    let pmt=monthly_payment(p,rate,months)?;
    finite(pmt*months as f64-p)
}
/// Interest or principal portion of one scheduled payment, 1-indexed.
fn amort(args:&[Value],want_interest:bool)->Result<Value,&'static str>{
    if args.len()!=4{return Err("ARG");}
    let p=num(&args[0])?;let rate=num(&args[1])?;
    let months=count(&args[2])?;let period=count(&args[3])?;
    if months==0||period==0||period>months{return Err("DOMAIN");}
    let pmt=monthly_payment(p,rate,months)?;
    let opening=balance_after(p,rate,months,period-1)?;
    let interest=opening*(rate/100.0/12.0);
    finite(if want_interest{interest}else{pmt-interest})
}

fn dep_straight(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let cost=num(&args[0])?;let salvage=num(&args[1])?;let life=num(&args[2])?;if life<=0.0||salvage>cost{return Err("DOMAIN");}finite((cost-salvage)/life)}
/// Double-declining balance for one period, clamped so book value never falls
/// below salvage -- the clamp is part of the method, not a guard bolted on.
fn dep_declining(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=4{return Err("ARG");}
    let cost=num(&args[0])?;let salvage=num(&args[1])?;
    let life=count(&args[2])?;let period=count(&args[3])?;
    if life==0||period==0||period>life||salvage>cost||cost<0.0{return Err("DOMAIN");}
    let rate=2.0/life as f64;
    let mut book=cost;
    let mut charge=0.0;
    for _ in 0..period{
        charge=(book*rate).min(book-salvage).max(0.0);
        book-=charge;
    }
    finite(charge)
}
fn dep_syd(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=4{return Err("ARG");}
    let cost=num(&args[0])?;let salvage=num(&args[1])?;
    let life=count(&args[2])?;let period=count(&args[3])?;
    if life==0||period==0||period>life||salvage>cost{return Err("DOMAIN");}
    let n=life as f64;
    let sum=n*(n+1.0)/2.0;
    finite((cost-salvage)*(n-period as f64+1.0)/sum)
}

fn break_even_units(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let fixed=num(&args[0])?;let price=num(&args[1])?;let variable=num(&args[2])?;let margin=price-variable;if margin<=0.0{return Err("DOMAIN");}finite(fixed/margin)}
fn effective_rate(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");}
    let nominal=num(&args[0])?/100.0;let n=num(&args[1])?;
    if n<=0.0{return Err("DOMAIN");}
    if 1.0+nominal/n<=0.0{return Err("DOMAIN");}
    finite(((1.0+nominal/n).powf(n)-1.0)*100.0)
}
fn nominal_rate(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let effective=num(&args[0])?/100.0;let n=num(&args[1])?;if n<=0.0||1.0+effective<=0.0{return Err("DOMAIN");}finite(n*((1.0+effective).powf(1.0/n)-1.0)*100.0)}

/// Periods until cumulative inflow covers the initial outlay, interpolated
/// linearly inside the period that crosses it.
fn payback_period(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");}
    let initial=num(&args[0])?;
    let flows=series(&args[1])?;
    if initial<=0.0{return Err("DOMAIN");}
    let mut cumulative=0.0;
    for (i,cf) in flows.iter().enumerate(){
        let next=cumulative+cf;
        if next>=initial{
            if *cf==0.0{return Err("DIV0");}
            return finite(i as f64+(initial-cumulative)/cf);
        }
        cumulative=next;
    }
    Err("NO_CONVERGE")
}

/// Present value of a coupon bond: coupons as an annuity plus discounted face.
fn bond_price(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=4{return Err("ARG");}
    let face=num(&args[0])?;
    let coupon=num(&args[1])?/100.0;
    let market=num(&args[2])?/100.0;
    let periods=num(&args[3])?;
    if periods<0.0||1.0+market<=0.0{return Err("DOMAIN");}
    let c=face*coupon;
    let discounted_face=face/(1.0+market).powf(periods);
    let coupons=if market==0.0{c*periods}else{c*(1.0-(1.0+market).powf(-periods))/market};
    finite(coupons+discounted_face)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(op: &str, a: &[Value]) -> f64 {
        execute(op, a).expect("op not routed").expect("op errored").as_f64().unwrap()
    }
    fn err(op: &str, a: &[Value]) -> &'static str {
        execute(op, a).expect("op not routed").unwrap_err()
    }
    fn close(a: f64, b: f64) { assert!((a - b).abs() < 1e-9 * b.abs().max(1.0), "{a} != {b}"); }

    #[test]
    fn percentage_operations_match_their_definitions() {
        close(n("pct.increase", &[json!(200), json!(15)]), 230.0);
        close(n("pct.decrease", &[json!(200), json!(15)]), 170.0);
        close(n("pct.ratio", &[json!(45), json!(180)]), 25.0);
        close(n("pct.point_change", &[json!(3.5), json!(4.25)]), 0.75);
        // A 10% rise then a 10% fall does not return to the start.
        close(n("pct.apply", &[json!(100), json!([10, -10])]), 99.0);
    }

    #[test]
    fn pct_reverse_undoes_pct_increase() {
        for v in [1.0, 250.0, 1e6] {
            for p in [-40.0, 0.0, 15.0, 300.0] {
                let up = n("pct.increase", &[json!(v), json!(p)]);
                close(n("pct.reverse", &[json!(up), json!(p)]), v);
            }
        }
    }

    #[test]
    fn npv_and_irr_agree_by_definition() {
        let flows = json!([-1000, 400, 400, 400]);
        let rate = n("fin.irr", std::slice::from_ref(&flows));
        // IRR is defined as the rate at which NPV is zero, so ask NPV.
        let at_irr = n("fin.npv", &[json!(rate), flows]);
        assert!(at_irr.abs() < 1e-6, "NPV at the reported IRR was {at_irr}");
    }

    #[test]
    fn irr_without_a_sign_change_does_not_invent_a_root() {
        assert_eq!(err("fin.irr", &[json!([100, 200, 300])]), "NO_CONVERGE");
        assert_eq!(err("fin.irr", &[json!([-100, -200])]), "NO_CONVERGE");
    }

    #[test]
    fn annuity_present_value_equals_the_discounted_sum() {
        let pv = n("fin.annuity_pv", &[json!(1000), json!(5), json!(10)]);
        let mut sum = 0.0;
        for t in 1..=10 { sum += 1000.0 / 1.05_f64.powi(t); }
        close(pv, sum);
    }

    #[test]
    fn effective_and_nominal_rates_round_trip() {
        for periods in [1.0, 2.0, 12.0, 365.0] {
            let ear = n("fin.effective_rate", &[json!(9.0), json!(periods)]);
            close(n("fin.nominal_rate", &[json!(ear), json!(periods)]), 9.0);
        }
    }

    #[test]
    fn amortisation_splits_the_payment_exactly() {
        let (p, rate, months) = (json!(300000), json!(6), json!(360));
        let pmt = n("fin.loan_payment", &[p.clone(), rate.clone(), months.clone()]);
        for period in [1, 2, 180, 360] {
            let i = n("fin.amort_interest", &[p.clone(), rate.clone(), months.clone(), json!(period)]);
            let pr = n("fin.amort_principal", &[p.clone(), rate.clone(), months.clone(), json!(period)]);
            close(i + pr, pmt);
        }
        // The first month's interest is simply the opening balance times the rate.
        close(n("fin.amort_interest", &[p.clone(), rate.clone(), months.clone(), json!(1)]), 1500.0);
        // The loan is exactly repaid at the end of the term.
        let end = n("fin.loan_balance", &[p, rate, months.clone(), months]);
        assert!(end.abs() < 1e-6, "balance after the final payment was {end}");
    }

    #[test]
    fn declining_balance_never_depreciates_below_salvage() {
        let (cost, salvage, life) = (10000.0, 2000.0, 5);
        let total: f64 = (1..=life)
            .map(|k| n("fin.depreciation_declining",
                       &[json!(cost), json!(salvage), json!(life), json!(k)]))
            .sum();
        assert!(total <= cost - salvage + 1e-9, "depreciated {total} past the depreciable base");
    }

    #[test]
    fn sum_of_years_digits_depreciates_the_whole_base() {
        let total: f64 = (1..=5)
            .map(|k| n("fin.depreciation_syd", &[json!(10000), json!(2000), json!(5), json!(k)]))
            .sum();
        close(total, 8000.0);
    }

    #[test]
    fn bond_at_par_prices_at_face() {
        // When the coupon equals the market rate the bond is worth its face value.
        close(n("fin.bond_price", &[json!(1000), json!(5), json!(5), json!(10)]), 1000.0);
    }

    #[test]
    fn new_unit_tables_convert_both_ways() {
        for (op, a, b) in [
            ("unit.force", "kN", "N"), ("unit.torque", "kNm", "Nm"),
            ("unit.density", "gcm3", "kgm3"), ("unit.flow", "m3s", "Ls"),
            ("unit.acceleration", "g", "mps2"), ("unit.charge", "Ah", "C"),
            ("unit.illuminance", "phot", "lux"),
        ] {
            let there = n(op, &[json!(1), json!(a), json!(b)]);
            let back = n(op, &[json!(there), json!(b), json!(a)]);
            close(back, 1.0);
        }
        close(n("unit.force", &[json!(1), json!("kgf"), json!("N")]), 9.80665);
        assert_eq!(err("unit.force", &[json!(1), json!("m"), json!("N")]), "UNIT");
    }

    /// Unit lookup is case-insensitive, so two keys differing only by case would
    /// make one unreachable. This is the guard, not a style preference.
    #[test]
    fn no_unit_table_has_case_colliding_keys() {
        for (name, table) in [
            ("LENGTH", LENGTH), ("MASS", MASS), ("DATA", DATA), ("TIME", TIME),
            ("AREA", AREA), ("VOLUME", VOLUME), ("SPEED", SPEED), ("ANGLE", ANGLE),
            ("FREQUENCY", FREQUENCY), ("PRESSURE", PRESSURE), ("ENERGY", ENERGY),
            ("POWER", POWER), ("FORCE", FORCE), ("TORQUE", TORQUE), ("DENSITY", DENSITY),
            ("FLOW", FLOW), ("ACCELERATION", ACCELERATION), ("CHARGE", CHARGE),
            ("ILLUMINANCE", ILLUMINANCE),
        ] {
            let mut seen = std::collections::HashSet::new();
            for (unit, _) in table {
                assert!(seen.insert(unit.to_ascii_lowercase()),
                        "{name} has two keys differing only by case: {unit}");
            }
        }
    }

    #[test]
    fn guards_reject_impossible_inputs() {
        assert_eq!(err("fin.rule72", &[json!(0)]), "DOMAIN");
        assert_eq!(err("fin.perpetuity_pv", &[json!(100), json!(0)]), "DOMAIN");
        assert_eq!(err("fin.break_even_units", &[json!(100), json!(10), json!(10)]), "DOMAIN");
        assert_eq!(err("fin.depreciation_straight", &[json!(100), json!(200), json!(5)]), "DOMAIN");
        assert_eq!(err("fin.amort_interest", &[json!(1000), json!(5), json!(12), json!(13)]), "DOMAIN");
        assert_eq!(err("fin.loan_balance", &[json!(1000), json!(5), json!(12), json!(13)]), "DOMAIN");
        assert_eq!(err("pct.ratio", &[json!(1), json!(0)]), "DIV0");
        // A fractional period index is a caller mistake, not a rounding request.
        assert_eq!(err("fin.depreciation_syd", &[json!(100), json!(0), json!(5), json!(2.5)]), "DOMAIN");
    }
}
