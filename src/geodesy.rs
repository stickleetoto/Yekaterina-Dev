use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

const R:f64=6_371_008.8; const A:f64=6_378_137.0; const B:f64=6_356_752.314245;
pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("geod."){Some(run(op,args))}else{None}}
fn rad(x:f64)->f64{x.to_radians()} fn deg(x:f64)->f64{x.to_degrees()}
fn hav(lat1:f64,lon1:f64,lat2:f64,lon2:f64)->f64{let p1=rad(lat1);let p2=rad(lat2);let dp=rad(lat2-lat1);let dl=rad(lon2-lon1);let h=(dp/2.0).sin().powi(2)+p1.cos()*p2.cos()*(dl/2.0).sin().powi(2);2.0*h.sqrt().asin()}
fn norm_lon(x:f64)->f64{((x+180.0).rem_euclid(360.0))-180.0}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"geod.great_circle_distance"=>{let v=nums(args,4)?;finite(R*hav(v[0],v[1],v[2],v[3]))},
"geod.central_angle"=>{let v=nums(args,4)?;finite(hav(v[0],v[1],v[2],v[3]))},
"geod.initial_bearing"=>{let v=nums(args,4)?;let p1=rad(v[0]);let p2=rad(v[2]);let dl=rad(v[3]-v[1]);finite((deg((dl.sin()*p2.cos()).atan2(p1.cos()*p2.sin()-p1.sin()*p2.cos()*dl.cos()))+360.0)%360.0)},
"geod.equirect_distance"=>{let v=nums(args,4)?;let x=rad(v[3]-v[1])*rad((v[0]+v[2])/2.0).cos();let y=rad(v[2]-v[0]);finite(R*(x*x+y*y).sqrt())},
"geod.chord_distance"=>{let v=nums(args,2)?;finite(2.0*positive(v[1])?*(v[0]/2.0).sin().abs())},"geod.arc_length"=>{let v=nums(args,2)?;finite(positive(v[1])?*v[0])},
"geod.earth_arc_length"=>finite(R*num1(args)?),"geod.horizon_distance"=>{let h=nonneg(num1(args)?)?;finite((2.0*R*h+h*h).sqrt())},
"geod.horizon_distance_radius"=>{let v=nums(args,2)?;let h=nonneg(v[0])?;let r=positive(v[1])?;finite((2.0*r*h+h*h).sqrt())},
"geod.sphere_circumference"=>finite(std::f64::consts::TAU*positive(num1(args)?)?),"geod.sphere_area"=>{let r=positive(num1(args)?)?;finite(4.0*std::f64::consts::PI*r*r)},
"geod.sphere_volume"=>{let r=positive(num1(args)?)?;finite(4.0/3.0*std::f64::consts::PI*r.powi(3))},"geod.radius_from_circumference"=>finite(positive(num1(args)?)?/std::f64::consts::TAU),
"geod.radius_from_area"=>finite((positive(num1(args)?)?/(4.0*std::f64::consts::PI)).sqrt()),"geod.radius_from_volume"=>finite((positive(num1(args)?)?*3.0/(4.0*std::f64::consts::PI)).cbrt()),
"geod.deg_lat_to_m"=>finite(num1(args)?*std::f64::consts::PI*R/180.0),"geod.m_to_deg_lat"=>finite(num1(args)?*180.0/(std::f64::consts::PI*R)),
"geod.deg_lon_to_m"=>{let v=nums(args,2)?;finite(v[0]*std::f64::consts::PI*R/180.0*rad(v[1]).cos())},"geod.m_to_deg_lon"=>{let v=nums(args,2)?;let c=rad(v[1]).cos();if c.abs()<1e-15{return Err("DOMAIN");}finite(v[0]*180.0/(std::f64::consts::PI*R*c))},
"geod.mercator_x"=>finite(R*rad(num1(args)?)),"geod.mercator_y"=>{let lat=num1(args)?.clamp(-85.05112878,85.05112878);finite(R*((std::f64::consts::FRAC_PI_4+rad(lat)/2.0).tan()).ln())},
"geod.mercator_lon"=>finite(deg(num1(args)?/R)),"geod.mercator_lat"=>finite(deg(2.0*(num1(args)?/R).exp().atan()-std::f64::consts::FRAC_PI_2)),
"geod.normalize_lon"=>finite(norm_lon(num1(args)?)),"geod.clamp_lat"=>finite(num1(args)?.clamp(-90.0,90.0)),
"geod.dms_to_deg"=>{let v=nums(args,3)?;let sign=if v[0]<0.0{-1.0}else{1.0};finite(sign*(v[0].abs()+v[1]/60.0+v[2]/3600.0))},
"geod.deg_to_dms"=>{let x=num1(args)?;let sign=if x<0.0{-1.0}else{1.0};let a=x.abs();let d=a.floor();let m=((a-d)*60.0).floor();let s=(a-d-m/60.0)*3600.0;Ok(json!([sign*d,m,s]))},
"geod.geocentric_radius"=>{let lat=rad(num1(args)?);let c=lat.cos();let s=lat.sin();let n=(A*A*c).powi(2)+(B*B*s).powi(2);let d=(A*c).powi(2)+(B*s).powi(2);finite((n/d).sqrt())},
"geod.gravity_latitude"=>{let p=rad(num1(args)?);let s=p.sin();finite(9.780327*(1.0+0.0053024*s*s-0.0000058*(2.0*p).sin().powi(2)))},
"geod.gravity_altitude"=>{let v=nums(args,2)?;let p=rad(v[0]);let s=p.sin();let g=9.780327*(1.0+0.0053024*s*s-0.0000058*(2.0*p).sin().powi(2));finite(g-3.086e-6*v[1])},
"geod.destination_lat"=>{let v=nums(args,4)?;let p1=rad(v[0]);let b=rad(v[2]);let d=v[3]/R;finite(deg((p1.sin()*d.cos()+p1.cos()*d.sin()*b.cos()).asin()))},
"geod.destination_lon"=>{let v=nums(args,4)?;let p1=rad(v[0]);let l1=rad(v[1]);let b=rad(v[2]);let d=v[3]/R;let p2=(p1.sin()*d.cos()+p1.cos()*d.sin()*b.cos()).asin();finite(norm_lon(deg(l1+(b.sin()*d.sin()*p1.cos()).atan2(d.cos()-p1.sin()*p2.sin()))))},
"geod.antipode_lat"=>finite(-num1(args)?),"geod.antipode_lon"=>finite(norm_lon(num1(args)?+180.0)),"geod.nautical_miles_to_m"=>finite(num1(args)?*1852.0),"geod.m_to_nautical_miles"=>finite(num1(args)?/1852.0),
"geod.km_to_miles"=>finite(num1(args)?/1.609344),"geod.miles_to_km"=>finite(num1(args)?*1.609344),"geod.feet_to_m"=>finite(num1(args)?*0.3048),"geod.m_to_feet"=>finite(num1(args)?/0.3048),
_=>Err("OP")}} fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
