use serde_json::{json, Value};

const MAX_DIM: usize = 128;
const MAX_ELEMS: usize = 20_000;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    if matches!(op,
        "mat.rank"|"mat.solve"|"mat.hadamard"|"mat.norm1"|"mat.norm_inf"|"mat.max_abs"|
        "mat.row_mean"|"mat.col_mean"|"mat.is_square"|"mat.is_symmetric"|"mat.is_diagonal"|
        "mat.is_identity"|"mat.is_upper_triangular"|"mat.is_lower_triangular"|"mat.minor"|
        "mat.cofactor"|"mat.adjugate"|"mat.power"|"mat.lu"|"mat.cholesky"|"mat.qr"|
        "mat.diag_extract"|"mat.flatten"|"mat.reshape"|"mat.concat_rows"|"mat.concat_cols"|
        "mat.row"|"mat.col"|"mat.swap_rows"|"mat.swap_cols"|"mat.condition_inf"|"mat.mean"
    ){Some(run(op,args))}else{None}
}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "mat.rank"=>rank_op(args),
    "mat.solve"=>solve_op(args),
    "mat.hadamard"=>binary(args,|a,b|a*b),
    "mat.norm1"=>{let m=one(args)?;let mut best=0.0f64;for j in 0..m[0].len(){let s=m.iter().map(|r|r[j].abs()).sum::<f64>();best=best.max(s);}finite(best)},
    "mat.norm_inf"=>{let m=one(args)?;finite(m.iter().map(|r|r.iter().map(|x|x.abs()).sum::<f64>()).fold(0.0,f64::max))},
    "mat.max_abs"=>{let m=one(args)?;finite(m.iter().flatten().map(|x|x.abs()).fold(0.0,f64::max))},
    "mat.row_mean"=>{let m=one(args)?;out_vec(m.iter().map(|r|r.iter().sum::<f64>()/r.len() as f64).collect())},
    "mat.col_mean"=>{let m=one(args)?;let mut o=vec![0.0;m[0].len()];for r in &m{for(j,x)in r.iter().enumerate(){o[j]+=x/m.len() as f64;}}out_vec(o)},
    "mat.is_square"=>{let m=one(args)?;Ok(json!(m.len()==m[0].len()))},
    "mat.is_symmetric"=>property_tol(args,"sym"),
    "mat.is_diagonal"=>property_tol(args,"diag"),
    "mat.is_identity"=>property_tol(args,"id"),
    "mat.is_upper_triangular"=>property_tol(args,"upper"),
    "mat.is_lower_triangular"=>property_tol(args,"lower"),
    "mat.minor"=>minor_op(args),
    "mat.cofactor"=>cofactor_op(args),
    "mat.adjugate"=>adjugate_op(args),
    "mat.power"=>power_op(args),
    "mat.lu"=>lu_op(args),
    "mat.cholesky"=>cholesky_op(args),
    "mat.qr"=>qr_op(args),
    "mat.diag_extract"=>{let m=one(args)?;let n=m.len().min(m[0].len());out_vec((0..n).map(|i|m[i][i]).collect())},
    "mat.flatten"=>{let m=one(args)?;out_vec(m.into_iter().flatten().collect())},
    "mat.reshape"=>reshape_op(args),
    "mat.concat_rows"=>concat_rows(args),
    "mat.concat_cols"=>concat_cols(args),
    "mat.row"=>row_col(args,true),
    "mat.col"=>row_col(args,false),
    "mat.swap_rows"=>swap_axis(args,true),
    "mat.swap_cols"=>swap_axis(args,false),
    "mat.condition_inf"=>condition_inf(args),
    "mat.mean"=>{let m=one(args)?;let n=m.len()*m[0].len();finite(m.iter().flatten().sum::<f64>()/n as f64)},
    _=>Err("OP")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn matrix(v:&Value)->Result<Vec<Vec<f64>>,&'static str>{let rows=v.as_array().ok_or("TYPE")?;if rows.is_empty()||rows.len()>MAX_DIM{return Err("SHAPE");}let mut out=Vec::with_capacity(rows.len());let mut cols=None;let mut count=0usize;for r in rows{let a=r.as_array().ok_or("TYPE")?;if a.is_empty()||a.len()>MAX_DIM{return Err("SHAPE");}if let Some(c)=cols{if c!=a.len(){return Err("SHAPE");}}else{cols=Some(a.len());}count+=a.len();if count>MAX_ELEMS{return Err("LIMIT");}out.push(a.iter().map(num).collect::<Result<Vec<_>,_>>()?);}Ok(out)}
fn one(args:&[Value])->Result<Vec<Vec<f64>>,&'static str>{if args.len()!=1{return Err("ARG");}matrix(&args[0])}
fn out(m:Vec<Vec<f64>>)->Result<Value,&'static str>{if m.iter().flatten().all(|x|x.is_finite()){Ok(json!(m))}else{Err("NONFINITE")}}
fn out_vec(v:Vec<f64>)->Result<Value,&'static str>{if v.iter().all(|x|x.is_finite()){Ok(json!(v))}else{Err("NONFINITE")}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn binary<F:Fn(f64,f64)->f64>(args:&[Value],f:F)->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=matrix(&args[0])?;let b=matrix(&args[1])?;if a.len()!=b.len()||a[0].len()!=b[0].len(){return Err("SHAPE");}out(a.into_iter().zip(b).map(|(ra,rb)|ra.into_iter().zip(rb).map(|(x,y)|f(x,y)).collect()).collect())}
fn rank_op(args:&[Value])->Result<Value,&'static str>{let mut a=one(args)?;let rows=a.len();let cols=a[0].len();let mut rank=0usize;let mut c=0usize;while rank<rows&&c<cols{let mut p=rank;for r in rank..rows{if a[r][c].abs()>a[p][c].abs(){p=r;}}if a[p][c].abs()<=1e-12{c+=1;continue;}a.swap(rank,p);let pv=a[rank][c];for j in c..cols{a[rank][j]/=pv;}for r in 0..rows{if r!=rank{let f=a[r][c];for j in c..cols{a[r][j]-=f*a[rank][j];}}}rank+=1;c+=1;}Ok(json!(rank))}
fn solve_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let mut a=matrix(&args[0])?;let b=args[1].as_array().ok_or("TYPE")?;let n=a.len();if n!=a[0].len()||b.len()!=n||n>64{return Err("SHAPE");}for i in 0..n{a[i].push(num(&b[i])?);}for i in 0..n{let mut p=i;for r in i+1..n{if a[r][i].abs()>a[p][i].abs(){p=r;}}if a[p][i].abs()<=1e-15{return Err("SINGULAR");}a.swap(i,p);let pv=a[i][i];for c in i..=n{a[i][c]/=pv;}for r in 0..n{if r!=i{let f=a[r][i];for c in i..=n{a[r][c]-=f*a[i][c];}}}}out_vec(a.into_iter().map(|r|r[n]).collect())}
fn property_tol(args:&[Value],mode:&str)->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let m=matrix(&args[0])?;let eps=if args.len()==2{num(&args[1])?}else{1e-12};if eps<0.0{return Err("DOMAIN");}let square=m.len()==m[0].len();if !square&&mode!="diag"{return Ok(json!(false));}let mut ok=true;for i in 0..m.len(){for j in 0..m[0].len(){let target=match mode{"sym"=>if j<m.len(){m[j][i]}else{m[i][j]+eps*2.0},"diag"=>if i==j{m[i][j]}else{0.0},"id"=>if i==j{1.0}else{0.0},"upper"=>if i>j{0.0}else{m[i][j]},"lower"=>if j>i{0.0}else{m[i][j]},_=>m[i][j]};if (m[i][j]-target).abs()>eps{ok=false;break;}}if !ok{break;}}Ok(json!(ok))}
fn remove_rc(m:&[Vec<f64>],rr:usize,cc:usize)->Vec<Vec<f64>>{m.iter().enumerate().filter(|(i,_)|*i!=rr).map(|(_,r)|r.iter().enumerate().filter(|(j,_)|*j!=cc).map(|(_,x)|*x).collect()).collect()}
fn determinant(mut a:Vec<Vec<f64>>)->f64{let n=a.len();if n==0{return 1.0;}let mut d=1.0;for i in 0..n{let mut p=i;for r in i+1..n{if a[r][i].abs()>a[p][i].abs(){p=r;}}if a[p][i].abs()<=1e-15{return 0.0;}if p!=i{a.swap(i,p);d=-d;}let pv=a[i][i];d*=pv;for r in i+1..n{let f=a[r][i]/pv;for c in i+1..n{a[r][c]-=f*a[i][c];}}}d}
fn minor_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let m=matrix(&args[0])?;if m.len()!=m[0].len()||m.len()<2{return Err("SHAPE");}let r=args[1].as_u64().ok_or("TYPE")? as usize;let c=args[2].as_u64().ok_or("TYPE")? as usize;if r>=m.len()||c>=m.len(){return Err("DOMAIN");}finite(determinant(remove_rc(&m,r,c)))}
fn cofactor_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let m=matrix(&args[0])?;if m.len()!=m[0].len()||m.len()<2{return Err("SHAPE");}let r=args[1].as_u64().ok_or("TYPE")? as usize;let c=args[2].as_u64().ok_or("TYPE")? as usize;if r>=m.len()||c>=m.len(){return Err("DOMAIN");}finite(if (r+c)%2==0{1.0}else{-1.0}*determinant(remove_rc(&m,r,c)))}
fn adjugate_op(args:&[Value])->Result<Value,&'static str>{let m=one(args)?;let n=m.len();if n!=m[0].len()||n>12{return Err("SHAPE");}if n==1{return Ok(json!([[1.0]]));}let mut o=vec![vec![0.0;n];n];for i in 0..n{for j in 0..n{o[j][i]=(if(i+j)%2==0{1.0}else{-1.0})*determinant(remove_rc(&m,i,j));}}out(o)}
fn identity(n:usize)->Vec<Vec<f64>>{let mut m=vec![vec![0.0;n];n];for i in 0..n{m[i][i]=1.0;}m}
fn mul(a:&[Vec<f64>],b:&[Vec<f64>])->Vec<Vec<f64>>{let mut o=vec![vec![0.0;b[0].len()];a.len()];for i in 0..a.len(){for k in 0..a[0].len(){for j in 0..b[0].len(){o[i][j]+=a[i][k]*b[k][j];}}}o}
fn inverse(a:Vec<Vec<f64>>)->Result<Vec<Vec<f64>>,&'static str>{let n=a.len();if n!=a[0].len(){return Err("SHAPE");}let mut aug=vec![vec![0.0;2*n];n];for i in 0..n{for j in 0..n{aug[i][j]=a[i][j];}aug[i][n+i]=1.0;}for i in 0..n{let mut p=i;for r in i+1..n{if aug[r][i].abs()>aug[p][i].abs(){p=r;}}if aug[p][i].abs()<=1e-15{return Err("SINGULAR");}aug.swap(i,p);let pv=aug[i][i];for c in 0..2*n{aug[i][c]/=pv;}for r in 0..n{if r!=i{let f=aug[r][i];for c in 0..2*n{aug[r][c]-=f*aug[i][c];}}}}Ok((0..n).map(|i|aug[i][n..].to_vec()).collect())}
fn power_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let mut a=matrix(&args[0])?;let n=a.len();if n!=a[0].len()||n>32{return Err("SHAPE");}let e=args[1].as_i64().ok_or("TYPE")?;if e<0{a=inverse(a)?;}let mut k=e.unsigned_abs();let mut r=identity(n);while k>0{if k&1==1{r=mul(&r,&a);}k>>=1;if k>0{a=mul(&a,&a);}}out(r)}
fn lu_op(args:&[Value])->Result<Value,&'static str>{let mut u=one(args)?;let n=u.len();if n!=u[0].len()||n>64{return Err("SHAPE");}let mut l=identity(n);let mut p: Vec<usize> =(0..n).collect();for k in 0..n{let mut piv=k;for r in k+1..n{if u[r][k].abs()>u[piv][k].abs(){piv=r;}}if u[piv][k].abs()<=1e-15{return Err("SINGULAR");}if piv!=k{u.swap(k,piv);p.swap(k,piv);for j in 0..k{let t=l[k][j];l[k][j]=l[piv][j];l[piv][j]=t;}}for i in k+1..n{let f=u[i][k]/u[k][k];l[i][k]=f;for j in k..n{u[i][j]-=f*u[k][j];}}}Ok(json!({"l":l,"u":u,"p":p}))}
fn cholesky_op(args:&[Value])->Result<Value,&'static str>{let a=one(args)?;let n=a.len();if n!=a[0].len()||n>64{return Err("SHAPE");}for i in 0..n{for j in 0..n{if (a[i][j]-a[j][i]).abs()>1e-10{return Err("DOMAIN");}}}let mut l=vec![vec![0.0;n];n];for i in 0..n{for j in 0..=i{let mut s=a[i][j];for k in 0..j{s-=l[i][k]*l[j][k];}if i==j{if s<=0.0{return Err("DOMAIN");}l[i][j]=s.sqrt();}else{l[i][j]=s/l[j][j];}}}out(l)}
fn qr_op(args:&[Value])->Result<Value,&'static str>{let a=one(args)?;let rows=a.len();let cols=a[0].len();if cols>rows||rows>128{return Err("SHAPE");}let mut qcols:Vec<Vec<f64>>=Vec::new();let mut r=vec![vec![0.0;cols];cols];for j in 0..cols{let mut v=(0..rows).map(|i|a[i][j]).collect::<Vec<_>>();for i in 0..j{let dot=v.iter().zip(qcols[i].iter()).map(|(x,y)|x*y).sum::<f64>();r[i][j]=dot;for k in 0..rows{v[k]-=dot*qcols[i][k];}}let norm=v.iter().map(|x|x*x).sum::<f64>().sqrt();if norm<=1e-14{return Err("SINGULAR");}r[j][j]=norm;for x in &mut v{*x/=norm;}qcols.push(v);}let mut q=vec![vec![0.0;cols];rows];for j in 0..cols{for i in 0..rows{q[i][j]=qcols[j][i];}}Ok(json!({"q":q,"r":r}))}
fn reshape_op(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let vals=args[0].as_array().ok_or("TYPE")?;let r=args[1].as_u64().ok_or("TYPE")? as usize;let c=args[2].as_u64().ok_or("TYPE")? as usize;if r==0||c==0||r>MAX_DIM||c>MAX_DIM||r*c!=vals.len(){return Err("SHAPE");}let xs=vals.iter().map(num).collect::<Result<Vec<_>,_>>()?;out((0..r).map(|i|xs[i*c..(i+1)*c].to_vec()).collect())}
fn concat_rows(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let mut a=matrix(&args[0])?;let b=matrix(&args[1])?;if a[0].len()!=b[0].len(){return Err("SHAPE");}a.extend(b);if a.len()>MAX_DIM{return Err("LIMIT");}out(a)}
fn concat_cols(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let mut a=matrix(&args[0])?;let b=matrix(&args[1])?;if a.len()!=b.len(){return Err("SHAPE");}for(i,r)in b.into_iter().enumerate(){a[i].extend(r);if a[i].len()>MAX_DIM{return Err("LIMIT");}}out(a)}
fn row_col(args:&[Value],row:bool)->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let m=matrix(&args[0])?;let i=args[1].as_u64().ok_or("TYPE")? as usize;if row{if i>=m.len(){return Err("DOMAIN");}out_vec(m[i].clone())}else{if i>=m[0].len(){return Err("DOMAIN");}out_vec(m.iter().map(|r|r[i]).collect())}}
fn swap_axis(args:&[Value],rows:bool)->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let mut m=matrix(&args[0])?;let a=args[1].as_u64().ok_or("TYPE")? as usize;let b=args[2].as_u64().ok_or("TYPE")? as usize;if rows{if a>=m.len()||b>=m.len(){return Err("DOMAIN");}m.swap(a,b);}else{if a>=m[0].len()||b>=m[0].len(){return Err("DOMAIN");}for r in &mut m{r.swap(a,b);}}out(m)}
fn condition_inf(args:&[Value])->Result<Value,&'static str>{let a=one(args)?;let inv=inverse(a.clone())?;let norm=|m:&Vec<Vec<f64>>|m.iter().map(|r|r.iter().map(|x|x.abs()).sum::<f64>()).fold(0.0,f64::max);finite(norm(&a)*norm(&inv))}
