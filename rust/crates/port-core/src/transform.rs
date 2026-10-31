// SPDX-License-Identifier: AGPL-3.0-only
//! Rigid poses: a row-major 3x3 rotation and a translation, both `f64`, as the original
//! stores them. Points the original bakes through a pose are `f32`; that rounding belongs
//! to the mesh code, not here.

/// A rotation (row-major) followed by a translation: `p' = R p + t`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidTransform {
    /// Row-major rotation matrix.
    pub rotation: [[f64; 3]; 3],
    /// Translation in millimetres.
    pub translation: [f64; 3],
}

impl RigidTransform {
    /// The identity pose.
    pub const IDENTITY: Self = Self {
        rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        translation: [0.0; 3],
    };

    /// Applies the pose to a point.
    pub fn apply(&self, p: [f64; 3]) -> [f64; 3] {
        let r = &self.rotation;
        let mut out = self.translation;
        for (row, o) in r.iter().zip(out.iter_mut()) {
            *o += row[0] * p[0] + row[1] * p[1] + row[2] * p[2];
        }
        out
    }

    /// The pose that applies `self` first, then `then`.
    pub fn then(&self, then: &Self) -> Self {
        let mut rotation = [[0.0; 3]; 3];
        for (i, row) in rotation.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3)
                    .map(|k| then.rotation[i][k] * self.rotation[k][j])
                    .sum();
            }
        }
        Self {
            rotation,
            translation: then.apply(self.translation),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rot_z_90() -> RigidTransform {
        RigidTransform {
            rotation: [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
            translation: [0.0; 3],
        }
    }

    #[test]
    fn quarter_turn_moves_x_to_y() {
        let p = rot_z_90().apply([1.0, 0.0, 0.0]);
        assert_eq!(p, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn composition_applies_in_order() {
        let shift = RigidTransform {
            translation: [1.0, 0.0, 0.0],
            ..RigidTransform::IDENTITY
        };
        // shift first, then turn: (0,0,0) -> (1,0,0) -> (0,1,0)
        let both = shift.then(&rot_z_90());
        assert_eq!(both.apply([0.0; 3]), [0.0, 1.0, 0.0]);
    }
}
