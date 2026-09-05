use serde_json::{Value,json};
use crate::curve::{parse_points, point_in_polygon_raw, point_segment_distance, segments_intersect};

type P=[f64;2];
fn num(v:&Value)->Result<f64,&'static str>{let x=v.as_f64().ok_or("TYPE")?;if !x.is_finite(){Err("NONFINITE")}else{Ok(x)}}
fn point(v:&Value)->Result<P,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=2{return Err("SHAPE");}Ok([num(&a[0])?,num(&a[1])?])}
fn radius(v:&Value)->Result<f64,&'static str>{let r=num(v)?;if r<0.0{Err("DOMAIN")}else{Ok(r)}}
fn dist(a:P,b:P)->f64{((a[0]-b[0]).powi(2)+(a[1]-b[1]).powi(2)).sqrt()}
fn poly_edges(poly:&[P])->Vec<(P,P)>{let q=if poly.len()>1&&dist(poly[0],*poly.last().unwrap())<=1e-9{&poly[..poly.len()-1]}else{poly};let mut e=Vec::new();if q.len()>=2{for i in 0..q.len(){e.push((q[i],q[(i+1)%q.len()]));}}e}
fn boundary_distance(p:P,poly:&[P])->Result<f64,&'static str>{let e=poly_edges(poly);if e.is_empty(){return Err("ARG");}Ok(e.iter().map(|(a,b)|point_segment_distance(p,*a,*b)).fold(f64::INFINITY,f64::min))}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    Some(match op{
        "predicate.point_in_polygon"=>point_in_polygon(args),
        "predicate.point_on_segment"=>point_on_segment(args),
        "predicate.segments_intersect"=>segments_intersect_op(args),
        "predicate.point_in_circle"=>point_in_circle(args),
        "predicate.circle_in_circle"=>circle_in_circle(args),
        "predicate.circles_intersect"=>circles_intersect(args),
        "predicate.aabb_intersects"=>aabb_intersects(args),
        "predicate.aabb_contains"=>aabb_contains(args),
        "predicate.point_in_aabb"=>point_in_aabb(args),
        "predicate.polygon_in_polygon"=>polygon_in_polygon(args),
        "predicate.polygons_intersect"=>polygons_intersect_op(args),
        "predicate.disjoint"=>disjoint(args),
        "predicate.circle_in_polygon"=>circle_in_polygon(args),
        "predicate.circle_polygon_clearance"=>circle_polygon_clearance(args),
        "predicate.point_polygon_clearance"=>point_polygon_clearance(args),
        "predicate.segment_circle_intersect"=>segment_circle_intersect(args),
        "predicate.polygon_bbox"=>polygon_bbox(args),
        _=>return None,
    })
}
fn point_in_polygon(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let p=point(&args[0])?;let poly=parse_points(&args[1])?;Ok(json!(point_in_polygon_raw(p,&poly)))}
fn point_on_segment(args:&[Value])->Result<Value,&'static str>{if args.len()<3||args.len()>4{return Err("ARG");}let p=point(&args[0])?;let a=point(&args[1])?;let b=point(&args[2])?;let t=if args.len()==4{num(&args[3])?}else{1e-9};if t<0.0{return Err("DOMAIN");}Ok(json!(point_segment_distance(p,a,b)<=t))}
fn segments_intersect_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}Ok(json!(segments_intersect(point(&args[0])?,point(&args[1])?,point(&args[2])?,point(&args[3])?)))}
fn point_in_circle(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}Ok(json!(dist(point(&args[0])?,point(&args[1])?)<=radius(&args[2])?))}
fn circle_in_circle(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let c1=point(&args[0])?;let r1=radius(&args[1])?;let c2=point(&args[2])?;let r2=radius(&args[3])?;Ok(json!(dist(c1,c2)+r1<=r2))}
fn circles_intersect(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let c1=point(&args[0])?;let r1=radius(&args[1])?;let c2=point(&args[2])?;let r2=radius(&args[3])?;let d=dist(c1,c2);Ok(json!(d<=r1+r2&&d+r1>=r2&&d+r2>=r1))}
fn aabb(v:&Value)->Result<(P,P),&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=2{return Err("SHAPE");}let lo=point(&a[0])?;let hi=point(&a[1])?;if lo[0]>hi[0]||lo[1]>hi[1]{return Err("DOMAIN");}Ok((lo,hi))}
fn aabb_intersects(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let (a0,a1)=aabb(&args[0])?;let (b0,b1)=aabb(&args[1])?;Ok(json!(a0[0]<=b1[0]&&a1[0]>=b0[0]&&a0[1]<=b1[1]&&a1[1]>=b0[1]))}
fn aabb_contains(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let (o0,o1)=aabb(&args[0])?;let (i0,i1)=aabb(&args[1])?;Ok(json!(i0[0]>=o0[0]&&i1[0]<=o1[0]&&i0[1]>=o0[1]&&i1[1]<=o1[1]))}
fn point_in_aabb(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let p=point(&args[0])?;let (lo,hi)=aabb(&args[1])?;Ok(json!(p[0]>=lo[0]&&p[0]<=hi[0]&&p[1]>=lo[1]&&p[1]<=hi[1]))}
fn polygons_intersect(a:&[P],b:&[P])->bool{for (a0,a1) in poly_edges(a){for (b0,b1) in poly_edges(b){if segments_intersect(a0,a1,b0,b1){return true;}}}a.first().is_some_and(|p|point_in_polygon_raw(*p,b))||b.first().is_some_and(|p|point_in_polygon_raw(*p,a))}
fn polygon_in_polygon(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let inner=parse_points(&args[0])?;let outer=parse_points(&args[1])?;if !inner.iter().all(|p|point_in_polygon_raw(*p,&outer)){return Ok(json!(false));}for (a,b) in poly_edges(&inner){let m=[(a[0]+b[0])/2.,(a[1]+b[1])/2.];if !point_in_polygon_raw(m,&outer){return Ok(json!(false));}}Ok(json!(true))}
fn polygons_intersect_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=parse_points(&args[0])?;let b=parse_points(&args[1])?;Ok(json!(polygons_intersect(&a,&b)))}
fn disjoint(args:&[Value])->Result<Value,&'static str>{let v=polygons_intersect_op(args)?;Ok(json!(!v.as_bool().ok_or("TYPE")?))}
fn circle_in_polygon(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let c=point(&args[0])?;let r=radius(&args[1])?;let poly=parse_points(&args[2])?;if !point_in_polygon_raw(c,&poly){return Ok(json!(false));}Ok(json!(boundary_distance(c,&poly)?+1e-12>=r))}
fn circle_polygon_clearance(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let c=point(&args[0])?;let r=radius(&args[1])?;let poly=parse_points(&args[2])?;let d=boundary_distance(c,&poly)?;Ok(json!(if point_in_polygon_raw(c,&poly){d-r}else{-(d+r)}))}
fn point_polygon_clearance(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let p=point(&args[0])?;let poly=parse_points(&args[1])?;let d=boundary_distance(p,&poly)?;Ok(json!(if point_in_polygon_raw(p,&poly){d}else{-d}))}
fn segment_circle_intersect(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let a=point(&args[0])?;let b=point(&args[1])?;let c=point(&args[2])?;let r=radius(&args[3])?;Ok(json!(point_segment_distance(c,a,b)<=r))}
fn polygon_bbox(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let ps=parse_points(&args[0])?;let mut lo=[f64::INFINITY;2];let mut hi=[f64::NEG_INFINITY;2];for p in ps{lo[0]=lo[0].min(p[0]);lo[1]=lo[1].min(p[1]);hi[0]=hi[0].max(p[0]);hi[1]=hi[1].max(p[1]);}Ok(json!([lo,hi]))}
