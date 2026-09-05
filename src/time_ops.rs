use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("time."){Some(run(op,args))}else{None}}
fn year(v:&Value)->Result<i64,&'static str>{v.as_i64().ok_or("TYPE")}
fn leap(y:i64)->bool{y%4==0&&(y%100!=0||y%400==0)}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"time.seconds_to_minutes"=>finite(num1(args)?/60.0),"time.minutes_to_seconds"=>finite(num1(args)?*60.0),
"time.seconds_to_hours"=>finite(num1(args)?/3600.0),"time.hours_to_seconds"=>finite(num1(args)?*3600.0),
"time.minutes_to_hours"=>finite(num1(args)?/60.0),"time.hours_to_minutes"=>finite(num1(args)?*60.0),
"time.seconds_to_days"=>finite(num1(args)?/86400.0),"time.days_to_seconds"=>finite(num1(args)?*86400.0),
"time.hours_to_days"=>finite(num1(args)?/24.0),"time.days_to_hours"=>finite(num1(args)?*24.0),
"time.days_to_weeks"=>finite(num1(args)?/7.0),"time.weeks_to_days"=>finite(num1(args)?*7.0),
"time.ms_to_seconds"=>finite(num1(args)?/1000.0),"time.seconds_to_ms"=>finite(num1(args)?*1000.0),
"time.us_to_seconds"=>finite(num1(args)?/1_000_000.0),"time.seconds_to_us"=>finite(num1(args)?*1_000_000.0),
"time.hz_to_period"=>{let x=positive(num1(args)?)?;finite(1.0/x)},"time.period_to_hz"=>{let x=positive(num1(args)?)?;finite(1.0/x)},
"time.rpm_to_hz"=>finite(num1(args)?/60.0),"time.hz_to_rpm"=>finite(num1(args)?*60.0),
"time.per_second_to_per_minute"=>finite(num1(args)?*60.0),"time.per_minute_to_per_second"=>finite(num1(args)?/60.0),
"time.per_hour_to_per_second"=>finite(num1(args)?/3600.0),"time.per_second_to_per_hour"=>finite(num1(args)?*3600.0),
"time.per_day_to_per_second"=>finite(num1(args)?/86400.0),"time.per_second_to_per_day"=>finite(num1(args)?*86400.0),
"time.elapsed"=>{let v=nums(args,2)?;finite(v[1]-v[0])},"time.midpoint"=>{let v=nums(args,2)?;finite((v[0]+v[1])/2.0)},
"time.duration_percent"=>{let v=nums(args,2)?;finite(v[0]/nonzero(v[1])?*100.0)},"time.remaining"=>{let v=nums(args,2)?;finite(v[1]-v[0])},
"time.pace"=>{let v=nums(args,2)?;finite(v[1]/positive(v[0])?)},"time.eta"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"time.cycles"=>{let v=nums(args,2)?;finite(v[0]*v[1])},"time.duty_time"=>{let v=nums(args,2)?;finite(v[0]*v[1]/100.0)},
"time.duty_percent"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?*100.0)},
"time.is_leap_year"=>{need(args,1)?;Ok(json!(leap(year(&args[0])?)))},
"time.days_in_year"=>{need(args,1)?;Ok(json!(if leap(year(&args[0])?){366}else{365}))},
"time.days_in_month"=>{need(args,2)?;let y=year(&args[0])?;let m=args[1].as_u64().ok_or("TYPE")?;let d=match m{1|3|5|7|8|10|12=>31,4|6|9|11=>30,2=>if leap(y){29}else{28},_=>return Err("DOMAIN")};Ok(json!(d))},
"time.hours_in_year"=>{need(args,1)?;Ok(json!((if leap(year(&args[0])?){366}else{365})*24))},
"time.seconds_in_year"=>{need(args,1)?;Ok(json!((if leap(year(&args[0])?){366_i64}else{365_i64})*86400))},
_=>Err("OP")}}
fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
