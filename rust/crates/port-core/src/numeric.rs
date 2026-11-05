// SPDX-License-Identifier: AGPL-3.0-only
//! Internal pure-Rust platform numeric primitives.

/// Scaled compensated three-component norm at the source Math.hypot boundary.
pub(crate) fn norm3(v: [f64; 3]) -> f64 {
    let abs = v.map(f64::abs);
    let max = abs[0].max(abs[1]).max(abs[2]);
    if max == 0. {
        0.
    } else if max.is_infinite() {
        f64::INFINITY
    } else if abs.iter().any(|x| x.is_nan()) {
        f64::NAN
    } else {
        let mut sum = 0.;
        let mut compensation = 0.;
        for x in abs {
            let scaled = x / max;
            let summand = scaled * scaled - compensation;
            let preliminary = sum + summand;
            compensation = (preliminary - sum) - summand;
            sum = preliminary;
        }
        sum.sqrt() * max
    }
}

/// Scaled compensated norm of any number of components at the source Math.hypot
/// boundary (the two-argument calls in the circle and cylinder kernels).
pub(crate) fn hypot(v: &[f64]) -> f64 {
    let mut max = 0f64;
    let mut nan = false;
    for &x in v {
        let a = x.abs();
        if a.is_infinite() {
            return f64::INFINITY;
        }
        if a.is_nan() {
            nan = true;
        } else if a > max {
            max = a;
        }
    }
    if nan {
        return f64::NAN;
    }
    if max == 0. {
        return 0.;
    }
    let mut sum = 0.;
    let mut compensation = 0.;
    for &x in v {
        let scaled = x.abs() / max;
        let summand = scaled * scaled - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

// V8's Math.atan2 (src/base/ieee754.cc) is FreeBSD msun's fdlibm e_atan2.c and
// s_atan.c; Rust's f64::atan2 calls the platform libm and differs in the last
// bit for about one input in six. Written from the fdlibm algorithm (Sun's
// permissive notice), verified bit-for-bit against Node 22's Math.atan2.
#[allow(clippy::excessive_precision, clippy::approx_constant)] // fdlibm constants, verbatim.
const ATAN_HI: [f64; 4] = [
    4.636_476_090_008_060_935_15e-1,
    7.853_981_633_974_482_789_99e-1,
    9.827_937_232_473_290_540_82e-1,
    1.570_796_326_794_896_558_00e0,
];
#[allow(clippy::excessive_precision, clippy::approx_constant)] // fdlibm constants, verbatim.
const ATAN_LO: [f64; 4] = [
    2.269_877_745_296_168_709_24e-17,
    3.061_616_997_868_383_017_93e-17,
    1.390_331_103_123_099_845_16e-17,
    6.123_233_995_736_766_035_87e-17,
];
#[allow(clippy::excessive_precision, clippy::approx_constant)] // fdlibm constants, verbatim.
const AT: [f64; 11] = [
    3.333_333_333_333_293_180_27e-1,
    -1.999_999_999_987_648_324_76e-1,
    1.428_571_427_250_346_637_11e-1,
    -1.111_111_040_546_235_578_80e-1,
    9.090_887_133_436_506_561_96e-2,
    -7.691_876_205_044_829_994_95e-2,
    6.661_073_137_387_531_206_69e-2,
    -5.833_570_133_790_573_486_45e-2,
    4.976_877_994_615_932_360_17e-2,
    -3.653_157_274_421_691_552_70e-2,
    1.628_582_011_536_578_236_23e-2,
];

fn high(x: f64) -> i32 {
    (x.to_bits() >> 32) as u32 as i32
}
fn low(x: f64) -> u32 {
    x.to_bits() as u32
}

/// fdlibm `atan`, as V8's `ieee754::atan`.
pub fn js_atan(x: f64) -> f64 {
    let hx = high(x);
    let ix = hx & 0x7fff_ffff;
    let mut x = x;
    let id: i32;
    if ix >= 0x4410_0000 {
        if ix > 0x7ff0_0000 || (ix == 0x7ff0_0000 && low(x) != 0) {
            return x + x;
        }
        return if hx > 0 {
            ATAN_HI[3] + ATAN_LO[3]
        } else {
            -ATAN_HI[3] - ATAN_LO[3]
        };
    }
    if ix < 0x3fdc_0000 {
        if ix < 0x3e40_0000 && 1.0e300 + x > 1. {
            return x;
        }
        id = -1;
    } else {
        x = x.abs();
        if ix < 0x3ff3_0000 {
            if ix < 0x3fe6_0000 {
                id = 0;
                x = (2. * x - 1.) / (2. + x);
            } else {
                id = 1;
                x = (x - 1.) / (x + 1.);
            }
        } else if ix < 0x4003_8000 {
            id = 2;
            x = (x - 1.5) / (1. + 1.5 * x);
        } else {
            id = 3;
            x = -1. / x;
        }
    }
    let z = x * x;
    let w = z * z;
    let s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
    let s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
    if id < 0 {
        return x - x * (s1 + s2);
    }
    let i = id as usize;
    let z = ATAN_HI[i] - ((x * (s1 + s2) - ATAN_LO[i]) - x);
    if hx < 0 { -z } else { z }
}

/// fdlibm `atan2`, as V8's `Math.atan2`.
#[allow(clippy::excessive_precision, clippy::approx_constant)] // fdlibm constants, verbatim.
pub fn js_atan2(y: f64, x: f64) -> f64 {
    const TINY: f64 = 1.0e-300;
    const PI_O_4: f64 = 7.853_981_633_974_482_790_0e-1;
    const PI_O_2: f64 = 1.570_796_326_794_896_558_0e0;
    const PI: f64 = 3.141_592_653_589_793_116_0e0;
    const PI_LO: f64 = 1.224_646_799_147_353_177_2e-16;
    let (hx, lx) = (high(x), low(x));
    let ix = hx & 0x7fff_ffff;
    let (hy, ly) = (high(y), low(y));
    let iy = hy & 0x7fff_ffff;
    let nonzero = |l: u32| i32::from(l != 0);
    if (ix | nonzero(lx)) > 0x7ff0_0000 || (iy | nonzero(ly)) > 0x7ff0_0000 {
        return x + y;
    }
    if hx == 0x3ff0_0000 && lx == 0 {
        return js_atan(y);
    }
    let mut m = ((hy >> 31) & 1) | ((hx >> 30) & 2);
    if (iy as u32 | ly) == 0 {
        return match m {
            0 | 1 => y,
            2 => PI + TINY,
            _ => -PI - TINY,
        };
    }
    if (ix as u32 | lx) == 0 {
        return if hy < 0 {
            -PI_O_2 - TINY
        } else {
            PI_O_2 + TINY
        };
    }
    if ix == 0x7ff0_0000 {
        if iy == 0x7ff0_0000 {
            return match m {
                0 => PI_O_4 + TINY,
                1 => -PI_O_4 - TINY,
                2 => 3. * PI_O_4 + TINY,
                _ => -3. * PI_O_4 - TINY,
            };
        }
        return match m {
            0 => 0.,
            1 => -0.,
            2 => PI + TINY,
            _ => -PI - TINY,
        };
    }
    if iy == 0x7ff0_0000 {
        return if hy < 0 {
            -PI_O_2 - TINY
        } else {
            PI_O_2 + TINY
        };
    }
    let k = (iy - ix) >> 20;
    let z = if k > 60 {
        m &= 1;
        PI_O_2 + 0.5 * PI_LO
    } else if hx < 0 && k < -60 {
        0.
    } else {
        js_atan((y / x).abs())
    };
    match m {
        0 => z,
        1 => -z,
        2 => PI - (z - PI_LO),
        _ => (z - PI_LO) - PI,
    }
}
