use std::collections::BTreeMap;
use serde_json::{json, Value};

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{
    if op.starts_with("reg.")||op.starts_with("test.")||matches!(op,
        "stat.trimmed_mean"|"stat.winsorized_mean"|"stat.median_abs_deviation"|"stat.quartiles"|
        "stat.five_number"|"stat.unique_count"|"stat.frequency"|"stat.entropy"|"stat.gini_impurity"|
        "stat.gini_coefficient"|"stat.ranks"|"stat.spearman"|"stat.autocorr"|"stat.lag"|
        "stat.residuals"|"stat.mae"|"stat.mse"|"stat.rmse"|"stat.mape"|"stat.sse"|
        "stat.sample_covariance"|"stat.pooled_mean"|"stat.pooled_variance"|"stat.quantile"|
        "stat.ecdf"|"stat.outlier_iqr"|"stat.outlier_z"|"stat.center"|"stat.standardize_sample"
    ){Some(run(op,args))}else{None}
}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "reg.linear_slope"=>linear_component(args,"slope"),
    "reg.linear_intercept"=>linear_component(args,"intercept"),
    "reg.linear_r2"=>linear_component(args,"r2"),
    "reg.linear"=>linear_full(args),
    "reg.predict"=>predict(args),
    "reg.residuals"=>reg_residuals(args),
    "reg.sse"=>reg_metric(args,"sse"),
    "reg.mse"=>reg_metric(args,"mse"),
    "reg.rmse"=>reg_metric(args,"rmse"),
    "reg.mae"=>reg_metric(args,"mae"),
    "reg.origin_slope"=>origin_slope(args),
    "reg.weighted_linear"=>weighted_linear(args),
    "reg.correlation"=>pair_corr(args),
    "reg.standard_error"=>reg_standard_error(args),
    "test.z_one_sample"=>z_one_sample(args),
    "test.t_one_sample"=>t_one_sample(args),
    "test.t_two_equal"=>t_two_equal(args),
    "test.t_welch"=>t_welch(args),
    "test.chi_square_gof"=>chi_square_gof(args),
    "test.chi_square_independence"=>chi_square_independence(args),
    "test.f_variance_ratio"=>f_variance_ratio(args),
    "test.cohen_d"=>cohen_d(args),
    "test.proportion_z"=>proportion_z(args),
    "test.mean_diff"=>mean_diff(args),
    "stat.trimmed_mean"=>trimmed_mean(args),
    "stat.winsorized_mean"=>winsorized_mean(args),
    "stat.median_abs_deviation"=>median_abs_dev(args),
    "stat.quartiles"=>quartiles(args),
    "stat.five_number"=>five_number(args),
    "stat.unique_count"=>unique_count(args),
    "stat.frequency"=>frequency(args),
    "stat.entropy"=>entropy(args),
    "stat.gini_impurity"=>gini_impurity(args),
    "stat.gini_coefficient"=>gini_coefficient(args),
    "stat.ranks"=>ranks_op(args),
    "stat.spearman"=>spearman(args),
    "stat.autocorr"=>autocorr(args),
    "stat.lag"=>lag(args),
    "stat.residuals"=>residuals(args),
    "stat.mae"=>metric_two(args,"mae"),
    "stat.mse"=>metric_two(args,"mse"),
    "stat.rmse"=>metric_two(args,"rmse"),
    "stat.mape"=>metric_two(args,"mape"),
    "stat.sse"=>metric_two(args,"sse"),
    "stat.sample_covariance"=>sample_cov(args),
    "stat.pooled_mean"=>pooled_mean(args),
    "stat.pooled_variance"=>pooled_variance(args),
    "stat.quantile"=>quantile(args),
    "stat.ecdf"=>ecdf(args),
    "stat.outlier_iqr"=>outlier_iqr(args),
    "stat.outlier_z"=>outlier_z(args),
    "stat.center"=>center(args),
    "stat.standardize_sample"=>standardize_sample(args),
    _=>Err("OP")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn vec(v:&Value)->Result<Vec<f64>,&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()>100_000{return Err("LIMIT");}a.iter().map(num).collect()}
fn one(args:&[Value])->Result<Vec<f64>,&'static str>{if args.len()!=1{return Err("ARG");}let x=vec(&args[0])?;if x.is_empty(){Err("EMPTY")}else{Ok(x)}}
fn pair(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let y=vec(&args[1])?;if x.is_empty()||x.len()!=y.len(){Err("SHAPE")}else{Ok((x,y))}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn mean(x:&[f64])->f64{x.iter().sum::<f64>()/x.len() as f64}
fn sample_var(x:&[f64])->Result<f64,&'static str>{if x.len()<2{return Err("EMPTY");}let m=mean(x);Ok(x.iter().map(|v|{let d=*v-m;d*d}).sum::<f64>()/(x.len()-1) as f64)}
fn median(mut x:Vec<f64>)->Result<f64,&'static str>{if x.is_empty(){return Err("EMPTY");}x.sort_by(f64::total_cmp);let n=x.len();Ok(if n%2==1{x[n/2]}else{(x[n/2-1]+x[n/2])/2.0})}
fn percentile(mut x:Vec<f64>,p:f64)->Result<f64,&'static str>{if x.is_empty(){return Err("EMPTY");}if !(0.0..=100.0).contains(&p){return Err("DOMAIN");}x.sort_by(f64::total_cmp);if x.len()==1{return Ok(x[0]);}let pos=p/100.0*(x.len()-1) as f64;let lo=pos.floor() as usize;let hi=pos.ceil() as usize;let t=pos-lo as f64;Ok(x[lo]+(x[hi]-x[lo])*t)}
fn linear_params(x:&[f64],y:&[f64])->Result<(f64,f64,f64),&'static str>{let mx=mean(x);let my=mean(y);let mut sxx=0.0;let mut sxy=0.0;let mut syy=0.0;for(a,b)in x.iter().zip(y){let dx=*a-mx;let dy=*b-my;sxx+=dx*dx;sxy+=dx*dy;syy+=dy*dy;}if sxx==0.0{return Err("DOMAIN");}let slope=sxy/sxx;let intercept=my-slope*mx;let r2=if syy==0.0{if y.iter().all(|v|(*v-my).abs()<=1e-15){1.0}else{0.0}}else{(sxy*sxy/(sxx*syy)).clamp(0.0,1.0)};Ok((slope,intercept,r2))}
fn xy(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{pair(args)}
fn linear_component(args:&[Value],part:&str)->Result<Value,&'static str>{let(x,y)=xy(args)?;let(s,i,r)=linear_params(&x,&y)?;finite(match part{"slope"=>s,"intercept"=>i,"r2"=>r,_=>return Err("OP")})}
fn linear_full(args:&[Value])->Result<Value,&'static str>{let(x,y)=xy(args)?;let(s,i,r)=linear_params(&x,&y)?;Ok(json!({"slope":s,"intercept":i,"r2":r}))}
fn predict(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let slope=num(&args[0])?;let intercept=num(&args[1])?;match &args[2]{Value::Array(_)=>Ok(json!(vec(&args[2])?.into_iter().map(|x|slope*x+intercept).collect::<Vec<_>>())),v=>finite(slope*num(v)?+intercept)}}
fn reg_residuals(args:&[Value])->Result<Value,&'static str>{let(x,y)=xy(args)?;let(s,i,_)=linear_params(&x,&y)?;Ok(json!(x.iter().zip(y).map(|(a,b)|b-(s*a+i)).collect::<Vec<_>>()))}
fn reg_metric(args:&[Value],kind:&str)->Result<Value,&'static str>{let(x,y)=xy(args)?;let(s,i,_)=linear_params(&x,&y)?;let e=x.iter().zip(y).map(|(a,b)|b-(s*a+i)).collect::<Vec<_>>();let sse=e.iter().map(|v|v*v).sum::<f64>();let out=match kind{"sse"=>sse,"mse"=>sse/e.len() as f64,"rmse"=>(sse/e.len() as f64).sqrt(),"mae"=>e.iter().map(|v|v.abs()).sum::<f64>()/e.len() as f64,_=>return Err("OP")};finite(out)}
fn origin_slope(args:&[Value])->Result<Value,&'static str>{let(x,y)=xy(args)?;let d=x.iter().map(|v|v*v).sum::<f64>();if d==0.0{return Err("DOMAIN");}finite(x.iter().zip(y).map(|(a,b)|a*b).sum::<f64>()/d)}
fn weighted_linear(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let x=vec(&args[0])?;let y=vec(&args[1])?;let w=vec(&args[2])?;if x.is_empty()||x.len()!=y.len()||x.len()!=w.len(){return Err("SHAPE");}if w.iter().any(|v|*v<0.0){return Err("DOMAIN");}let sw=w.iter().sum::<f64>();if sw==0.0{return Err("DIV0");}let mx=x.iter().zip(&w).map(|(a,b)|a*b).sum::<f64>()/sw;let my=y.iter().zip(&w).map(|(a,b)|a*b).sum::<f64>()/sw;let sxx=x.iter().zip(&w).map(|(a,b)|b*(a-mx)*(a-mx)).sum::<f64>();if sxx==0.0{return Err("DOMAIN");}let sxy=x.iter().zip(&y).zip(&w).map(|((a,b),ww)|ww*(a-mx)*(b-my)).sum::<f64>();let s=sxy/sxx;Ok(json!({"slope":s,"intercept":my-s*mx}))}
fn pair_corr(args:&[Value])->Result<Value,&'static str>{let(x,y)=xy(args)?;let mx=mean(&x);let my=mean(&y);let mut a=0.0;let mut b=0.0;let mut c=0.0;for(u,v)in x.iter().zip(y){let dx=*u-mx;let dy=v-my;a+=dx*dy;b+=dx*dx;c+=dy*dy;}if b==0.0||c==0.0{return Err("DOMAIN");}finite(a/(b*c).sqrt())}
fn reg_standard_error(args:&[Value])->Result<Value,&'static str>{let(x,y)=xy(args)?;if x.len()<3{return Err("EMPTY");}let(s,i,_)=linear_params(&x,&y)?;let sse=x.iter().zip(y).map(|(a,b)|{let e=b-(s*a+i);e*e}).sum::<f64>();finite((sse/(x.len()-2) as f64).sqrt())}
fn z_one_sample(args:&[Value])->Result<Value,&'static str>{if args.len()!=4{return Err("ARG");}let m=num(&args[0])?;let mu=num(&args[1])?;let sigma=num(&args[2])?;let n=num(&args[3])?;if sigma<=0.0||n<=0.0{return Err("DOMAIN");}finite((m-mu)/(sigma/n.sqrt()))}
fn t_one_sample(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;if x.len()<2{return Err("EMPTY");}let mu=num(&args[1])?;let s=sample_var(&x)?.sqrt();if s==0.0{return Err("DOMAIN");}finite((mean(&x)-mu)/(s/(x.len() as f64).sqrt()))}
fn t_two_equal(args:&[Value])->Result<Value,&'static str>{let(a,b)=pair_two_samples(args)?;let va=sample_var(&a)?;let vb=sample_var(&b)?;let df=(a.len()+b.len()-2) as f64;let sp=(((a.len()-1) as f64*va+(b.len()-1) as f64*vb)/df).sqrt();if sp==0.0{return Err("DOMAIN");}finite((mean(&a)-mean(&b))/(sp*(1.0/a.len() as f64+1.0/b.len() as f64).sqrt()))}
fn t_welch(args:&[Value])->Result<Value,&'static str>{let(a,b)=pair_two_samples(args)?;let va=sample_var(&a)?;let vb=sample_var(&b)?;let se=(va/a.len() as f64+vb/b.len() as f64).sqrt();if se==0.0{return Err("DOMAIN");}finite((mean(&a)-mean(&b))/se)}
fn pair_two_samples(args:&[Value])->Result<(Vec<f64>,Vec<f64>),&'static str>{if args.len()!=2{return Err("ARG");}let a=vec(&args[0])?;let b=vec(&args[1])?;if a.len()<2||b.len()<2{return Err("EMPTY");}Ok((a,b))}
fn chi_square_gof(args:&[Value])->Result<Value,&'static str>{let(o,e)=pair(args)?;if e.iter().any(|v|*v<=0.0){return Err("DOMAIN");}finite(o.iter().zip(e).map(|(a,b)|{let d=a-b;d*d/b}).sum())}
fn chi_square_independence(args:&[Value])->Result<Value,&'static str>{if args.len()!=1{return Err("ARG");}let rows=args[0].as_array().ok_or("TYPE")?;if rows.len()<2{return Err("SHAPE");}let mut table=Vec::new();let mut cols=None;for r in rows{let v=vec(r)?;if v.len()<2{return Err("SHAPE");}if let Some(c)=cols{if c!=v.len(){return Err("SHAPE");}}else{cols=Some(v.len());}if v.iter().any(|x|*x<0.0){return Err("DOMAIN");}table.push(v);}let c=cols.unwrap();let row_s=table.iter().map(|r|r.iter().sum::<f64>()).collect::<Vec<_>>();let mut col_s=vec![0.0;c];for r in &table{for(j,x)in r.iter().enumerate(){col_s[j]+=x;}}let total=row_s.iter().sum::<f64>();if total==0.0{return Err("DOMAIN");}let mut chi=0.0;for i in 0..table.len(){for j in 0..c{let ex=row_s[i]*col_s[j]/total;if ex==0.0{return Err("DOMAIN");}let d=table[i][j]-ex;chi+=d*d/ex;}}finite(chi)}
fn f_variance_ratio(args:&[Value])->Result<Value,&'static str>{let(a,b)=pair_two_samples(args)?;let vb=sample_var(&b)?;if vb==0.0{return Err("DIV0");}finite(sample_var(&a)?/vb)}
fn cohen_d(args:&[Value])->Result<Value,&'static str>{let(a,b)=pair_two_samples(args)?;let va=sample_var(&a)?;let vb=sample_var(&b)?;let sp=((((a.len()-1) as f64)*va+((b.len()-1) as f64)*vb)/(a.len()+b.len()-2) as f64).sqrt();if sp==0.0{return Err("DOMAIN");}finite((mean(&a)-mean(&b))/sp)}
fn proportion_z(args:&[Value])->Result<Value,&'static str>{if args.len()!=3{return Err("ARG");}let ph=num(&args[0])?;let p0=num(&args[1])?;let n=num(&args[2])?;if !(0.0..=1.0).contains(&ph)||!(0.0..=1.0).contains(&p0)||n<=0.0{return Err("DOMAIN");}let se=(p0*(1.0-p0)/n).sqrt();if se==0.0{return Err("DOMAIN");}finite((ph-p0)/se)}
fn mean_diff(args:&[Value])->Result<Value,&'static str>{let(a,b)=pair_two_samples(args)?;finite(mean(&a)-mean(&b))}
fn trimmed_mean(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let mut x=vec(&args[0])?;if x.is_empty(){return Err("EMPTY");}let p=num(&args[1])?;if !(0.0..50.0).contains(&p){return Err("DOMAIN");}x.sort_by(f64::total_cmp);let k=((p/100.0)*x.len() as f64).floor() as usize;if 2*k>=x.len(){return Err("DOMAIN");}finite(mean(&x[k..x.len()-k]))}
fn winsorized_mean(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let mut x=vec(&args[0])?;if x.is_empty(){return Err("EMPTY");}let p=num(&args[1])?;if !(0.0..50.0).contains(&p){return Err("DOMAIN");}x.sort_by(f64::total_cmp);let k=((p/100.0)*x.len() as f64).floor() as usize;if k>0&&2*k<x.len(){let lo=x[k];let hi=x[x.len()-k-1];for i in 0..k{x[i]=lo;}let n=x.len();for i in n-k..n{x[i]=hi;}}finite(mean(&x))}
fn median_abs_dev(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let m=median(x.clone())?;finite(median(x.into_iter().map(|v|(v-m).abs()).collect())?)}
fn quartiles(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;Ok(json!([percentile(x.clone(),25.0)?,percentile(x.clone(),50.0)?,percentile(x,75.0)?]))}
fn five_number(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let lo=x.iter().copied().reduce(f64::min).unwrap();let hi=x.iter().copied().reduce(f64::max).unwrap();Ok(json!([lo,percentile(x.clone(),25.0)?,percentile(x.clone(),50.0)?,percentile(x.clone(),75.0)?,hi]))}
fn key(x:f64)->u64{x.to_bits()}
fn unique_count(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let mut m=BTreeMap::new();for v in x{m.insert(key(v),());}Ok(json!(m.len()))}
fn frequency(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let mut m:BTreeMap<u64,(f64,usize)>=BTreeMap::new();for v in x{let e=m.entry(key(v)).or_insert((v,0));e.1+=1;}Ok(json!(m.values().map(|(v,n)|json!([v,n])).collect::<Vec<_>>()))}
fn entropy(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let n=x.len() as f64;let mut m=BTreeMap::new();for v in x{*m.entry(key(v)).or_insert(0usize)+=1;}finite(m.values().map(|c|{let p=*c as f64/n;-p*p.log2()}).sum())}
fn gini_impurity(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let n=x.len() as f64;let mut m=BTreeMap::new();for v in x{*m.entry(key(v)).or_insert(0usize)+=1;}finite(1.0-m.values().map(|c|{let p=*c as f64/n;p*p}).sum::<f64>())}
fn gini_coefficient(args:&[Value])->Result<Value,&'static str>{let mut x=one(args)?;if x.iter().any(|v|*v<0.0){return Err("DOMAIN");}x.sort_by(f64::total_cmp);let sum=x.iter().sum::<f64>();if sum==0.0{return Ok(json!(0.0));}let n=x.len() as f64;let weighted=x.iter().enumerate().map(|(i,v)|(i+1) as f64*v).sum::<f64>();finite(2.0*weighted/(n*sum)-(n+1.0)/n)}
fn ranks(x:&[f64])->Vec<f64>{let mut idx=(0..x.len()).collect::<Vec<_>>();idx.sort_by(|&a,&b|x[a].total_cmp(&x[b]));let mut r=vec![0.0;x.len()];let mut i=0;while i<idx.len(){let mut j=i+1;while j<idx.len()&&x[idx[j]].to_bits()==x[idx[i]].to_bits(){j+=1;}let avg=((i+1+j) as f64)/2.0;for k in i..j{r[idx[k]]=avg;}i=j;}r}
fn ranks_op(args:&[Value])->Result<Value,&'static str>{Ok(json!(ranks(&one(args)?)))}
fn pearson(x:&[f64],y:&[f64])->Result<f64,&'static str>{let mx=mean(x);let my=mean(y);let mut n=0.0;let mut a=0.0;let mut b=0.0;for(u,v)in x.iter().zip(y){let dx=*u-mx;let dy=*v-my;n+=dx*dy;a+=dx*dx;b+=dy*dy;}if a==0.0||b==0.0{Err("DOMAIN")}else{Ok(n/(a*b).sqrt())}}
fn spearman(args:&[Value])->Result<Value,&'static str>{let(x,y)=pair(args)?;finite(pearson(&ranks(&x),&ranks(&y))?)}
fn autocorr(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let lag=args[1].as_u64().ok_or("TYPE")? as usize;if x.len()<2||lag==0||lag>=x.len(){return Err("DOMAIN");}finite(pearson(&x[..x.len()-lag],&x[lag..])?)}
fn lag(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let k=args[1].as_u64().ok_or("TYPE")? as usize;if k>x.len(){return Err("DOMAIN");}Ok(json!(x.into_iter().skip(k).collect::<Vec<_>>()))}
fn residuals(args:&[Value])->Result<Value,&'static str>{let(a,p)=pair(args)?;Ok(json!(a.into_iter().zip(p).map(|(x,y)|x-y).collect::<Vec<_>>()))}
fn metric_two(args:&[Value],kind:&str)->Result<Value,&'static str>{let(a,p)=pair(args)?;let n=a.len() as f64;let mut s=0.0;for(x,y)in a.iter().zip(p){let d=*x-y;s+=match kind{"mae"=>d.abs(),"mse"|"rmse"|"sse"=>d*d,"mape"=>{if *x==0.0{return Err("DIV0");}(d/(*x)).abs()*100.0},_=>return Err("OP")};}finite(match kind{"sse"=>s,"rmse"=>(s/n).sqrt(),_=>s/n})}
fn sample_cov(args:&[Value])->Result<Value,&'static str>{let(x,y)=pair(args)?;if x.len()<2{return Err("EMPTY");}let mx=mean(&x);let my=mean(&y);finite(x.iter().zip(y).map(|(a,b)|(a-mx)*(b-my)).sum::<f64>()/(x.len()-1) as f64)}
fn pooled_mean(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let a=vec(&args[0])?;let b=vec(&args[1])?;if a.is_empty()||b.is_empty(){return Err("EMPTY");}finite((a.iter().sum::<f64>()+b.iter().sum::<f64>())/(a.len()+b.len()) as f64)}
fn pooled_variance(args:&[Value])->Result<Value,&'static str>{let(a,b)=pair_two_samples(args)?;let va=sample_var(&a)?;let vb=sample_var(&b)?;finite(((a.len()-1) as f64*va+(b.len()-1) as f64*vb)/(a.len()+b.len()-2) as f64)}
fn quantile(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;let q=num(&args[1])?;if !(0.0..=1.0).contains(&q){return Err("DOMAIN");}finite(percentile(x,q*100.0)?)}
fn ecdf(args:&[Value])->Result<Value,&'static str>{if args.len()!=2{return Err("ARG");}let x=vec(&args[0])?;if x.is_empty(){return Err("EMPTY");}let q=num(&args[1])?;Ok(json!(x.iter().filter(|v|**v<=q).count() as f64/x.len() as f64))}
fn outlier_iqr(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let q1=percentile(x.clone(),25.0)?;let q3=percentile(x.clone(),75.0)?;let i=q3-q1;let lo=q1-1.5*i;let hi=q3+1.5*i;Ok(json!(x.into_iter().filter(|v|*v<lo||*v>hi).collect::<Vec<_>>()))}
fn outlier_z(args:&[Value])->Result<Value,&'static str>{if args.is_empty()||args.len()>2{return Err("ARG");}let x=vec(&args[0])?;if x.is_empty(){return Err("EMPTY");}let threshold=if args.len()==2{num(&args[1])?}else{3.0};if threshold<=0.0{return Err("DOMAIN");}let m=mean(&x);let sd=(x.iter().map(|v|{let d=*v-m;d*d}).sum::<f64>()/x.len() as f64).sqrt();if sd==0.0{return Ok(json!([]));}Ok(json!(x.into_iter().filter(|v|((*v-m)/sd).abs()>threshold).collect::<Vec<_>>()))}
fn center(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let m=mean(&x);Ok(json!(x.into_iter().map(|v|v-m).collect::<Vec<_>>()))}
fn standardize_sample(args:&[Value])->Result<Value,&'static str>{let x=one(args)?;let m=mean(&x);let sd=sample_var(&x)?.sqrt();if sd==0.0{return Err("DOMAIN");}Ok(json!(x.into_iter().map(|v|(v-m)/sd).collect::<Vec<_>>()))}
