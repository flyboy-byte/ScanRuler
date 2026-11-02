// SPDX-License-Identifier: AGPL-3.0-only
use port_engine::schema_value::{Text, Value};
struct Reader<'a>{b:&'a[u8],at:usize}
impl Reader<'_>{
 fn take(&mut self,n:usize)->&[u8]{let s=&self.b[self.at..self.at+n];self.at+=n;s}
 fn u32(&mut self)->usize{u32::from_le_bytes(self.take(4).try_into().unwrap())as usize}
 fn text(&mut self)->Text{let n=self.u32();self.take(n*2).as_chunks::<2>().0.iter().map(|b|u16::from_le_bytes(*b)).collect()}
 fn value(&mut self)->Value{match self.take(1)[0]{0=>Value::Undefined,1=>Value::Null,2=>Value::Boolean(false),3=>Value::Boolean(true),4=>Value::Number(f64::from_le_bytes(self.take(8).try_into().unwrap())),5=>Value::String(self.text()),6=>{let n=self.u32();Value::Array((0..n).map(|_|self.value()).collect())},7=>{let n=self.u32();Value::Object((0..n).map(|_|(self.text(),self.value())).collect())},8=>{let n=self.u32();Value::Bytes(self.take(n).to_vec())},_=>panic!("invalid committed tag")}}
}
fn n(b:&mut Vec<u8>,v:usize){b.extend((v as u32).to_le_bytes())}
fn t(b:&mut Vec<u8>,v:&[u16]){n(b,v.len());for u in v{b.extend(u.to_le_bytes())}}
fn index(k:&[u16])->Option<u32>{let s=String::from_utf16(k).ok()?;let v=s.parse::<u32>().ok()?;(v<u32::MAX&&v.to_string()==s).then_some(v)}
fn encode(b:&mut Vec<u8>,v:&Value){match v{
 Value::Undefined=>b.push(0),Value::Null=>b.push(1),Value::Boolean(x)=>b.push(if *x{3}else{2}),Value::Number(x)=>{b.push(4);b.extend(x.to_le_bytes());},Value::String(s)=>{b.push(5);t(b,s)},Value::Array(a)=>{b.push(6);n(b,a.len());for x in a{encode(b,x)}},Value::Object(a)=>{b.push(7);n(b,a.len());let mut ids:Vec<_>=(0..a.len()).collect();ids.sort_by_key(|&i|(index(&a[i].0).is_none(),index(&a[i].0).unwrap_or(0)));for i in ids{t(b,&a[i].0);encode(b,&a[i].1);}},Value::Bytes(a)=>{b.push(8);n(b,a.len());b.extend(a);}
}}
