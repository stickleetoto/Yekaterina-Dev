use serde_json::{json, Value};

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("color."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "color.rgb_to_hex"=>{let c=rgb_arg(args,0)?;need(args,1)?;Ok(json!(format!("#{:02X}{:02X}{:02X}",round8(c[0]),round8(c[1]),round8(c[2]))))},
    "color.hex_to_rgb"=>hex_to_rgb(args),
    "color.rgb_to_hsv"=>{need(args,1)?;Ok(json!(rgb_to_hsv(rgb_arg(args,0)?)))},
    "color.hsv_to_rgb"=>{need(args,1)?;Ok(json!(hsv_to_rgb(triple(&args[0])?)?))},
    "color.rgb_to_hsl"=>{need(args,1)?;Ok(json!(rgb_to_hsl(rgb_arg(args,0)?)))},
    "color.hsl_to_rgb"=>{need(args,1)?;Ok(json!(hsl_to_rgb(triple(&args[0])?)?))},
    "color.rgb_to_cmyk"=>{need(args,1)?;Ok(json!(rgb_to_cmyk(rgb_arg(args,0)?)))},
    "color.cmyk_to_rgb"=>{need(args,1)?;Ok(json!(cmyk_to_rgb(quad(&args[0])?)?))},
    "color.srgb_to_linear"=>{need(args,1)?;let c=rgb_arg(args,0)?;Ok(json!(c.map(|x|srgb_chan(x/255.0))))},
    "color.linear_to_srgb"=>{need(args,1)?;let c=triple(&args[0])?;if c.iter().any(|x|*x<0.0){return Err("DOMAIN");}Ok(json!(c.map(|x|255.0*linear_chan(x))))},
    "color.relative_luminance"=>{need(args,1)?;finite(luminance(rgb_arg(args,0)?))},
    "color.contrast_ratio"=>contrast_ratio(args),
    "color.grayscale_luma"=>{need(args,1)?;let c=rgb_arg(args,0)?;let y=0.2126*c[0]+0.7152*c[1]+0.0722*c[2];Ok(json!([y,y,y]))},
    "color.invert"=>{need(args,1)?;let c=rgb_arg(args,0)?;Ok(json!(c.map(|x|255.0-x)))},
    "color.mix"=>mix(args),
    "color.premultiply"=>premultiply(args),
    "color.unpremultiply"=>unpremultiply(args),
    "color.alpha_over"=>alpha_over(args),
    "color.rgb_to_yiq"=>{need(args,1)?;let c=rgb_arg(args,0)?;Ok(json!(rgb_to_yiq(c)))},
    "color.yiq_to_rgb"=>{need(args,1)?;Ok(json!(yiq_to_rgb(triple(&args[0])?)))},
    "color.rgb_to_ycbcr"=>{need(args,1)?;let c=rgb_arg(args,0)?;Ok(json!(rgb_to_ycbcr(c)))},
    "color.ycbcr_to_rgb"=>{need(args,1)?;Ok(json!(ycbcr_to_rgb(triple(&args[0])?)))},
    "color.sepia"=>{need(args,1)?;Ok(json!(sepia(rgb_arg(args,0)?)))},
    "color.brightness"=>brightness(args),
    "color.contrast_adjust"=>contrast_adjust(args),
    "color.saturate"=>saturate(args),
    "color.clamp_rgb"=>{need(args,1)?;let c=triple(&args[0])?;Ok(json!(c.map(clamp255)))},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn triple(v:&Value)->Result<[f64;3],&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=3{return Err("SHAPE");}Ok([num(&a[0])?,num(&a[1])?,num(&a[2])?])}
fn quad(v:&Value)->Result<[f64;4],&'static str>{let a=v.as_array().ok_or("TYPE")?;if a.len()!=4{return Err("SHAPE");}Ok([num(&a[0])?,num(&a[1])?,num(&a[2])?,num(&a[3])?])}
fn rgb_arg(args:&[Value],i:usize)->Result<[f64;3],&'static str>{let c=triple(args.get(i).ok_or("ARG")?)?;if c.iter().any(|x|!x.is_finite()||*x<0.0||*x>255.0){Err("DOMAIN")}else{Ok(c)}}
fn round8(x:f64)->u8{x.round().clamp(0.0,255.0) as u8}
fn clamp255(x:f64)->f64{x.clamp(0.0,255.0)}
fn hex_to_rgb(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let s=args[0].as_str().ok_or("TYPE")?.trim().trim_start_matches('#');if s.len()!=6||!s.bytes().all(|b|b.is_ascii_hexdigit()){return Err("COLOR");}let r=u8::from_str_radix(&s[0..2],16).map_err(|_|"COLOR")?;let g=u8::from_str_radix(&s[2..4],16).map_err(|_|"COLOR")?;let b=u8::from_str_radix(&s[4..6],16).map_err(|_|"COLOR")?;Ok(json!([r,g,b]))}
fn rgb_to_hsv(c:[f64;3])->[f64;3]{let r=c[0]/255.0;let g=c[1]/255.0;let b=c[2]/255.0;let max=r.max(g).max(b);let min=r.min(g).min(b);let d=max-min;let mut h=if d==0.0{0.0}else if max==r{60.0*(((g-b)/d)%6.0)}else if max==g{60.0*((b-r)/d+2.0)}else{60.0*((r-g)/d+4.0)};if h<0.0{h+=360.0;}let s=if max==0.0{0.0}else{d/max};[h,s,max]}
fn hsv_to_rgb(hsv:[f64;3])->Result<[f64;3],&'static str>{let(mut h,s,v)=(hsv[0],hsv[1],hsv[2]);if !h.is_finite()||!(0.0..=1.0).contains(&s)||!(0.0..=1.0).contains(&v){return Err("DOMAIN");}h=h.rem_euclid(360.0);let c=v*s;let x=c*(1.0-(((h/60.0)%2.0)-1.0).abs());let m=v-c;let(r,g,b)=match h{h if h<60.0=>(c,x,0.0),h if h<120.0=>(x,c,0.0),h if h<180.0=>(0.0,c,x),h if h<240.0=>(0.0,x,c),h if h<300.0=>(x,0.0,c),_=>(c,0.0,x)};Ok([255.0*(r+m),255.0*(g+m),255.0*(b+m)])}
fn rgb_to_hsl(c:[f64;3])->[f64;3]{let hsv=rgb_to_hsv(c);let v=hsv[2];let s_v=hsv[1];let l=v*(1.0-s_v/2.0);let s=if l==0.0||l==1.0{0.0}else{(v-l)/l.min(1.0-l)};[hsv[0],s,l]}
fn hsl_to_rgb(hsl:[f64;3])->Result<[f64;3],&'static str>{let(h,s,l)=(hsl[0].rem_euclid(360.0),hsl[1],hsl[2]);if !(0.0..=1.0).contains(&s)||!(0.0..=1.0).contains(&l){return Err("DOMAIN");}let c=(1.0-(2.0*l-1.0).abs())*s;let x=c*(1.0-(((h/60.0)%2.0)-1.0).abs());let m=l-c/2.0;let(r,g,b)=match h{h if h<60.0=>(c,x,0.0),h if h<120.0=>(x,c,0.0),h if h<180.0=>(0.0,c,x),h if h<240.0=>(0.0,x,c),h if h<300.0=>(x,0.0,c),_=>(c,0.0,x)};Ok([255.0*(r+m),255.0*(g+m),255.0*(b+m)])}
fn rgb_to_cmyk(c:[f64;3])->[f64;4]{let(r,g,b)=(c[0]/255.0,c[1]/255.0,c[2]/255.0);let k=1.0-r.max(g).max(b);if k>=1.0-1e-15{[0.0,0.0,0.0,1.0]}else{[(1.0-r-k)/(1.0-k),(1.0-g-k)/(1.0-k),(1.0-b-k)/(1.0-k),k]}}
fn cmyk_to_rgb(x:[f64;4])->Result<[f64;3],&'static str>{if x.iter().any(|v|!(0.0..=1.0).contains(v)){return Err("DOMAIN");}Ok([255.0*(1.0-x[0])*(1.0-x[3]),255.0*(1.0-x[1])*(1.0-x[3]),255.0*(1.0-x[2])*(1.0-x[3])])}
fn srgb_chan(x:f64)->f64{if x<=0.04045{x/12.92}else{((x+0.055)/1.055).powf(2.4)}}
fn linear_chan(x:f64)->f64{if x<=0.0031308{12.92*x}else{1.055*x.powf(1.0/2.4)-0.055}}
fn luminance(c:[f64;3])->f64{let l=c.map(|x|srgb_chan(x/255.0));0.2126*l[0]+0.7152*l[1]+0.0722*l[2]}
fn contrast_ratio(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let l1=luminance(rgb_arg(args,0)?);let l2=luminance(rgb_arg(args,1)?);finite((l1.max(l2)+0.05)/(l1.min(l2)+0.05))}
fn mix(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let a=rgb_arg(args,0)?;let b=rgb_arg(args,1)?;let t=num(&args[2])?;if !(0.0..=1.0).contains(&t){return Err("DOMAIN");}Ok(json!([a[0]+(b[0]-a[0])*t,a[1]+(b[1]-a[1])*t,a[2]+(b[2]-a[2])*t]))}
fn premultiply(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let c=rgb_arg(args,0)?;let a=num(&args[1])?;if !(0.0..=1.0).contains(&a){return Err("DOMAIN");}Ok(json!([c[0]*a,c[1]*a,c[2]*a,a]))}
fn unpremultiply(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let x=quad(&args[0])?;let a=x[3];if a<=0.0||a>1.0{return Err("DOMAIN");}Ok(json!([clamp255(x[0]/a),clamp255(x[1]/a),clamp255(x[2]/a),a]))}
fn alpha_over(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let f=quad(&args[0])?;let b=quad(&args[1])?;if !(0.0..=1.0).contains(&f[3])||!(0.0..=1.0).contains(&b[3]){return Err("DOMAIN");}let ao=f[3]+b[3]*(1.0-f[3]);if ao==0.0{return Ok(json!([0.0,0.0,0.0,0.0]));}let mut out=[0.0;4];for i in 0..3{out[i]=(f[i]*f[3]+b[i]*b[3]*(1.0-f[3]))/ao;}out[3]=ao;Ok(json!(out))}
fn rgb_to_yiq(c:[f64;3])->[f64;3]{let r=c[0]/255.0;let g=c[1]/255.0;let b=c[2]/255.0;[0.299*r+0.587*g+0.114*b,0.596*r-0.274*g-0.322*b,0.211*r-0.523*g+0.312*b]}
fn yiq_to_rgb(x:[f64;3])->[f64;3]{[clamp255(255.0*(x[0]+0.956*x[1]+0.621*x[2])),clamp255(255.0*(x[0]-0.272*x[1]-0.647*x[2])),clamp255(255.0*(x[0]-1.106*x[1]+1.703*x[2]))]}
fn rgb_to_ycbcr(c:[f64;3])->[f64;3]{let(r,g,b)=(c[0],c[1],c[2]);[0.299*r+0.587*g+0.114*b,128.0-0.168736*r-0.331264*g+0.5*b,128.0+0.5*r-0.418688*g-0.081312*b]}
fn ycbcr_to_rgb(x:[f64;3])->[f64;3]{let y=x[0];let cb=x[1]-128.0;let cr=x[2]-128.0;[clamp255(y+1.402*cr),clamp255(y-0.344136*cb-0.714136*cr),clamp255(y+1.772*cb)]}
fn sepia(c:[f64;3])->[f64;3]{[clamp255(0.393*c[0]+0.769*c[1]+0.189*c[2]),clamp255(0.349*c[0]+0.686*c[1]+0.168*c[2]),clamp255(0.272*c[0]+0.534*c[1]+0.131*c[2])]}
fn brightness(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let c=rgb_arg(args,0)?;let d=num(&args[1])?;Ok(json!(c.map(|x|clamp255(x+d))))}
fn contrast_adjust(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let c=rgb_arg(args,0)?;let f=num(&args[1])?;if f<0.0{return Err("DOMAIN");}Ok(json!(c.map(|x|clamp255((x-128.0)*f+128.0))))}
fn saturate(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let c=rgb_arg(args,0)?;let f=num(&args[1])?;if f<0.0{return Err("DOMAIN");}let y=0.2126*c[0]+0.7152*c[1]+0.0722*c[2];Ok(json!([clamp255(y+(c[0]-y)*f),clamp255(y+(c[1]-y)*f),clamp255(y+(c[2]-y)*f)]))}
