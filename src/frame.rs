use serde_json::{Value, json};

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    Some(match op {
        "frame.vec3" => tagged(args, "v"),
        "frame.point3" => tagged(args, "p"),
        "frame.identity" => identity(args),
        "frame.rot_x" => rotation(args, 0),
        "frame.rot_y" => rotation(args, 1),
        "frame.rot_z" => rotation(args, 2),
        "frame.translate" => translate(args),
        "frame.compose" => compose(args),
        "frame.inverse" => inverse(args),
        "frame.transform_vec" => transform_tagged(args, false),
        "frame.transform_point" => transform_tagged(args, true),
        "frame.add_vec" => vec_binary(args, true),
        "frame.sub_vec" => vec_binary(args, false),
        "frame.point_plus_vec" => point_plus_vec(args),
        "frame.point_minus_point" => point_minus_point(args),
        "frame.dot" => dot(args),
        "frame.cross" => cross(args),
        "frame.distance" => distance(args),
        "frame.basis" => basis(args),
        "frame.same" => same(args),
        _ => return None,
    })
}

type Vec3 = [f64; 3];
type Mat3 = [[f64; 3]; 3];

#[derive(Clone)]
struct Transform { from: String, to: String, r: Mat3, p: Vec3 }

fn number(v: &Value) -> Result<f64, &'static str> {
    let x = v.as_f64().ok_or("TYPE")?;
    if !x.is_finite() { return Err("NONFINITE"); }
    Ok(x)
}
fn name(v: &Value) -> Result<String, &'static str> {
    let s = v.as_str().ok_or("TYPE")?;
    if s.is_empty() || s.len() > 128 { return Err("NAME"); }
    Ok(s.to_owned())
}
fn vec3(v: &Value) -> Result<Vec3, &'static str> {
    let a = v.as_array().ok_or("TYPE")?;
    if a.len() != 3 { return Err("SHAPE"); }
    Ok([number(&a[0])?, number(&a[1])?, number(&a[2])?])
}
fn mat3(v: &Value) -> Result<Mat3, &'static str> {
    let a = v.as_array().ok_or("TYPE")?;
    if a.len() != 3 { return Err("SHAPE"); }
    Ok([vec3(&a[0])?, vec3(&a[1])?, vec3(&a[2])?])
}
fn tagged_value(v: &Value, kind: &str) -> Result<(String, Vec3), &'static str> {
    let o = v.as_object().ok_or("TYPE")?;
    if o.get("t").and_then(Value::as_str) != Some(kind) { return Err("TYPE"); }
    let f = name(o.get("f").ok_or("ARG")?)?;
    let x = vec3(o.get("v").ok_or("ARG")?)?;
    Ok((f, x))
}
fn tagged_json(f: &str, kind: &str, v: Vec3) -> Value { json!({"f":f,"t":kind,"v":v}) }
fn transform(v: &Value) -> Result<Transform, &'static str> {
    let o = v.as_object().ok_or("TYPE")?;
    let t = Transform {
        from: name(o.get("from").ok_or("ARG")?)?,
        to: name(o.get("to").ok_or("ARG")?)?,
        r: mat3(o.get("r").ok_or("ARG")?)?,
        p: vec3(o.get("p").ok_or("ARG")?)?,
    };
    if !rigid_rotation(&t.r) { return Err("FRAME"); }
    Ok(t)
}
fn rigid_rotation(r: &Mat3) -> bool {
    let tol = 1e-8;
    for i in 0..3 {
        let norm = (0..3).map(|k| r[k][i] * r[k][i]).sum::<f64>();
        if (norm - 1.0).abs() > tol { return false; }
        for j in i + 1..3 {
            let dot = (0..3).map(|k| r[k][i] * r[k][j]).sum::<f64>();
            if dot.abs() > tol { return false; }
        }
    }
    let det = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1])
        - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
        + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
    (det - 1.0).abs() <= tol
}
fn transform_json(t: &Transform) -> Value { json!({"from":t.from.clone(),"to":t.to.clone(),"r":t.r,"p":t.p}) }
fn tagged(args: &[Value], kind: &str) -> Result<Value, &'static str> {
    if args.len() != 2 { return Err("ARG"); }
    Ok(tagged_json(&name(&args[1])?, kind, vec3(&args[0])?))
}
fn identity(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 { return Err("ARG"); }
    let f = name(&args[0])?;
    Ok(transform_json(&Transform { from:f.clone(), to:f, r:[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]], p:[0.,0.,0.] }))
}
fn rotation(args: &[Value], axis: usize) -> Result<Value, &'static str> {
    if args.len() != 3 { return Err("ARG"); }
    let a = number(&args[0])?; let c=a.cos(); let s=a.sin();
    let r = match axis {
        0 => [[1.,0.,0.],[0.,c,-s],[0.,s,c]],
        1 => [[c,0.,s],[0.,1.,0.],[-s,0.,c]],
        _ => [[c,-s,0.],[s,c,0.],[0.,0.,1.]],
    };
    Ok(transform_json(&Transform{from:name(&args[1])?,to:name(&args[2])?,r,p:[0.,0.,0.]}))
}
fn translate(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 3 { return Err("ARG"); }
    Ok(transform_json(&Transform{from:name(&args[1])?,to:name(&args[2])?,r:[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]],p:vec3(&args[0])?}))
}
fn mv(r: &Mat3, v: Vec3) -> Vec3 {
    [r[0][0]*v[0]+r[0][1]*v[1]+r[0][2]*v[2], r[1][0]*v[0]+r[1][1]*v[1]+r[1][2]*v[2], r[2][0]*v[0]+r[2][1]*v[1]+r[2][2]*v[2]]
}
fn mm(a:&Mat3,b:&Mat3)->Mat3 {
    let mut o=[[0.;3];3];
    for i in 0..3 { for j in 0..3 { o[i][j]=(0..3).map(|k| a[i][k]*b[k][j]).sum(); } }
    o
}
fn add(a:Vec3,b:Vec3)->Vec3 {[a[0]+b[0],a[1]+b[1],a[2]+b[2]]}
fn sub(a:Vec3,b:Vec3)->Vec3 {[a[0]-b[0],a[1]-b[1],a[2]-b[2]]}
fn transpose(a:&Mat3)->Mat3 {[[a[0][0],a[1][0],a[2][0]],[a[0][1],a[1][1],a[2][1]],[a[0][2],a[1][2],a[2][2]]]}
fn compose(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let a=transform(&args[0])?; let b=transform(&args[1])?;
    if a.to!=b.from{return Err("FRAME");}
    let t=Transform{from:a.from,to:b.to,r:mm(&b.r,&a.r),p:add(mv(&b.r,a.p),b.p)}; Ok(transform_json(&t))
}
fn inverse(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=1{return Err("ARG");} let a=transform(&args[0])?; let rt=transpose(&a.r); let p=mv(&rt,[-a.p[0],-a.p[1],-a.p[2]]);
    Ok(transform_json(&Transform{from:a.to,to:a.from,r:rt,p}))
}
fn transform_tagged(args:&[Value],point:bool)->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let t=transform(&args[0])?; let kind=if point{"p"}else{"v"}; let (f,v)=tagged_value(&args[1],kind)?;
    if f!=t.from{return Err("FRAME");} let mut out=mv(&t.r,v); if point{out=add(out,t.p);} Ok(tagged_json(&t.to,kind,out))
}
fn vec_binary(args:&[Value],plus:bool)->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let (fa,a)=tagged_value(&args[0],"v")?; let (fb,b)=tagged_value(&args[1],"v")?; if fa!=fb{return Err("FRAME");}
    Ok(tagged_json(&fa,"v",if plus{add(a,b)}else{sub(a,b)}))
}
fn point_plus_vec(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let (fp,p)=tagged_value(&args[0],"p")?; let (fv,v)=tagged_value(&args[1],"v")?; if fp!=fv{return Err("FRAME");} Ok(tagged_json(&fp,"p",add(p,v)))
}
fn point_minus_point(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let (fa,a)=tagged_value(&args[0],"p")?; let (fb,b)=tagged_value(&args[1],"p")?; if fa!=fb{return Err("FRAME");} Ok(tagged_json(&fa,"v",sub(a,b)))
}
fn dot(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let (fa,a)=tagged_value(&args[0],"v")?; let (fb,b)=tagged_value(&args[1],"v")?; if fa!=fb{return Err("FRAME");} Ok(json!(a[0]*b[0]+a[1]*b[1]+a[2]*b[2]))
}
fn cross(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let (fa,a)=tagged_value(&args[0],"v")?; let (fb,b)=tagged_value(&args[1],"v")?; if fa!=fb{return Err("FRAME");}
    Ok(tagged_json(&fa,"v",[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]))
}
fn distance(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} let (fa,a)=tagged_value(&args[0],"p")?; let (fb,b)=tagged_value(&args[1],"p")?; if fa!=fb{return Err("FRAME");} let d=sub(a,b); Ok(json!((d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()))
}
fn basis(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=1{return Err("ARG");} let t=transform(&args[0])?;
    Ok(json!({"x":mv(&t.r,[1.,0.,0.]),"y":mv(&t.r,[0.,1.,0.]),"z":mv(&t.r,[0.,0.,1.])}))
}
fn same(args:&[Value])->Result<Value,&'static str>{
    if args.len()!=2{return Err("ARG");} fn frame_of(v:&Value)->Result<String,&'static str>{let o=v.as_object().ok_or("TYPE")?; name(o.get("f").ok_or("ARG")?)} Ok(json!(frame_of(&args[0])?==frame_of(&args[1])?))
}
