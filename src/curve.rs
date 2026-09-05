use serde_json::{Value, json};

const MAX_POINTS: usize = 2048;
const EPS: f64 = 1e-12;
pub(crate) type P = [f64;2];

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    Some(match op {
        "curve.is_closed"=>is_closed_op(args),
        "curve.closure_error"=>closure_error_op(args),
        "curve.signed_area"=>signed_area_op(args),
        "curve.area"=>area_op(args),
        "curve.arc_length"=>arc_length_op(args),
        "curve.bbox"=>bbox_op(args),
        "curve.orientation"=>orientation_op(args),
        "curve.self_intersections"=>self_intersections_op(args),
        "curve.is_simple_closed"=>simple_closed_op(args),
        "curve.min_segment"=>segment_extreme(args,true),
        "curve.max_segment"=>segment_extreme(args,false),
        "curve.duplicate_points"=>duplicate_points_op(args),
        "curve.backtrack_count"=>backtrack_count_op(args),
        "curve.centroid"=>centroid_op(args),
        "curve.point_at_fraction"=>point_at_fraction_op(args),
        "curve.audit"=>audit_op(args),
        _=>return None,
    })
}

fn num(v:&Value)->Result<f64,&'static str>{let x=v.as_f64().ok_or("TYPE")?;if !x.is_finite(){Err("NONFINITE")}else{Ok(x)}}
fn point(v:&Value)->Result<P,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=2{return Err("SHAPE");}Ok([num(&a[0])?,num(&a[1])?])}
fn points(v:&Value)->Result<Vec<P>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>MAX_POINTS{return Err("LIMIT");}if a.len()<2{return Err("ARG");}a.iter().map(point).collect()}
fn tol(args:&[Value],idx:usize)->Result<f64,&'static str>{let t=if args.len()>idx{num(&args[idx])?}else{1e-9};if t<0.0{Err("DOMAIN")}else{Ok(t)}}
fn dist(a:P,b:P)->f64{((a[0]-b[0]).powi(2)+(a[1]-b[1]).powi(2)).sqrt()}
fn cross(a:P,b:P,c:P)->f64{(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])}
fn effective<'a>(ps:&'a [P],t:f64)->&'a [P]{if ps.len()>2&&dist(ps[0],*ps.last().unwrap())<=t{&ps[..ps.len()-1]}else{ps}}
fn closure_error(ps:&[P])->f64{dist(ps[0],*ps.last().unwrap())}
fn signed_area(ps:&[P],t:f64)->Result<f64,&'static str>{let q=effective(ps,t);if q.len()<3{return Err("ARG");}let mut s=0.;for i in 0..q.len(){let j=(i+1)%q.len();s+=q[i][0]*q[j][1]-q[j][0]*q[i][1];}Ok(s*0.5)}
fn segments(ps:&[P],closed:bool,t:f64)->Vec<(P,P,usize)>{let q=effective(ps,t);let mut o=Vec::new();for i in 0..q.len().saturating_sub(1){o.push((q[i],q[i+1],i));}if closed&&q.len()>=3{o.push((q[q.len()-1],q[0],q.len()-1));}o}
fn on_segment(a:P,b:P,p:P)->bool{cross(a,b,p).abs()<=EPS&&p[0]>=a[0].min(b[0])-EPS&&p[0]<=a[0].max(b[0])+EPS&&p[1]>=a[1].min(b[1])-EPS&&p[1]<=a[1].max(b[1])+EPS}
pub(crate) fn segments_intersect(a:P,b:P,c:P,d:P)->bool{let c1=cross(a,b,c);let c2=cross(a,b,d);let c3=cross(c,d,a);let c4=cross(c,d,b);if ((c1>EPS&&c2< -EPS)||(c1< -EPS&&c2>EPS))&&((c3>EPS&&c4< -EPS)||(c3< -EPS&&c4>EPS)){return true;}on_segment(a,b,c)||on_segment(a,b,d)||on_segment(c,d,a)||on_segment(c,d,b)}
fn self_intersections(ps:&[P],closed:bool,t:f64)->usize{let seg=segments(ps,closed,t);let n=seg.len();let mut count=0;for i in 0..n{for j in i+1..n{if j==i+1{continue;}if closed&&i==0&&j+1==n{continue;}if segments_intersect(seg[i].0,seg[i].1,seg[j].0,seg[j].1){count+=1;}}}count}
fn polyline_length(ps:&[P])->f64{ps.windows(2).map(|w|dist(w[0],w[1])).sum()}
fn is_closed_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;Ok(json!(closure_error(&ps)<=tol(args,1)?))}
fn closure_error_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}Ok(json!(closure_error(&points(&args[0])?)))}
fn signed_area_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;Ok(json!(signed_area(&ps,tol(args,1)?)?))}
fn area_op(args:&[Value])->Result<Value,&'static str>{let v=signed_area_op(args)?;Ok(json!(v.as_f64().ok_or("TYPE")?.abs()))}
fn arc_length_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}Ok(json!(polyline_length(&points(&args[0])?)))}
fn bbox_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let ps=points(&args[0])?;let mut lo=[f64::INFINITY;2];let mut hi=[f64::NEG_INFINITY;2];for p in ps{for k in 0..2{lo[k]=lo[k].min(p[k]);hi[k]=hi[k].max(p[k]);}}Ok(json!([lo,hi]))}
fn orientation_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;let a=signed_area(&ps,tol(args,1)?)?;Ok(json!(if a>EPS{"ccw"}else if a< -EPS{"cw"}else{"degenerate"}))}
fn self_intersections_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;let t=tol(args,1)?;let closed=closure_error(&ps)<=t;Ok(json!(self_intersections(&ps,closed,t)))}
fn simple_closed_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;let t=tol(args,1)?;let closed=closure_error(&ps)<=t;let q=effective(&ps,t);Ok(json!(closed&&q.len()>=3&&self_intersections(&ps,true,t)==0&&duplicate_count(q,t)==0&&backtrack_count(&ps,t)==0))}
fn segment_extreme(args:&[Value],min:bool)->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let ps=points(&args[0])?;let mut it=ps.windows(2).map(|w|dist(w[0],w[1]));let Some(mut x)=it.next()else{return Err("ARG");};for v in it{x=if min{x.min(v)}else{x.max(v)}}Ok(json!(x))}
fn duplicate_count(ps:&[P],t:f64)->usize{let mut c=0;for i in 0..ps.len(){for j in 0..i{if dist(ps[i],ps[j])<=t{c+=1;break;}}}c}
fn duplicate_points_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;let t=tol(args,1)?;Ok(json!(duplicate_count(effective(&ps,t),t)))}
fn backtrack_count(ps:&[P],t:f64)->usize{let mut c=0;for w in ps.windows(3){let a=[w[1][0]-w[0][0],w[1][1]-w[0][1]];let b=[w[2][0]-w[1][0],w[2][1]-w[1][1]];let na=(a[0]*a[0]+a[1]*a[1]).sqrt();let nb=(b[0]*b[0]+b[1]*b[1]).sqrt();if na>t&&nb>t{let cr=(a[0]*b[1]-a[1]*b[0]).abs();let dot=a[0]*b[0]+a[1]*b[1];if cr<=t*na*nb&&dot<0.0{c+=1;}}}c}
fn backtrack_count_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;let t=tol(args,1)?;Ok(json!(backtrack_count(&ps,t)))}
fn centroid_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;let t=tol(args,1)?;let q=effective(&ps,t);if q.len()<3{return Err("ARG");}let a=signed_area(&ps,t)?;if a.abs()<=EPS{return Err("DEGENERATE");}let mut cx=0.;let mut cy=0.;for i in 0..q.len(){let j=(i+1)%q.len();let cr=q[i][0]*q[j][1]-q[j][0]*q[i][1];cx+=(q[i][0]+q[j][0])*cr;cy+=(q[i][1]+q[j][1])*cr;}Ok(json!([cx/(6.*a),cy/(6.*a)]))}
fn point_at_fraction_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let ps=points(&args[0])?;let f=num(&args[1])?;if !(0.0..=1.0).contains(&f){return Err("DOMAIN");}let total=polyline_length(&ps);if total<=EPS{return Err("DEGENERATE");}let target=f*total;let mut acc=0.;for w in ps.windows(2){let l=dist(w[0],w[1]);if acc+l>=target{let u=if l<=EPS{0.0}else{(target-acc)/l};return Ok(json!([w[0][0]+u*(w[1][0]-w[0][0]),w[0][1]+u*(w[1][1]-w[0][1])]));}acc+=l;}Ok(json!(ps.last().unwrap()))}
fn audit_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let ps=points(&args[0])?;let t=tol(args,1)?;let ce=closure_error(&ps);let closed=ce<=t;let q=effective(&ps,t);let x=self_intersections(&ps,closed,t);let area=if q.len()>=3{signed_area(&ps,t)?.abs()}else{0.0};let bt=backtrack_count(&ps,t);let dup=duplicate_count(q,t);Ok(json!({"c":closed,"s":closed&&q.len()>=3&&x==0&&bt==0&&dup==0,"x":x,"a":area,"l":polyline_length(&ps),"ce":ce,"bt":bt}))}

pub(crate) fn parse_points(v:&Value)->Result<Vec<P>,&'static str>{points(v)}
pub(crate) fn point_in_polygon_raw(p:P,poly:&[P])->bool{let q=if poly.len()>1&&dist(poly[0],*poly.last().unwrap())<=1e-9{&poly[..poly.len()-1]}else{poly};if q.len()<3{return false;}for i in 0..q.len(){if on_segment(q[i],q[(i+1)%q.len()],p){return true;}}let mut inside=false;let mut j=q.len()-1;for i in 0..q.len(){let pi=q[i];let pj=q[j];if ((pi[1]>p[1])!=(pj[1]>p[1]))&&(p[0]<(pj[0]-pi[0])*(p[1]-pi[1])/(pj[1]-pi[1])+pi[0]){inside=!inside;}j=i;}inside}
pub(crate) fn point_segment_distance(p:P,a:P,b:P)->f64{let dx=b[0]-a[0];let dy=b[1]-a[1];let d2=dx*dx+dy*dy;if d2<=EPS{return dist(p,a);}let t=(((p[0]-a[0])*dx+(p[1]-a[1])*dy)/d2).clamp(0.,1.);dist(p,[a[0]+t*dx,a[1]+t*dy])}
