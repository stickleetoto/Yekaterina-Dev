use serde_json::{json, Value};

const MAX_DIM: usize = 64;
const EPS: f64 = 1e-12;

type Mat = Vec<Vec<f64>>;

pub fn execute(op: &str, args: &[Value]) -> Option<Result<Value, &'static str>> {
    if op.starts_with("linalg.") { Some(run(op, args)) } else { None }
}

fn run(op: &str, args: &[Value]) -> Result<Value, &'static str> {
    match op {
        "linalg.eigen_symmetric" => eigen_symmetric_op(args, true),
        "linalg.eigenvalues_symmetric" => eigen_symmetric_op(args, false),
        "linalg.svd" => svd_op(args, true),
        "linalg.singular_values" => svd_op(args, false),
        "linalg.pinv" => pinv_op(args),
        "linalg.condition_number" => condition_number(args),
        "linalg.rank" => rank_op(args),
        "linalg.nullity" => nullity_op(args),
        "linalg.pca" => pca_op(args),
        "linalg.covariance" => covariance_op(args),
        "linalg.least_squares" => least_squares(args),
        "linalg.project" => project(args),
        "linalg.gram" => gram(args),
        "linalg.orthogonality_error" => orthogonality_error_op(args),
        "linalg.reconstruction_error" => reconstruction_error(args),
        "linalg.moore_penrose_error" => moore_penrose_error(args),
        "linalg.power_iteration" => power_iteration(args),
        "linalg.rayleigh_quotient" => rayleigh_quotient(args),
        "linalg.spectral_norm" => spectral_norm(args),
        "linalg.center_columns" => center_columns_op(args),
        _ => Err("OP"),
    }
}

fn num(v: &Value) -> Result<f64, &'static str> {
    let x = v.as_f64().ok_or("TYPE")?;
    if !x.is_finite() { return Err("NONFINITE"); }
    Ok(x)
}

fn mat(v: &Value) -> Result<Mat, &'static str> {
    let rows = v.as_array().ok_or("TYPE")?;
    if rows.is_empty() || rows.len() > MAX_DIM { return Err("SHAPE"); }
    let first = rows[0].as_array().ok_or("TYPE")?;
    if first.is_empty() || first.len() > MAX_DIM { return Err("SHAPE"); }
    let n = first.len();
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let r = row.as_array().ok_or("TYPE")?;
        if r.len() != n { return Err("SHAPE"); }
        out.push(r.iter().map(num).collect::<Result<Vec<_>, _>>()?);
    }
    Ok(out)
}

fn vecf(v: &Value) -> Result<Vec<f64>, &'static str> {
    let a = v.as_array().ok_or("TYPE")?;
    if a.is_empty() || a.len() > MAX_DIM { return Err("SHAPE"); }
    a.iter().map(num).collect()
}

fn square(a: &Mat) -> Result<usize, &'static str> {
    let n = a.len();
    if a.iter().any(|r| r.len() != n) { return Err("SHAPE"); }
    Ok(n)
}

fn transpose(a: &Mat) -> Mat {
    let m = a.len(); let n = a[0].len();
    let mut t = vec![vec![0.0; m]; n];
    for i in 0..m { for j in 0..n { t[j][i] = a[i][j]; } }
    t
}

fn mul(a: &Mat, b: &Mat) -> Result<Mat, &'static str> {
    if a[0].len() != b.len() { return Err("SHAPE"); }
    let m = a.len(); let k = b.len(); let n = b[0].len();
    let mut o = vec![vec![0.0; n]; m];
    for i in 0..m { for p in 0..k { let ap = a[i][p]; for j in 0..n { o[i][j] += ap * b[p][j]; } } }
    Ok(o)
}

fn mat_vec(a: &Mat, x: &[f64]) -> Result<Vec<f64>, &'static str> {
    if a[0].len() != x.len() { return Err("SHAPE"); }
    Ok(a.iter().map(|r| r.iter().zip(x).map(|(u,v)|u*v).sum()).collect())
}

fn identity(n: usize) -> Mat {
    let mut a=vec![vec![0.0;n];n]; for (i,row) in a.iter_mut().enumerate(){row[i]=1.0;} a
}

fn frob(a: &Mat) -> f64 { a.iter().flatten().fold(0.0, |acc, x| acc.hypot(*x)) }
fn norm(x: &[f64]) -> f64 { x.iter().fold(0.0, |acc, v| acc.hypot(*v)) }
fn dot(a:&[f64], b:&[f64])->f64 { a.iter().zip(b).map(|(x,y)|x*y).sum() }
fn max_abs(a:&Mat)->f64 { a.iter().flatten().fold(0.0, |m,x| m.max(x.abs())) }

fn symmetric(a:&Mat, tol:f64)->bool {
    a.iter().enumerate().all(|(i,r)| r.iter().enumerate().all(|(j,x)| (*x-a[j][i]).abs()<=tol))
}

fn jacobi_eigen(a: &Mat, tol: f64, max_iter: usize) -> Result<(Vec<f64>, Mat, usize), &'static str> {
    let n=square(a)?;
    if !symmetric(a, tol.max(1e-10)) { return Err("DOMAIN"); }
    let mut d=a.clone(); let mut v=identity(n);
    for iter in 0..max_iter {
        let mut p=0usize; let mut q=0usize; let mut best=0.0;
        for i in 0..n { for j in i+1..n { let z=d[i][j].abs(); if z>best {best=z;p=i;q=j;} } }
        if best <= tol {
            let mut vals=(0..n).map(|i| d[i][i]).collect::<Vec<_>>();
            let mut idx=(0..n).collect::<Vec<_>>();
            idx.sort_by(|&i,&j| vals[j].total_cmp(&vals[i]));
            let sorted=idx.iter().map(|&i| vals[i]).collect::<Vec<_>>();
            let mut vv=vec![vec![0.0;n];n];
            for (newc,&oldc) in idx.iter().enumerate(){for r in 0..n{vv[r][newc]=v[r][oldc];}}
            vals=sorted;
            return Ok((vals,vv,iter));
        }
        let app=d[p][p]; let aqq=d[q][q]; let apq=d[p][q];
        let phi=0.5*(2.0*apq).atan2(aqq-app); let c=phi.cos(); let s=phi.sin();
        for k in 0..n { if k!=p && k!=q {
            let dkp=d[k][p]; let dkq=d[k][q];
            d[k][p]=c*dkp-s*dkq; d[p][k]=d[k][p];
            d[k][q]=s*dkp+c*dkq; d[q][k]=d[k][q];
        }}
        d[p][p]=c*c*app-2.0*s*c*apq+s*s*aqq;
        d[q][q]=s*s*app+2.0*s*c*apq+c*c*aqq;
        d[p][q]=0.0; d[q][p]=0.0;
        for k in 0..n { let vkp=v[k][p]; let vkq=v[k][q]; v[k][p]=c*vkp-s*vkq; v[k][q]=s*vkp+c*vkq; }
    }
    Err("NO_CONVERGE")
}

fn eigen_symmetric_op(args:&[Value], vectors:bool)->Result<Value,&'static str>{
    if args.is_empty() || args.len()>3 {return Err("ARG");}
    let a=mat(&args[0])?; square(&a)?;
    let tol=if args.len()>1{num(&args[1])?}else{1e-12};
    let max=if args.len()>2{args[2].as_u64().ok_or("TYPE")?.min(100_000) as usize}else{10_000};
    if tol<=0.0 || max==0 {return Err("DOMAIN");}
    let (values,v,it)=jacobi_eigen(&a,tol,max)?;
    if vectors {Ok(json!({"values":values,"vectors":v,"iterations":it}))} else {Ok(json!(values))}
}

fn svd_core(a:&Mat, tol:f64)->Result<(Mat,Vec<f64>,Mat,f64),&'static str>{
    if tol<=0.0 { return Err("DOMAIN"); }
    let m=a.len(); let n=a[0].len(); let r=n;
    let scale=max_abs(a);
    if scale==0.0 { return Ok((vec![vec![0.0;r];m],vec![0.0;r],transpose(&identity(n)),0.0)); }
    let scaled=a.iter().map(|row|row.iter().map(|x|x/scale).collect::<Vec<_>>()).collect::<Mat>();
    let at=transpose(&scaled); let ata=mul(&at,&scaled)?;
    let jacobi_tol=tol.max(1e-14);
    let (evals,v,_)=jacobi_eigen(&ata,jacobi_tol,20_000)?;
    let ss=evals.iter().map(|x| if *x<=0.0 {0.0}else{x.sqrt()}).collect::<Vec<_>>();
    let s=ss.iter().map(|x|x*scale).collect::<Vec<_>>();
    let ssmax=ss.iter().copied().fold(0.0,f64::max);
    let cut=tol*ssmax;
    let mut u=vec![vec![0.0;r];m];
    for j in 0..r {
        if ss[j] > cut {
            let col=(0..n).map(|i|v[i][j]).collect::<Vec<_>>();
            let av=mat_vec(&scaled,&col)?;
            for i in 0..m {u[i][j]=av[i]/ss[j];}
        }
    }
    let vt=transpose(&v);
    let mut us=vec![vec![0.0;r];m]; for i in 0..m{for j in 0..r{us[i][j]=u[i][j]*ss[j];}}
    let recon=mul(&us,&vt)?;
    let mut diff=scaled.clone(); for i in 0..m{for j in 0..n{diff[i][j]-=recon[i][j];}}
    let residual=frob(&diff)/frob(&scaled).max(EPS);
    Ok((u,s,vt,residual))
}

fn svd_op(args:&[Value], full:bool)->Result<Value,&'static str>{
    if args.is_empty()||args.len()>2{return Err("ARG");} let a=mat(&args[0])?; let tol=if args.len()==2{num(&args[1])?}else{1e-10}; if tol<=0.0{return Err("DOMAIN");}
    let (u,s,vt,residual)=svd_core(&a,tol)?;
    if full {Ok(json!({"u":u,"s":s,"vt":vt,"residual":residual}))} else {Ok(json!(s))}
}

fn pinv_core(a:&Mat,tol:f64)->Result<Mat,&'static str>{
    let (u,s,vt,_)=svd_core(a,tol)?; let v=transpose(&vt); let ut=transpose(&u); let n=v.len(); let m=ut[0].len(); let r=s.len();
    let mut o=vec![vec![0.0;m];n];
    let smax=s.iter().copied().fold(0.0,f64::max); let cut=tol*smax;
    for i in 0..n{for j in 0..m{let mut z=0.0;for k in 0..r{if s[k]>cut{z+=v[i][k]*(1.0/s[k])*ut[k][j];}}o[i][j]=z;}}
    Ok(o)
}
fn pinv_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let a=mat(&args[0])?;let tol=if args.len()==2{num(&args[1])?}else{1e-10};Ok(json!(pinv_core(&a,tol)?))}
fn condition_number(args:&[Value])->Result<Value,&'static str>{
    if args.is_empty()||args.len()>2{return Err("ARG");}
    let a=mat(&args[0])?;let tol=if args.len()==2{num(&args[1])?}else{1e-12};if tol<=0.0{return Err("DOMAIN");}
    let (_,s,_,_)=svd_core(&a,tol)?;let smax=s.iter().copied().fold(0.0,f64::max);
    if smax==0.0{return Ok(json!(null));}
    let cut=tol*smax;let rank=s.iter().filter(|&&x|x>cut).count();let full_rank=a.len().min(a[0].len());
    if rank<full_rank{return Ok(json!(null));}
    let smin=s.iter().copied().filter(|x|*x>cut).fold(f64::INFINITY,f64::min);
    if !smin.is_finite()||smin==0.0{return Ok(json!(null));}Ok(json!(smax/smin))
}
fn rank_core(a:&Mat,tol:f64)->Result<usize,&'static str>{if tol<=0.0{return Err("DOMAIN");}let(_,s,_,_)=svd_core(a,tol)?;let m=s.iter().copied().fold(0.0,f64::max);if m==0.0{return Ok(0);}Ok(s.iter().filter(|&&x|x>tol*m).count())}
fn rank_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let a=mat(&args[0])?;let tol=if args.len()==2{num(&args[1])?}else{1e-10};Ok(json!(rank_core(&a,tol)?))}
fn nullity_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let a=mat(&args[0])?;let tol=if args.len()==2{num(&args[1])?}else{1e-10};Ok(json!(a[0].len()-rank_core(&a,tol)?))}

fn center_columns(a:&Mat)->(Mat,Vec<f64>){let m=a.len();let n=a[0].len();let mut means=vec![0.0;n];for row in a{for j in 0..n{means[j]+=row[j];}}for x in &mut means{*x/=m as f64;}let mut c=a.clone();for row in &mut c{for j in 0..n{row[j]-=means[j];}}(c,means)}
fn center_columns_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let a=mat(&args[0])?;let(c,m)=center_columns(&a);Ok(json!({"data":c,"mean":m}))}
fn covariance_core(a:&Mat)->Result<Mat,&'static str>{if a.len()<2{return Err("SHAPE");}let(c,_)=center_columns(a);let ct=transpose(&c);let mut o=mul(&ct,&c)?;let d=(a.len()-1)as f64;for r in &mut o{for x in r{*x/=d;}}Ok(o)}
fn covariance_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}Ok(json!(covariance_core(&mat(&args[0])?)?))}
fn pca_op(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let a=mat(&args[0])?;let k=if args.len()==2{args[1].as_u64().ok_or("TYPE")? as usize}else{a[0].len()};if k==0||k>a[0].len(){return Err("SHAPE");}let cov=covariance_core(&a)?;let(vals,vecs,_)=jacobi_eigen(&cov,1e-12,20_000)?;let total=vals.iter().map(|x|x.max(0.0)).sum::<f64>();let ratios=vals.iter().take(k).map(|x|if total==0.0{0.0}else{x.max(0.0)/total}).collect::<Vec<_>>();let components=(0..k).map(|j|(0..vecs.len()).map(|i|vecs[i][j]).collect::<Vec<_>>()).collect::<Vec<_>>();Ok(json!({"values":vals[..k].to_vec(),"ratio":ratios,"components":components}))}
fn least_squares(args:&[Value])->Result<Value,&'static str>{if args.len()<2||args.len()>3{return Err("ARG");}let a=mat(&args[0])?;let b=vecf(&args[1])?;if a.len()!=b.len(){return Err("SHAPE");}let tol=if args.len()==3{num(&args[2])?}else{1e-10};let p=pinv_core(&a,tol)?;let x=mat_vec(&p,&b)?;let ax=mat_vec(&a,&x)?;let res=ax.iter().zip(&b).map(|(u,v)|(u-v)*(u-v)).sum::<f64>().sqrt();Ok(json!({"x":x,"residual":res}))}
fn project(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vecf(&args[0])?;let b=vecf(&args[1])?;if x.len()!=b.len(){return Err("SHAPE");}let bb=dot(&b,&b);if bb<=EPS{return Err("DIV0");}let s=dot(&x,&b)/bb;Ok(json!(b.iter().map(|v|s*v).collect::<Vec<_>>()))}
fn gram(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let a=mat(&args[0])?;Ok(json!(mul(&transpose(&a),&a)?))}
fn orthogonality_error(a:&Mat)->Result<f64,&'static str>{let at=transpose(a);let g=mul(&at,a)?;let n=g.len();let mut d=g;for i in 0..n{d[i][i]-=1.0;}Ok(frob(&d))}
fn orthogonality_error_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}Ok(json!(orthogonality_error(&mat(&args[0])?)?))}
fn reconstruction_error(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=mat(&args[0])?;let b=mat(&args[1])?;if a.len()!=b.len()||a[0].len()!=b[0].len(){return Err("SHAPE");}let mut d=a.clone();for i in 0..a.len(){for j in 0..a[0].len(){d[i][j]-=b[i][j];}}Ok(json!(frob(&d)/frob(&a).max(EPS)))}
fn matrix_diff_norm(a:&Mat,b:&Mat)->Result<f64,&'static str>{if a.len()!=b.len()||a[0].len()!=b[0].len(){return Err("SHAPE");}let mut d=a.clone();for i in 0..a.len(){for j in 0..a[0].len(){d[i][j]-=b[i][j];}}Ok(frob(&d))}
fn symmetry_error(a:&Mat)->Result<f64,&'static str>{matrix_diff_norm(a,&transpose(a))}
fn moore_penrose_error(args:&[Value])->Result<Value,&'static str>{
    if args.is_empty()||args.len()>2{return Err("ARG");}
    let a=mat(&args[0])?;let tol=if args.len()==2{num(&args[1])?}else{1e-10};if tol<=0.0{return Err("DOMAIN");}
    let p=pinv_core(&a,tol)?;let ap=mul(&a,&p)?;let pa=mul(&p,&a)?;let apa=mul(&ap,&a)?;let pap=mul(&pa,&p)?;
    Ok(json!({"a_pa":matrix_diff_norm(&a,&apa)?,"p_ap":matrix_diff_norm(&p,&pap)?,"ap_sym":symmetry_error(&ap)?,"pa_sym":symmetry_error(&pa)?}))
}
fn power_iteration(args:&[Value])->Result<Value,&'static str>{
    if args.is_empty()||args.len()>3{return Err("ARG");}
    let a=mat(&args[0])?;let n=square(&a)?;let tol=if args.len()>1{num(&args[1])?}else{1e-10};let max=if args.len()>2{args[2].as_u64().ok_or("TYPE")?.min(100_000)as usize}else{10_000};
    if tol<=0.0||max==0{return Err("DOMAIN");}
    if symmetric(&a,tol.max(1e-10)){
        let(values,vectors,it)=jacobi_eigen(&a,tol,max)?;
        let idx=(0..values.len()).max_by(|&i,&j|values[i].abs().total_cmp(&values[j].abs())).ok_or("DOMAIN")?;
        let max_abs=values[idx].abs();
        let tied=values.iter().filter(|v|(v.abs()-max_abs).abs()<=tol*(1.0+max_abs)).count();
        if tied>1{return Err("NO_CONVERGE");}
        let vector=(0..n).map(|r|vectors[r][idx]).collect::<Vec<_>>();let value=values[idx];let ax=mat_vec(&a,&vector)?;let residual=norm(&ax.iter().zip(&vector).map(|(u,w)|u-value*w).collect::<Vec<_>>());
        if residual>tol*(1.0+value.abs()){return Err("NO_CONVERGE");}
        return Ok(json!({"value":value,"vector":vector,"residual":residual,"iterations":it}));
    }
    let mut x=(0..n).map(|i|(i+1)as f64).collect::<Vec<_>>();let nx=norm(&x);for v in &mut x{*v/=nx;}
    for it in 0..max{let y=mat_vec(&a,&x)?;let ny=norm(&y);if ny<=EPS{return Err("DOMAIN");}let xn=y.iter().map(|v|v/ny).collect::<Vec<_>>();let ax=mat_vec(&a,&xn)?;let ln=dot(&xn,&ax);let res=norm(&ax.iter().zip(&xn).map(|(u,v)|u-ln*v).collect::<Vec<_>>());if res<=tol*(1.0+ln.abs()){return Ok(json!({"value":ln,"vector":xn,"residual":res,"iterations":it+1}));}x=xn;}Err("NO_CONVERGE")
}
fn rayleigh_quotient(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=mat(&args[0])?;square(&a)?;let x=vecf(&args[1])?;if x.len()!=a.len(){return Err("SHAPE");}let den=dot(&x,&x);if den<=EPS{return Err("DIV0");}Ok(json!(dot(&x,&mat_vec(&a,&x)?)/den))}
fn spectral_norm(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let a=mat(&args[0])?;let(_,s,_,_)=svd_core(&a,1e-12)?;Ok(json!(s.first().copied().unwrap_or(0.0)))}
