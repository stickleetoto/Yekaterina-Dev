use serde_json::{json, Value};

fn need(args:&[Value], n:usize)->Result<(),&'static str>{if args.len()==n{Ok(())}else{Err("ARG")}}
fn num(v:&Value)->Result<f64,&'static str>{v.as_f64().ok_or("TYPE")}
fn nums(args:&[Value], n:usize)->Result<Vec<f64>,&'static str>{need(args,n)?;args.iter().map(num).collect()}
fn finite(x:f64)->Result<Value,&'static str>{if x.is_finite(){Ok(json!(x))}else{Err("NONFINITE")}}
fn nonzero(x:f64)->Result<f64,&'static str>{if x==0.0{Err("DIV0")}else{Ok(x)}}
fn positive(x:f64)->Result<f64,&'static str>{if x>0.0{Ok(x)}else{Err("DOMAIN")}}
fn nonneg(x:f64)->Result<f64,&'static str>{if x>=0.0{Ok(x)}else{Err("DOMAIN")}}

pub fn execute(op:&str,args:&[Value])->Option<Result<Value,&'static str>>{if op.starts_with("data."){Some(run(op,args))}else{None}}
fn u64v(v:&Value)->Result<u64,&'static str>{v.as_u64().ok_or("TYPE")} fn one_u(args:&[Value])->Result<u64,&'static str>{need(args,1)?;u64v(&args[0])}
fn bytes(args:&[Value])->Result<Vec<u8>,&'static str>{need(args,1)?;let a=args[0].as_array().ok_or("TYPE")?;if a.len()>1_000_000{return Err("LIMIT");}a.iter().map(|v|v.as_u64().and_then(|x|u8::try_from(x).ok()).ok_or("TYPE")).collect()}
fn ceil_div(a:u64,b:u64)->Result<u64,&'static str>{if b==0{return Err("DIV0");}Ok(a/b+if a%b!=0{1}else{0})}
fn run(op:&str,args:&[Value])->Result<Value,&'static str>{match op{
"data.bits_to_bytes"=>finite(num1(args)?/8.0),"data.bytes_to_bits"=>finite(num1(args)?*8.0),"data.kb_to_bytes"=>finite(num1(args)?*1000.0),"data.bytes_to_kb"=>finite(num1(args)?/1000.0),"data.kib_to_bytes"=>finite(num1(args)?*1024.0),"data.bytes_to_kib"=>finite(num1(args)?/1024.0),
"data.mb_to_bytes"=>finite(num1(args)?*1e6),"data.bytes_to_mb"=>finite(num1(args)?/1e6),"data.mib_to_bytes"=>finite(num1(args)?*1_048_576.0),"data.bytes_to_mib"=>finite(num1(args)?/1_048_576.0),
"data.gb_to_bytes"=>finite(num1(args)?*1e9),"data.bytes_to_gb"=>finite(num1(args)?/1e9),"data.gib_to_bytes"=>finite(num1(args)?*1_073_741_824.0),"data.bytes_to_gib"=>finite(num1(args)?/1_073_741_824.0),
"data.tb_to_bytes"=>finite(num1(args)?*1e12),"data.bytes_to_tb"=>finite(num1(args)?/1e12),"data.tib_to_bytes"=>finite(num1(args)?*1_099_511_627_776.0),"data.bytes_to_tib"=>finite(num1(args)?/1_099_511_627_776.0),
"data.transfer_seconds"=>{let v=nums(args,2)?;finite(v[0]*8.0/positive(v[1])?)},"data.throughput_bps"=>{let v=nums(args,2)?;finite(v[0]*8.0/positive(v[1])?)},"data.throughput_Bps"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},
"data.compression_ratio"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?)},"data.compression_savings"=>{let v=nums(args,2)?;finite((v[0]-v[1])/positive(v[0])?*100.0)},"data.overhead_percent"=>{let v=nums(args,2)?;finite((v[1]-v[0])/positive(v[1])?*100.0)},"data.payload_efficiency"=>{let v=nums(args,2)?;finite(v[0]/positive(v[1])?*100.0)},
"data.packets_needed"=>{need(args,2)?;Ok(json!(ceil_div(u64v(&args[0])?,u64v(&args[1])?)?))},"data.blocks_needed"=>{need(args,2)?;Ok(json!(ceil_div(u64v(&args[0])?,u64v(&args[1])?)?))},"data.ceil_div"=>{need(args,2)?;Ok(json!(ceil_div(u64v(&args[0])?,u64v(&args[1])?)?))},
"data.xor_checksum"=>{let b=bytes(args)?;Ok(json!(b.iter().fold(0u8,|a,&x|a^x)))},"data.sum8_checksum"=>{let b=bytes(args)?;Ok(json!((b.iter().map(|&x|x as u64).sum::<u64>()&0xff) as u8))},"data.sum16_checksum"=>{let b=bytes(args)?;Ok(json!((b.iter().map(|&x|x as u64).sum::<u64>()&0xffff) as u16))},
"data.fletcher16"=>{let b=bytes(args)?;let(mut a,mut c)=(0u16,0u16);for x in b{a=(a+x as u16)%255;c=(c+a)%255;}Ok(json!((c<<8)|a))},
"data.adler32"=>{let b=bytes(args)?;let(mut a,mut c)=(1u32,0u32);for x in b{a=(a+x as u32)%65521;c=(c+a)%65521;}Ok(json!((c<<16)|a))},
"data.parity_even"=>Ok(json!(one_u(args)?.count_ones()%2==0)),"data.parity_odd"=>Ok(json!(one_u(args)?.count_ones()%2==1)),"data.hamming_weight"=>Ok(json!(one_u(args)?.count_ones())),
"data.hamming_distance"=>{need(args,2)?;Ok(json!((u64v(&args[0])?^u64v(&args[1])?).count_ones()))},"data.bit_length"=>{let x=one_u(args)?;Ok(json!(if x==0{0}else{64-x.leading_zeros()}))},
"data.address_space"=>{need(args,1)?;let bits=args[0].as_u64().ok_or("TYPE")?;if bits>4096{return Err("LIMIT");}let n=num_bigint::BigUint::from(2u8).pow(bits as u32);Ok(json!(n.to_string()))},
"data.entropy_uniform"=>{let x=positive(num1(args)?)?;finite(x.log2())},
_=>Err("OP")}} fn num1(args:&[Value])->Result<f64,&'static str>{need(args,1)?;num(&args[0])}
