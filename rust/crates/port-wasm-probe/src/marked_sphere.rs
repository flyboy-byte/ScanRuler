// SPDX-License-Identifier: AGPL-3.0-only
//! Complete marked-sphere values/identity or exact FitError, input only.
use super::{MAX_POINTS, Output, sphere};
use port_core::marked_sphere::{Error, Settings, fit_sphere_on_selection};
use std::sync::Arc;
pub(super) fn execute(bytes: &[u8]) -> Result<Vec<u8>, ()> {
    let data = sphere::decode(bytes)?;
    let region: Arc<[u32]> = data.idx.into();
    let result = fit_sphere_on_selection(
        &data.positions,
        region.clone(),
        Settings {
            sigma: data.k,
            known_diameter: data.held,
        },
        MAX_POINTS,
    );
    let mut out = Output(vec![]);
    match result {
        Ok(r) => {
            out.bytes(&[0])?;
            out.text("sphere")?;
            for n in r.center.into_iter().chain([
                r.radius,
                r.sigma,
                r.form_error,
                r.residual_min,
                r.residual_max,
            ]) {
                out.number(n)?
            }
            out.word(r.used_points)?;
            out.word(r.region_size)?;
            out.bytes(&[u8::from(r.held.is_some())])?;
            if let Some(held) = r.held {
                out.number(held.free_diameter)?;
                out.number(held.shift)?
            }
            out.indices(&r.region)?;
            out.bytes(&[u8::from(Arc::ptr_eq(&r.region, &region))])?
        }
        Err(Error::Source(message)) => {
            out.bytes(&[1])?;
            out.text("Error")?;
            out.bytes(&[1])?;
            out.text("FitError")?;
            out.text(&message)?
        }
        Err(Error::InvalidInput | Error::Allocation) => return Err(()),
    }
    Ok(out.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marked_refusal_is_completed_but_bad_packets_are_host_failures() {
        let error = execute(&[0; 17]).expect("source marking-size refusal");
        assert_eq!(error[0], 1);
        assert!(error.windows(8).any(|w| w == b"FitError"));
        assert!(execute(&[]).is_err());
        let mut bad = vec![0; 17];
        bad[16] = 2;
        assert!(execute(&bad).is_err());
        let mut bad = vec![0; 109];
        bad[..4].copy_from_slice(&3u32.to_le_bytes());
        bad[4..8].copy_from_slice(&20u32.to_le_bytes());
        for at in (29..109).step_by(4) {
            bad[at..at + 4].copy_from_slice(&1u32.to_le_bytes())
        }
        assert!(execute(&bad).is_err());
    }
}
