// SPDX-License-Identifier: AGPL-3.0-only
//! Shared independently authored native sphere input model; no reference outputs.
pub struct Model {
    pub id: String,
    pub p: Vec<f32>,
    pub idx: Vec<u32>,
    pub k: f64,
    pub held: Option<f64>,
}
fn number(s: &str) -> f64 {
    match s {
        "nan" => f64::NAN,
        "inf" => f64::INFINITY,
        "-inf" => f64::NEG_INFINITY,
        _ => s.parse().unwrap(),
    }
}
pub fn model(line: &str) -> Model {
    let fields: Vec<_> = line.split('\t').collect();
    let mode: usize = fields[1].parse().unwrap();
    let seed: u32 = fields[2].parse().unwrap();
    let mut state = seed;
    let mut p: Vec<f64> = vec![];
    let mut next = || {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        state
    };
    match mode {
        0 => {}
        1 => p = vec![0., 0., 0., 1., 0., 0., 0., 1., 0.],
        2 => p = vec![1.; 48],
        3 => {
            for i in 0..20 {
                let i = f64::from(i);
                p.extend([i, 2. * i, -3. * i])
            }
        }
        4 => {
            for i in 0..32 {
                p.extend([f64::from(i % 8), f64::from(i / 8), 5.])
            }
        }
        5 => p = vec![1., 1., 1., 1., -1., -1., -1., 1., -1., -1., -1., 1.],
        13 => {
            p = vec![
                10., 0., 0., -10., 0., 0., 0., 10., 0., 0., -10., 0., 0., 0., 10., 0., 0., -10.,
                0., 0., 0.,
            ]
        }
        _ => {
            let radius = if mode == 11 {
                1e-7
            } else if mode == 12 {
                1e12
            } else {
                10.
            };
            let center = if mode == 10 {
                [1e6, -2e6, 3e6]
            } else {
                [13., -7., 3.]
            };
            for i in 0..128 {
                let x = f64::from(next() % 2049) - 1024.;
                let y = f64::from(next() % 2049) - 1024.;
                let v = f64::from(next() % 2049) - 1024.;
                let z = if mode == 8 { v.abs() + 2048. } else { v };
                let norm = (x * x + y * y + z * z).sqrt();
                let mut r = radius;
                if [7, 8, 9, 15].contains(&mode) {
                    r += (f64::from(next() % 1001) - 500.) / 20000.
                }
                if mode == 9 && i % 17 == 0 {
                    r += 2.
                }
                p.extend([
                    center[0] + r * x / norm,
                    center[1] + r * y / norm,
                    center[2] + r * z / norm,
                ]);
            }
        }
    }
    let mut p: Vec<f32> = p.into_iter().map(|n| n as f32).collect();
    if mode == 14 {
        p[0] = match seed {
            1 => f32::NAN,
            7 => f32::INFINITY,
            99 => f32::NEG_INFINITY,
            _ => f32::MAX,
        }
    }
    let mut idx: Vec<u32> = (0..p.len() as u32 / 3).collect();
    if mode == 15 {
        idx.retain(|i| i % 3 != 0);
        idx.reverse();
        idx.extend([1, 1, 2])
    }
    Model {
        id: fields[0].into(),
        p,
        idx,
        k: number(fields[3]),
        held: if fields[4] == "none" {
            None
        } else {
            Some(number(fields[4]))
        },
    }
}
pub fn packet(m: &Model) -> Vec<u8> {
    let mut b = vec![];
    b.extend((m.p.len() as u32).to_le_bytes());
    b.extend((m.idx.len() as u32).to_le_bytes());
    b.extend(m.k.to_le_bytes());
    b.push(u8::from(m.held.is_some()));
    if let Some(held) = m.held {
        b.extend(held.to_le_bytes())
    }
    for p in &m.p {
        b.extend(p.to_le_bytes())
    }
    for i in &m.idx {
        b.extend(i.to_le_bytes())
    }
    b
}
