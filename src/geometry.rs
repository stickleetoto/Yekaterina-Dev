use std::f64::consts::PI;
use serde_json::{Value,json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "geo.distance2d" | "geo.distance3d" | "geo.midpoint2d" | "geo.circle_area" |
        "geo.circle_circumference" | "geo.rectangle_area" | "geo.rectangle_perimeter" |
        "geo.triangle_area" |
        "geo.midpoint3d" | "geo.cylinder_volume" | "geo.cylinder_area" | "geo.cone_volume" | "geo.cone_area" |
        "geo.cube_volume" | "geo.cube_area" | "geo.box_volume" | "geo.box_area" |
        "geo.pyramid_volume" | "geo.torus_volume" | "geo.torus_area" |
        "geo.ellipse_area" | "geo.ellipse_perimeter" | "geo.trapezoid_area" |
        "geo.parallelogram_area" | "geo.regular_polygon_area" | "geo.regular_polygon_perimeter" |
        "geo.circle_sector_area" | "geo.circle_segment_area" |
        "geo.point_line_distance" | "geo.triangle_area_points"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "geo.distance2d"=>{let p=points(args,2)?;finite(dist(&p.0,&p.1))},
        "geo.distance3d"=>{let p=points(args,3)?;finite(dist(&p.0,&p.1))},
        "geo.midpoint2d"=>{let(a,b)=points(args,2)?;Ok(json!([(a[0]+b[0])/2.0,(a[1]+b[1])/2.0]))},
        "geo.circle_area"=>{let r=one(args)?;if r<0.0{Err("DOMAIN")}else{finite(PI*r*r)}},
        "geo.circle_circumference"=>{let r=one(args)?;if r<0.0{Err("DOMAIN")}else{finite(2.0*PI*r)}},
        "geo.rectangle_area"=>{let(a,b)=two(args)?;if a<0.0||b<0.0{Err("DOMAIN")}else{finite(a*b)}},
        "geo.rectangle_perimeter"=>{let(a,b)=two(args)?;if a<0.0||b<0.0{Err("DOMAIN")}else{finite(2.0*(a+b))}},
        "geo.triangle_area"=>triangle_area(args),
        "geo.midpoint3d"=>{let(a,b)=points(args,3)?;Ok(json!([(a[0]+b[0])/2.0,(a[1]+b[1])/2.0,(a[2]+b[2])/2.0]))},
        "geo.cylinder_volume"=>{let(r,h)=two_nonneg(args)?;finite(PI*r*r*h)},
        "geo.cylinder_area"=>{let(r,h)=two_nonneg(args)?;finite(2.0*PI*r*(r+h))},
        "geo.cone_volume"=>{let(r,h)=two_nonneg(args)?;finite(PI*r*r*h/3.0)},
        "geo.cone_area"=>{let(r,h)=two_nonneg(args)?;finite(PI*r*(r+(r*r+h*h).sqrt()))},
        "geo.cube_volume"=>{let l=nonneg(one(args)?)?;finite(l*l*l)},
        "geo.cube_area"=>{let l=nonneg(one(args)?)?;finite(6.0*l*l)},
        "geo.box_volume"=>{let(w,h,d)=three_nonneg(args)?;finite(w*h*d)},
        "geo.box_area"=>{let(w,h,d)=three_nonneg(args)?;finite(2.0*(w*h+w*d+h*d))},
        "geo.pyramid_volume"=>{let(base,h)=two_nonneg(args)?;finite(base*h/3.0)},
        "geo.torus_volume"=>{let(big,small)=torus(args)?;finite(2.0*PI*PI*big*small*small)},
        "geo.torus_area"=>{let(big,small)=torus(args)?;finite(4.0*PI*PI*big*small)},
        "geo.ellipse_area"=>{let(a,b)=two_nonneg(args)?;finite(PI*a*b)},
        "geo.ellipse_perimeter"=>ellipse_perimeter(args),
        "geo.trapezoid_area"=>{let(a,b,h)=three_nonneg(args)?;finite((a+b)/2.0*h)},
        "geo.parallelogram_area"=>{let(b,h)=two_nonneg(args)?;finite(b*h)},
        "geo.regular_polygon_area"=>{let(n,s)=polygon(args)?;finite(n*s*s/(4.0*(PI/n).tan()))},
        "geo.regular_polygon_perimeter"=>{let(n,s)=polygon(args)?;finite(n*s)},
        "geo.circle_sector_area"=>{let(r,a)=arc(args)?;finite(r*r*a/2.0)},
        "geo.circle_segment_area"=>{let(r,a)=arc(args)?;finite(r*r*(a-a.sin())/2.0)},
        "geo.point_line_distance"=>point_line_distance(args),
        "geo.triangle_area_points"=>triangle_area_points(args),
        _ => Err("OP"),
    }
}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one(args:&[Value])->Result<f64,&'static str>{if args.len()!=1{return Err("ARG");}num(&args[0])}
fn two(args:&[Value])->Result<(f64,f64),&'static str>{if args.len()!=2{return Err("ARG");}Ok((num(&args[0])?,num(&args[1])?))}
fn point(v:&Value,n:usize)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=n{return Err("SHAPE");}a.iter().map(num).collect()}
fn points(args:&[Value],n:usize)->Result<(Vec<f64>,Vec<f64>),&'static str>{if args.len()!=2{return Err("ARG");}Ok((point(&args[0],n)?,point(&args[1],n)?))}
fn dist(a:&[f64],b:&[f64])->f64{a.iter().zip(b.iter()).map(|(x,y)|(x-y)*(x-y)).sum::<f64>().sqrt()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn triangle_area(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let a=num(&args[0])?;let b=num(&args[1])?;let c=num(&args[2])?;if a<=0.0||b<=0.0||c<=0.0||a+b<=c||a+c<=b||b+c<=a{return Err("DOMAIN");}let s=(a+b+c)/2.0;finite((s*(s-a)*(s-b)*(s-c)).sqrt())}

// A negative length is not a small shape, it is a bad argument. Every solid
// below rejects one rather than returning a signed volume.
fn nonneg(x:f64)->Result<f64,&'static str>{if x<0.0{Err("DOMAIN")}else{Ok(x)}}
fn two_nonneg(args:&[Value])->Result<(f64,f64),&'static str>{let(a,b)=two(args)?;Ok((nonneg(a)?,nonneg(b)?))}
fn three_nonneg(args:&[Value])->Result<(f64,f64,f64),&'static str>{if args.len()!=3{return Err("ARG");}Ok((nonneg(num(&args[0])?)?,nonneg(num(&args[1])?)?,nonneg(num(&args[2])?)?))}
/// A torus is only a torus while the tube fits around the hole.
fn torus(args:&[Value])->Result<(f64,f64),&'static str>{let(big,small)=two_nonneg(args)?;if small>big{Err("DOMAIN")}else{Ok((big,small))}}
fn polygon(args:&[Value])->Result<(f64,f64),&'static str>{
    if args.len()!=2{return Err("ARG");}
    let n=num(&args[0])?;
    let s=nonneg(num(&args[1])?)?;
    if n<3.0||n.fract()!=0.0||n>1e6{return Err("DOMAIN");}
    Ok((n,s))
}
/// Central angle in radians, bounded by one full turn so a "sector" cannot
/// silently wrap past the whole circle.
fn arc(args:&[Value])->Result<(f64,f64),&'static str>{
    let(r,a)=two(args)?;
    if r<0.0||a<0.0||a>std::f64::consts::TAU{return Err("DOMAIN");}
    Ok((r,a))
}
/// Ramanujan's second approximation: relative error below 1e-9 for the
/// eccentricities this is used at, and far cheaper than the exact elliptic
/// integral. Approximate by construction, so it is documented as such.
fn ellipse_perimeter(args:&[Value])->Result<Value,&'static str>{
    let(a,b)=two_nonneg(args)?;
    let h=if a+b==0.0{0.0}else{(a-b)*(a-b)/((a+b)*(a+b))};
    finite(PI*(a+b)*(1.0+3.0*h/(10.0+(4.0-3.0*h).sqrt())))
}
fn point_line_distance(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let p=point(&args[0],2)?;let a=point(&args[1],2)?;let b=point(&args[2],2)?;
    let (dx,dy)=(b[0]-a[0],b[1]-a[1]);
    let len=(dx*dx+dy*dy).sqrt();
    if len==0.0{return Err("DEGENERATE");}
    finite(((p[0]-a[0])*dy-(p[1]-a[1])*dx).abs()/len)
}
fn triangle_area_points(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let a=point(&args[0],2)?;let b=point(&args[1],2)?;let c=point(&args[2],2)?;
    finite(((a[0]*(b[1]-c[1])+b[0]*(c[1]-a[1])+c[0]*(a[1]-b[1]))/2.0).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(op: &str, a: &[Value]) -> f64 {
        execute(op, a).expect("op not routed").expect("op errored").as_f64().unwrap()
    }
    fn v(op: &str, a: &[Value]) -> Vec<f64> {
        execute(op, a).expect("op not routed").expect("op errored")
            .as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect()
    }
    fn err(op: &str, a: &[Value]) -> &'static str {
        execute(op, a).expect("op not routed").unwrap_err()
    }
    fn close(a: f64, b: f64) { assert!((a - b).abs() < 1e-9 * b.abs().max(1.0), "{a} != {b}"); }

    #[test]
    fn solids_match_their_closed_forms() {
        close(n("geo.cylinder_volume", &[json!(2), json!(5)]), PI * 4.0 * 5.0);
        close(n("geo.cylinder_area", &[json!(2), json!(5)]), 2.0 * PI * 2.0 * 7.0);
        close(n("geo.cone_volume", &[json!(3), json!(4)]), PI * 9.0 * 4.0 / 3.0);
        // Slant height of a 3-4 cone is exactly 5.
        close(n("geo.cone_area", &[json!(3), json!(4)]), PI * 3.0 * 8.0);
        close(n("geo.cube_volume", &[json!(3)]), 27.0);
        close(n("geo.cube_area", &[json!(3)]), 54.0);
        close(n("geo.box_volume", &[json!(2), json!(3), json!(4)]), 24.0);
        close(n("geo.box_area", &[json!(2), json!(3), json!(4)]), 52.0);
        close(n("geo.pyramid_volume", &[json!(12), json!(5)]), 20.0);
        close(n("geo.torus_volume", &[json!(5), json!(2)]), 2.0 * PI * PI * 5.0 * 4.0);
        close(n("geo.torus_area", &[json!(5), json!(2)]), 4.0 * PI * PI * 10.0);
        assert_eq!(v("geo.midpoint3d", &[json!([0, 0, 0]), json!([2, 4, 6])]), vec![1.0, 2.0, 3.0]);
    }

    /// A cube is a box with equal sides, a cone is a pyramid over a circle:
    /// the general form and the special case must agree.
    #[test]
    fn special_cases_agree_with_the_general_forms() {
        close(n("geo.cube_volume", &[json!(3)]), n("geo.box_volume", &[json!(3), json!(3), json!(3)]));
        close(n("geo.cube_area", &[json!(3)]), n("geo.box_area", &[json!(3), json!(3), json!(3)]));
        let base = n("geo.circle_area", &[json!(3)]);
        close(n("geo.cone_volume", &[json!(3), json!(4)]),
              n("geo.pyramid_volume", &[json!(base), json!(4)]));
        // A trapezoid with equal parallel sides is a parallelogram.
        close(n("geo.trapezoid_area", &[json!(6), json!(6), json!(4)]),
              n("geo.parallelogram_area", &[json!(6), json!(4)]));
    }

    /// An ellipse with equal axes is a circle, and both formulas must say so.
    #[test]
    fn ellipse_degenerates_to_a_circle() {
        for r in [0.5, 1.0, 7.25] {
            close(n("geo.ellipse_area", &[json!(r), json!(r)]), n("geo.circle_area", &[json!(r)]));
            close(n("geo.ellipse_perimeter", &[json!(r), json!(r)]),
                  n("geo.circle_circumference", &[json!(r)]));
        }
    }

    #[test]
    fn sector_and_segment_close_over_a_full_turn() {
        let r = json!(3);
        let whole = n("geo.circle_area", std::slice::from_ref(&r));
        close(n("geo.circle_sector_area", &[r.clone(), json!(std::f64::consts::TAU)]), whole);
        // A half-turn segment is exactly half the disc.
        close(n("geo.circle_segment_area", &[r.clone(), json!(PI)]), whole / 2.0);
        // Sector minus segment is the triangle between the two radii.
        let a = 1.0_f64;
        let sector = n("geo.circle_sector_area", &[r.clone(), json!(a)]);
        let segment = n("geo.circle_segment_area", &[r, json!(a)]);
        close(sector - segment, 9.0 * a.sin() / 2.0);
    }

    #[test]
    fn regular_polygons_match_known_shapes() {
        // A square of side 2.
        close(n("geo.regular_polygon_area", &[json!(4), json!(2)]), 4.0);
        // A hexagon is six equilateral triangles.
        close(n("geo.regular_polygon_area", &[json!(6), json!(2)]), 6.0 * 3.0_f64.sqrt());
        close(n("geo.regular_polygon_perimeter", &[json!(6), json!(2)]), 12.0);
        // As the side count grows the polygon approaches its circumcircle.
        let s = 2.0 * PI / 10_000.0;
        close(n("geo.regular_polygon_perimeter", &[json!(10_000), json!(s)]), 2.0 * PI);
    }

    #[test]
    fn point_geometry_uses_coordinates_correctly() {
        close(n("geo.point_line_distance", &[json!([0, 3]), json!([0, 0]), json!([4, 0])]), 3.0);
        // A point on the line is at distance zero, from either side.
        close(n("geo.point_line_distance", &[json!([2, 0]), json!([0, 0]), json!([4, 0])]), 0.0);
        close(n("geo.triangle_area_points", &[json!([0, 0]), json!([4, 0]), json!([0, 3])]), 6.0);
        // Area from vertices must agree with Heron's formula on the same triangle.
        close(n("geo.triangle_area_points", &[json!([0, 0]), json!([4, 0]), json!([0, 3])]),
              n("geo.triangle_area", &[json!(3), json!(4), json!(5)]));
    }

    #[test]
    fn impossible_shapes_are_rejected() {
        assert_eq!(err("geo.regular_polygon_area", &[json!(2), json!(1)]), "DOMAIN");
        assert_eq!(err("geo.regular_polygon_area", &[json!(6.5), json!(1)]), "DOMAIN");
        assert_eq!(err("geo.torus_volume", &[json!(2), json!(5)]), "DOMAIN");
        assert_eq!(err("geo.cube_volume", &[json!(-1)]), "DOMAIN");
        assert_eq!(err("geo.box_volume", &[json!(1), json!(-1), json!(1)]), "DOMAIN");
        assert_eq!(err("geo.circle_sector_area", &[json!(1), json!(7)]), "DOMAIN");
        assert_eq!(err("geo.point_line_distance",
                       &[json!([0, 1]), json!([2, 2]), json!([2, 2])]), "DEGENERATE");
        assert_eq!(err("geo.midpoint3d", &[json!([0, 0]), json!([1, 1])]), "SHAPE");
    }
}
