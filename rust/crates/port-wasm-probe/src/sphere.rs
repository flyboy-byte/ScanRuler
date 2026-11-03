// SPDX-License-Identifier: AGPL-3.0-only
//! Input-only fixed-selection sphere packet. No reference-derived values.
use super::{Input, MAX_POINTS, Output};
fn sphere(s: port_core::sphere::Sphere, out: &mut Output) -> Result<(), ()> {
    for n in [s.cx, s.cy, s.cz, s.r] {
        out.number(n)?
    }
    Ok(())
}
pub(super) struct Data {
    pub(super) positions: Vec<f32>,
    pub(super) idx: Vec<u32>,
    pub(super) k: f64,
    pub(super) held: Option<f64>,
}
pub(super) fn decode(bytes: &[u8]) -> Result<Data, ()> {
    let mut input = Input::new(bytes);
    let n = input.word()? as usize;
    let selected = input.word()? as usize;
    if selected > MAX_POINTS {
        return Err(());
    }
    let k = input.number()?;
    let tag = *input.bytes.get(input.offset).ok_or(())?;
    input.offset += 1;
    let held = match tag {
        0 => None,
        1 => Some(input.number()?),
        _ => return Err(()),
    };
    let positions = input.floats(n)?;
    let idx = input.indices()?;
    if idx.len() != selected {
        return Err(());
    }
    Ok(Data {
        positions,
        idx,
        k,
        held,
    })
}
pub(super) fn execute(bytes: &[u8]) -> Result<Vec<u8>, ()> {
    let Data {
        positions,
        idx,
        k,
        held,
    } = decode(bytes)?;
    let result = port_core::sphere::fit_sphere_clipped(&positions, &idx, k, held, MAX_POINTS)
        .map_err(|_| ())?;
    let mut out = Output(vec![]);
    out.bytes(&[u8::from(result.is_some())])?;
    if let Some(result) = result {
        sphere(result.sphere, &mut out)?;
        for n in [result.sigma, result.span, result.min, result.max] {
            out.number(n)?
        }
        out.indices(&result.used)?;
        out.bytes(&[u8::from(result.free.is_some())])?;
        if let Some(free) = result.free {
            sphere(free, &mut out)?
        }
    }
    Ok(out.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_refusal_is_distinct_from_malformed_or_unbounded_packets() {
        assert_eq!(execute(&[0; 17]), Ok(vec![0]));
        for n in 0..17 {
            assert!(execute(&vec![0; n]).is_err())
        }
        let mut wrong = vec![0; 17];
        wrong[16] = 2;
        assert!(execute(&wrong).is_err());
        let mut large = vec![0; 17];
        large[..4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(execute(&large).is_err());
        let mut large = vec![0; 17];
        large[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(execute(&large).is_err());
        let mut trailing = vec![0; 17];
        trailing.extend([1, 0, 0, 0]);
        assert!(execute(&trailing).is_err());
        let mut bad = vec![0; 33];
        bad[..4].copy_from_slice(&3u32.to_le_bytes());
        bad[4..8].copy_from_slice(&1u32.to_le_bytes());
        bad[29..33].copy_from_slice(&1u32.to_le_bytes());
        assert!(execute(&bad).is_err());
    }
}
