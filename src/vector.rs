use serde_json::{Value,json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if matches!(op,
        "vec.add" | "vec.sub" | "vec.scale" | "vec.dot" | "vec.norm" | "vec.distance" |
        "vec.cosine" | "vec.cross3" | "vec.sum" |
        "vec.normalize" | "vec.angle" | "vec.reject" | "vec.reflect" |
        "vec.lerp" | "vec.manhattan" | "vec.chebyshev" | "vec.minkowski" | "vec.hadamard" | "vec.negate" | "vec.abs" |
        "vec.triple3" | "vec.rotate2d"
    ) { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "vec.add"=>binary_vec(args,|a,b|a+b),
        "vec.sub"=>binary_vec(args,|a,b|a-b),
        "vec.scale"=>scale(args),
        "vec.dot"=>dot(args).and_then(finite),
        "vec.norm"=>{let x=one_vec(args)?;finite(x.iter().map(|v|v*v).sum::<f64>().sqrt())},
        "vec.distance"=>{let (a,b)=two_vec(args)?;finite(a.iter().zip(b.iter()).map(|(x,y)|(x-y)*(x-y)).sum::<f64>().sqrt())},
        "vec.cosine"=>cosine(args),
        "vec.cross3"=>cross3(args),
        "vec.sum"=>{let x=one_vec(args)?;finite(x.iter().sum())},
        "vec.normalize"=>normalize(args),
        "vec.angle"=>angle(args),
        "vec.reject"=>project(args).map(|(_,r)|json!(r)),
        "vec.reflect"=>reflect(args),
        "vec.lerp"=>lerp(args),
        "vec.manhattan"=>{let(a,b)=two_vec(args)?;finite(a.iter().zip(b.iter()).map(|(x,y)|(x-y).abs()).sum())},
        "vec.chebyshev"=>{let(a,b)=two_vec(args)?;finite(a.iter().zip(b.iter()).map(|(x,y)|(x-y).abs()).fold(0.0,f64::max))},
        "vec.minkowski"=>minkowski(args),
        "vec.hadamard"=>binary_vec(args,|a,b|a*b),
        "vec.negate"=>map_vec(args,|x|-x),
        "vec.abs"=>map_vec(args,f64::abs),
        "vec.triple3"=>triple3(args),
        "vec.rotate2d"=>rotate2d(args),
        _ => Err("OP"),
    }
}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn vec(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>100_000{return Err("LIMIT");}a.iter().map(num).collect()}
fn one_vec(args:&[Value])->Result<Vec<f64>,&'static str>{if args.len()!=1{return Err("ARG");}vec(&args[0])}
fn two_vec(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{if args.len()!=2{return Err("ARG");}let a=vec(&args[0])?;let b=vec(&args[1])?;if a.len()!=b.len(){Err("SHAPE")}else{Ok((a,b))}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn binary_vec<F:Fn(f64,f64)->f64>(args:&[Value],f:F)->Result<Value,&'static str>{let(a,b)=two_vec(args)?;let out=a.into_iter().zip(b).map(|(x,y)|f(x,y)).collect::<Vec<_>>();if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}}
fn scale(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=vec(&args[0])?;let s=num(&args[1])?;let out=a.into_iter().map(|x|x*s).collect::<Vec<_>>();if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}}
fn dot(args:&[Value])->Result<f64,&'static str>{let(a,b)=two_vec(args)?;let x=a.iter().zip(b.iter()).map(|(x,y)|x*y).sum::<f64>();if x.is_finite(){Ok(x)}else{Err("NONFINITE")}}
fn cosine(args:&[Value])->Result<Value,&'static str>{let(a,b)=two_vec(args)?;let d=a.iter().zip(b.iter()).map(|(x,y)|x*y).sum::<f64>();let na=a.iter().map(|x|x*x).sum::<f64>().sqrt();let nb=b.iter().map(|x|x*x).sum::<f64>().sqrt();if na==0.0||nb==0.0{Err("DOMAIN")}else{finite(d/(na*nb))}}
fn cross3(args:&[Value])->Result<Value,&'static str>{let(a,b)=two_vec(args)?;if a.len()!=3{return Err("SHAPE");}Ok(json!([a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]))}

fn nonempty(x:Vec<f64>)->Result<Vec<f64>,&'static str>{if x.is_empty(){Err("EMPTY")}else{Ok(x)}}
fn norm(v:&[f64])->f64{v.iter().map(|x|x*x).sum::<f64>().sqrt()}
fn map_vec<F:Fn(f64)->f64>(args:&[Value],f:F)->Result<Value,&'static str>{let a=one_vec(args)?;let out=a.into_iter().map(f).collect::<Vec<_>>();if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}}
fn normalize(args:&[Value])->Result<Value,&'static str>{
    let a=nonempty(one_vec(args)?)?;
    let n=norm(&a);
    if n==0.0{return Err("DOMAIN");}
    let out=a.into_iter().map(|x|x/n).collect::<Vec<_>>();
    if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}
}
/// Angle between two vectors in radians. The ratio is clamped before `acos`:
/// rounding can push a mathematically valid cosine just past 1, and a NaN there
/// would be an artefact of arithmetic, not of the input.
fn angle(args:&[Value])->Result<Value,&'static str>{
    let(a,b)=two_vec(args)?;
    let(na,nb)=(norm(&a),norm(&b));
    if na==0.0||nb==0.0{return Err("DOMAIN");}
    let c=a.iter().zip(b.iter()).map(|(x,y)|x*y).sum::<f64>()/(na*nb);
    finite(c.clamp(-1.0,1.0).acos())
}
/// Returns both halves of the decomposition a = projection + rejection, so the
/// two operations cannot drift apart.
fn project(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{
    let(a,b)=two_vec(args)?;
    let bb=b.iter().map(|x|x*x).sum::<f64>();
    if bb==0.0{return Err("DOMAIN");}
    let k=a.iter().zip(b.iter()).map(|(x,y)|x*y).sum::<f64>()/bb;
    let proj=b.iter().map(|x|k*x).collect::<Vec<_>>();
    let rej=a.iter().zip(proj.iter()).map(|(x,p)|x-p).collect::<Vec<_>>();
    if proj.iter().chain(rej.iter()).any(|x|!x.is_finite()){return Err("NONFINITE");}
    Ok((proj,rej))
}
/// Reflection of `v` in the hyperplane whose normal is `n`.
fn reflect(args:&[Value])->Result<Value,&'static str>{
    let(v,n)=two_vec(args)?;
    let nn=n.iter().map(|x|x*x).sum::<f64>();
    if nn==0.0{return Err("DOMAIN");}
    let k=2.0*v.iter().zip(n.iter()).map(|(x,y)|x*y).sum::<f64>()/nn;
    let out=v.iter().zip(n.iter()).map(|(x,y)|x-k*y).collect::<Vec<_>>();
    if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}
}
fn lerp(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let a=vec(&args[0])?;let b=vec(&args[1])?;let t=num(&args[2])?;
    if a.len()!=b.len(){return Err("SHAPE");}
    let out=a.iter().zip(b.iter()).map(|(x,y)|x+(y-x)*t).collect::<Vec<_>>();
    if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}
}
fn minkowski(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let a=vec(&args[0])?;let b=vec(&args[1])?;let p=num(&args[2])?;
    if a.len()!=b.len(){return Err("SHAPE");}
    if p<1.0{return Err("DOMAIN");}
    let s=a.iter().zip(b.iter()).map(|(x,y)|(x-y).abs().powf(p)).sum::<f64>();
    finite(s.powf(1.0/p))
}
/// Scalar triple product a . (b x c): the signed volume of the parallelepiped.
fn triple3(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=3{return Err("ARG");}
    let a=vec(&args[0])?;let b=vec(&args[1])?;let c=vec(&args[2])?;
    if a.len()!=3||b.len()!=3||c.len()!=3{return Err("SHAPE");}
    let cross=[b[1]*c[2]-b[2]*c[1],b[2]*c[0]-b[0]*c[2],b[0]*c[1]-b[1]*c[0]];
    finite(a[0]*cross[0]+a[1]*cross[1]+a[2]*cross[2])
}
fn rotate2d(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");}
    let v=vec(&args[0])?;let a=num(&args[1])?;
    if v.len()!=2{return Err("SHAPE");}
    if !a.is_finite(){return Err("NONFINITE");}
    let(s,c)=a.sin_cos();
    let out=[v[0]*c-v[1]*s,v[0]*s+v[1]*c];
    if out.iter().any(|x|!x.is_finite()){Err("NONFINITE")}else{Ok(json!(out))}
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
    fn close_vec(a: &[f64], b: &[f64]) {
        assert_eq!(a.len(), b.len(), "{a:?} vs {b:?}");
        for (x, y) in a.iter().zip(b) { close(*x, *y); }
    }

    #[test]
    fn normalize_produces_unit_length() {
        close_vec(&v("vec.normalize", &[json!([3, 4])]), &[0.6, 0.8]);
        for case in [json!([3, 4]), json!([1, 1, 1]), json!([-2, 0, 0, 5])] {
            let unit = v("vec.normalize", &[case]);
            close(n("vec.norm", &[json!(unit)]), 1.0);
        }
        assert_eq!(err("vec.normalize", &[json!([0, 0])]), "DOMAIN");
    }

    #[test]
    fn angle_is_consistent_with_cosine_similarity() {
        close(n("vec.angle", &[json!([1, 0]), json!([0, 1])]), std::f64::consts::FRAC_PI_2);
        close(n("vec.angle", &[json!([1, 1]), json!([1, 0])]), std::f64::consts::FRAC_PI_4);
        for (a, b) in [(json!([1, 2, 3]), json!([4, 5, 6])), (json!([1, 0]), json!([-1, 0]))] {
            let cos = n("vec.cosine", &[a.clone(), b.clone()]);
            close(n("vec.angle", &[a, b]).cos(), cos);
        }
        // Identical vectors: the clamp must keep acos out of NaN territory.
        close(n("vec.angle", &[json!([0.1, 0.2, 0.3]), json!([0.1, 0.2, 0.3])]), 0.0);
        assert_eq!(err("vec.angle", &[json!([0, 0]), json!([1, 0])]), "DOMAIN");
    }

    /// The rejection is what is left after removing the parallel component, so
    /// it must be orthogonal to the reference and must reconstruct the input.
    #[test]
    fn rejection_is_orthogonal_and_reconstructs() {
        for (a, b) in [(json!([3, 4]), json!([1, 0])), (json!([1, 2, 3]), json!([0, 0, 5]))] {
            let rej = v("vec.reject", &[a.clone(), b.clone()]);
            close(n("vec.dot", &[json!(rej.clone()), b.clone()]), 0.0);
            let parallel = v("vec.sub", &[a.clone(), json!(rej)]);
            // The removed part is a multiple of b, so it is parallel to it.
            close(n("vec.norm", &[json!(v("vec.reject", &[json!(parallel), b]))]), 0.0);
        }
        assert_eq!(err("vec.reject", &[json!([1, 2]), json!([0, 0])]), "DOMAIN");
    }

    /// Reflection is an isometry and its own inverse: length is unchanged and
    /// reflecting twice in the same plane returns the original vector.
    #[test]
    fn reflection_preserves_length_and_is_an_involution() {
        for (x, normal) in [
            (json!([1, -1]), json!([0, 1])),
            (json!([2, 3, 4]), json!([1, 1, 0])),
            (json!([-5, 0.5, 2, 7]), json!([1, 2, 3, 4])),
        ] {
            let original: Vec<f64> = x.as_array().unwrap()
                .iter().map(|q| q.as_f64().unwrap()).collect();
            let once = v("vec.reflect", &[x, normal.clone()]);
            close(n("vec.norm", &[json!(once.clone())]), n("vec.norm", &[json!(original.clone())]));
            let twice = v("vec.reflect", &[json!(once), normal]);
            close_vec(&twice, &original);
        }
        close_vec(&v("vec.reflect", &[json!([1, -1]), json!([0, 1])]), &[1.0, 1.0]);
        assert_eq!(err("vec.reflect", &[json!([1, 2]), json!([0, 0])]), "DOMAIN");
    }

    #[test]
    fn lerp_hits_both_endpoints() {
        let (a, b) = (json!([0, 0]), json!([10, 20]));
        close_vec(&v("vec.lerp", &[a.clone(), b.clone(), json!(0)]), &[0.0, 0.0]);
        close_vec(&v("vec.lerp", &[a.clone(), b.clone(), json!(1)]), &[10.0, 20.0]);
        close_vec(&v("vec.lerp", &[a, b, json!(0.25)]), &[2.5, 5.0]);
    }

    /// Minkowski is the family the other two distances belong to: p=1 must be
    /// manhattan and p=2 must be euclidean, or one of the three is wrong.
    #[test]
    fn minkowski_generalises_the_other_distances() {
        for (a, b) in [
            (json!([1, 2, 3]), json!([4, 6, 3])),
            (json!([0, 0]), json!([-3, 4])),
            (json!([1.5, -2.5, 0.25, 9]), json!([0.5, 2.5, 0.25, -1])),
        ] {
            close(n("vec.minkowski", &[a.clone(), b.clone(), json!(1)]),
                  n("vec.manhattan", &[a.clone(), b.clone()]));
            close(n("vec.minkowski", &[a.clone(), b.clone(), json!(2)]),
                  n("vec.distance", &[a.clone(), b.clone()]));
            // Large p approaches the chebyshev limit.
            let big = n("vec.minkowski", &[a.clone(), b.clone(), json!(64)]);
            let cheb = n("vec.chebyshev", &[a, b]);
            assert!((big - cheb).abs() < 0.1, "{big} should be near {cheb}");
        }
        assert_eq!(err("vec.minkowski", &[json!([1]), json!([2]), json!(0.5)]), "DOMAIN");
    }

    #[test]
    fn elementwise_operations() {
        close_vec(&v("vec.hadamard", &[json!([1, 2, 3]), json!([4, 5, 6])]), &[4.0, 10.0, 18.0]);
        close_vec(&v("vec.negate", &[json!([1, -2, 3])]), &[-1.0, 2.0, -3.0]);
        close_vec(&v("vec.abs", &[json!([1, -2, 3])]), &[1.0, 2.0, 3.0]);
        // The dot product is the sum of the hadamard product.
        let (a, b) = (json!([1, 2, 3]), json!([4, 5, 6]));
        let had = v("vec.hadamard", &[a.clone(), b.clone()]);
        close(n("vec.sum", &[json!(had)]), n("vec.dot", &[a, b]));
    }

    #[test]
    fn triple_product_is_the_determinant() {
        close(n("vec.triple3", &[json!([1, 0, 0]), json!([0, 1, 0]), json!([0, 0, 1])]), 1.0);
        close(n("vec.triple3", &[json!([2, 0, 0]), json!([0, 3, 0]), json!([0, 0, 4])]), 24.0);
        // Coplanar vectors enclose no volume.
        close(n("vec.triple3", &[json!([1, 2, 0]), json!([3, 4, 0]), json!([5, 6, 0])]), 0.0);
        // It equals a . (b x c), which the engine can also compute directly.
        let (a, b, c) = (json!([1, 2, 3]), json!([4, 5, 6]), json!([7, 8, 10]));
        let cross = execute("vec.cross3", &[b, c]).unwrap().unwrap();
        close(n("vec.triple3", &[a.clone(), json!([4, 5, 6]), json!([7, 8, 10])]),
              n("vec.dot", &[a, cross]));
        assert_eq!(err("vec.triple3", &[json!([1, 0]), json!([0, 1]), json!([0, 0])]), "SHAPE");
    }

    #[test]
    fn rotate2d_preserves_length_and_composes() {
        let quarter = std::f64::consts::FRAC_PI_2;
        close_vec(&v("vec.rotate2d", &[json!([1, 0]), json!(quarter)]), &[0.0, 1.0]);
        // Four quarter turns return to the start.
        let mut p = json!([3, -7]);
        for _ in 0..4 { p = json!(v("vec.rotate2d", &[p, json!(quarter)])); }
        close_vec(&p.as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect::<Vec<_>>(),
                  &[3.0, -7.0]);
        close(n("vec.norm", &[json!(v("vec.rotate2d", &[json!([3, -7]), json!(1.234)]))]),
              n("vec.norm", &[json!([3, -7])]));
        assert_eq!(err("vec.rotate2d", &[json!([1, 2, 3]), json!(1)]), "SHAPE");
    }
}
