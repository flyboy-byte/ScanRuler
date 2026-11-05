// SPDX-License-Identifier: AGPL-3.0-only
//! js_atan2 against Node's Math.atan2, bit for bit.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use port_core::js_atan2;
use std::process::Command;

#[test]
fn atan2_specials_hold_their_signs() {
    assert_eq!(js_atan2(0., -1.).to_bits(), std::f64::consts::PI.to_bits());
    assert_eq!(js_atan2(-0., 1.).to_bits(), (-0f64).to_bits());
    assert!(js_atan2(f64::NAN, 1.).is_nan());
    assert_eq!(js_atan2(1., 1.), std::f64::consts::FRAC_PI_4);
}

/// 300,000 random pairs over many magnitudes plus every special-value pair.
#[test]
#[ignore = "needs Node"]
fn atan2_matches_node() {
    let script = r#"
let s=987654321;const r=()=>{s=(s*1103515245+12345)>>>0;return s/4294967296*2-1}
const sp=[0,-0,1,-1,Infinity,-Infinity,NaN,1e-310,-1e-310,5e-324,1.7976931348623157e308,2**60,2**-60,0.4375,1.1875,2.4375,11/16]
const xs=[];for(const a of sp)for(const b of sp)xs.push([a,b])
for(let i=0;i<300000;i++)xs.push([r()*Math.pow(10,(r()*40)|0),r()*Math.pow(10,(r()*40)|0)])
const b=Buffer.alloc(xs.length*24);xs.forEach(([y,x],i)=>{b.writeDoubleLE(y,i*24);b.writeDoubleLE(x,i*24+8);b.writeDoubleLE(Math.atan2(y,x),i*24+16)})
process.stdout.write(b)"#;
    let out = Command::new("node").args(["-e", script]).output().unwrap();
    assert!(out.status.success());
    let b = out.stdout;
    let f = |k: usize| f64::from_le_bytes(b[k..k + 8].try_into().unwrap());
    let mut bad = 0;
    for i in 0..b.len() / 24 {
        let (y, x, want) = (f(i * 24), f(i * 24 + 8), f(i * 24 + 16));
        let got = js_atan2(y, x);
        if got.to_bits() != want.to_bits() && !(got.is_nan() && want.is_nan()) {
            bad += 1;
            if bad < 5 {
                eprintln!("atan2({y:e}, {x:e}) = {got:e}, node {want:e}");
            }
        }
    }
    assert_eq!(bad, 0, "{bad} of {} differ", b.len() / 24);
}
