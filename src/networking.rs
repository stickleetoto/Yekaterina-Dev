use serde_json::{json, Value};
use std::net::Ipv4Addr;

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("net."){Some(run(op,args))}else{None}}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
    "net.ipv4_to_u32"=>{let ip=one_ip(args)?;Ok(json!(u32::from(ip)))},
    "net.u32_to_ipv4"=>{let n=one_u32(args)?;Ok(json!(Ipv4Addr::from(n).to_string()))},
    "net.mask_from_prefix"=>{let p=one_prefix(args)?;Ok(json!(Ipv4Addr::from(mask(p)).to_string()))},
    "net.prefix_from_mask"=>{let m=u32::from(one_ip(args)?);Ok(json!(prefix_from_mask(m)?))},
    "net.wildcard_from_prefix"=>{let p=one_prefix(args)?;Ok(json!(Ipv4Addr::from(!mask(p)).to_string()))},
    "net.network"=>{let(ip,p)=ip_prefix(args)?;Ok(json!(Ipv4Addr::from(u32::from(ip)&mask(p)).to_string()))},
    "net.broadcast"=>{let(ip,p)=ip_prefix(args)?;let m=mask(p);Ok(json!(Ipv4Addr::from((u32::from(ip)&m)|!m).to_string()))},
    "net.first_host"=>first_last(args,true),
    "net.last_host"=>first_last(args,false),
    "net.address_count"=>{let p=one_prefix(args)?;Ok(json!(1u64<<(32-p)))},
    "net.usable_host_count"=>{let p=one_prefix(args)?;let n=1u64<<(32-p);Ok(json!(if p>=31{n}else{n-2}))},
    "net.contains"=>contains(args),
    "net.same_subnet"=>same_subnet(args),
    "net.cidr_normalize"=>cidr_normalize(args),
    "net.subnet_index"=>{let(ip,p)=ip_prefix(args)?;let size=1u64<<(32-p);Ok(json!(u64::from(u32::from(ip))/size))},
    "net.nth_host"=>nth_host(args),
    "net.host_offset"=>host_offset(args),
    "net.supernet"=>supernet(args),
    "net.subnet_count"=>{let(a,b)=two_prefix(args)?;if b<a{return Err("DOMAIN");}Ok(json!(1u64<<(b-a)))},
    "net.fragment_count"=>fragment_count(args),
    "net.serialization_delay"=>{let(bytes,bps)=two_num(args)?;positive(bps)?;nonneg(bytes)?;finite(bytes*8.0/bps)},
    "net.transfer_time"=>{let(bytes,bps)=two_num(args)?;positive(bps)?;nonneg(bytes)?;finite(bytes*8.0/bps)},
    "net.propagation_delay"=>{let(d,s)=two_num(args)?;nonneg(d)?;positive(s)?;finite(d/s)},
    "net.bandwidth_delay_product"=>{let(bps,rtt)=two_num(args)?;nonneg(bps)?;nonneg(rtt)?;finite(bps*rtt/8.0)},
    "net.packet_rate"=>{let(bps,bytes)=two_num(args)?;nonneg(bps)?;positive(bytes)?;finite(bps/(bytes*8.0))},
    "net.goodput"=>goodput(args),
    "net.utilization"=>{let(u,c)=two_num(args)?;nonneg(u)?;positive(c)?;finite(u/c*100.0)},
    "net.bitrate_from_baud"=>{let(b,s)=two_num(args)?;nonneg(b)?;nonneg(s)?;finite(b*s)},
    "net.baud_from_bitrate"=>{let(r,s)=two_num(args)?;nonneg(r)?;positive(s)?;finite(r/s)},
    "net.bits_per_symbol"=>{let levels=one_num(args)?;if levels<2.0{return Err("DOMAIN");}finite(levels.log2())},
    "net.tcp_window_bytes"=>{let(bps,rtt)=two_num(args)?;nonneg(bps)?;nonneg(rtt)?;finite(bps*rtt/8.0)},
    "net.mtu_payload"=>{let(mtu,h)=two_num(args)?;if mtu<=0.0||h<0.0||h>mtu{return Err("DOMAIN");}finite(mtu-h)},
    "net.overhead_percent"=>{let(total,payload)=two_num(args)?;positive(total)?;if payload<0.0||payload>total{return Err("DOMAIN");}finite((total-payload)/total*100.0)},
    _=>Err("OP")}}
fn need(args:&[Value],n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn parse_ip(v:&Value)->Result<Ipv4Addr,&'static str>{v.as_str().ok_or("TYPE")?.parse().map_err(|_|"IP")}
fn one_ip(args:&[Value])->Result<Ipv4Addr,&'static str>{need(args,1)?;parse_ip(&args[0])}
fn one_u32(args:&[Value])->Result<u32,&'static str>{need(args,1)?;let n=args[0].as_u64().ok_or("TYPE")?;u32::try_from(n).map_err(|_|"DOMAIN")}
fn prefix(v:&Value)->Result<u8,&'static str>{let p=v.as_u64().ok_or("TYPE")?;if p<=32{Ok(p as u8)}else{Err("DOMAIN")}}
fn one_prefix(args:&[Value])->Result<u8,&'static str>{need(args,1)?;prefix(&args[0])}
fn two_prefix(args:&[Value])->Result<(u8,u8),&'static str>{need(args,2)?;Ok((prefix(&args[0])?,prefix(&args[1])?))}
fn ip_prefix(args:&[Value])->Result<(Ipv4Addr,u8),&'static str>{need(args,2)?;Ok((parse_ip(&args[0])?,prefix(&args[1])?))}
fn mask(p:u8)->u32{if p==0{0}else{u32::MAX<<(32-p)}}
fn prefix_from_mask(m:u32)->Result<u8,&'static str>{let p=m.leading_ones() as u8;if m!=mask(p){Err("MASK")}else{Ok(p)}}
fn first_last(args:&[Value],first:bool)->Result<Value,&'static str>{let(ip,p)=ip_prefix(args)?;let m=mask(p);let n=u32::from(ip)&m;let b=n|!m;let out=if p>=31{if first{n}else{b}}else if first{n+1}else{b-1};Ok(json!(Ipv4Addr::from(out).to_string()))}
fn parse_cidr(v:&Value)->Result<(Ipv4Addr,u8),&'static str>{let s=v.as_str().ok_or("TYPE")?;let mut it=s.split('/');let ip:Ipv4Addr=it.next().ok_or("CIDR")?.parse().map_err(|_|"CIDR")?;let p:u8=it.next().ok_or("CIDR")?.parse().map_err(|_|"CIDR")?;if it.next().is_some()||p>32{return Err("CIDR");}Ok((ip,p))}
fn contains(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let(base,p)=parse_cidr(&args[0])?;let ip=parse_ip(&args[1])?;let m=mask(p);Ok(json!((u32::from(base)&m)==(u32::from(ip)&m)))}
fn same_subnet(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let a=parse_ip(&args[0])?;let b=parse_ip(&args[1])?;let p=prefix(&args[2])?;let m=mask(p);Ok(json!((u32::from(a)&m)==(u32::from(b)&m)))}
fn cidr_normalize(args:&[Value])->Result<Value,&'static str>{need(args,1)?;let(ip,p)=parse_cidr(&args[0])?;let n=u32::from(ip)&mask(p);Ok(json!(format!("{}/{}",Ipv4Addr::from(n),p)))}
fn nth_host(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let(ip,p)=parse_cidr(&args[0])?;let idx=args[1].as_u64().ok_or("TYPE")?;let size=1u64<<(32-p);if idx>=size{return Err("DOMAIN");}let n=u64::from(u32::from(ip)&mask(p));Ok(json!(Ipv4Addr::from(u32::try_from(n+idx).map_err(|_|"DOMAIN")?).to_string()))}
fn host_offset(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let(base,p)=parse_cidr(&args[0])?;let ip=parse_ip(&args[1])?;let n=u32::from(base)&mask(p);let x=u32::from(ip);if (x&mask(p))!=n{return Err("DOMAIN");}Ok(json!(u64::from(x-n)))}
fn supernet(args:&[Value])->Result<Value,&'static str>{need(args,2)?;let(ip,p)=parse_cidr(&args[0])?;let np=prefix(&args[1])?;if np>p{return Err("DOMAIN");}let n=u32::from(ip)&mask(np);Ok(json!(format!("{}/{}",Ipv4Addr::from(n),np)))}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn one_num(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
fn two_num(args:&[Value])->Result<(f64,f64),&'static str>{need(args,2)?;Ok((num(&args[0])?,num(&args[1])?))}
fn positive(x:f64)->Result<(),&'static str>{if x>0.0{Ok(())}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<(),&'static str>{if x>=0.0{Ok(())}else{Err("DOMAIN")}}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn fragment_count(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let payload=args[0].as_u64().ok_or("TYPE")?;let mtu=args[1].as_u64().ok_or("TYPE")?;let header=args[2].as_u64().ok_or("TYPE")?;if mtu<=header{return Err("DOMAIN");}let frag_payload=((mtu-header)/8)*8;if frag_payload==0{return Err("DOMAIN");}if payload==0{return Ok(json!(0));}Ok(json!((payload+frag_payload-1)/frag_payload))}
fn goodput(args:&[Value])->Result<Value,&'static str>{need(args,3)?;let payload=num(&args[0])?;let total=num(&args[1])?;let raw=num(&args[2])?;if payload<0.0||total<=0.0||payload>total||raw<0.0{return Err("DOMAIN");}finite(raw*payload/total)}
