// SPDX-License-Identifier: AGPL-3.0-only
//! Explicit units. Lengths are millimetres throughout the engine; angles cross an API
//! boundary only as one of these two types, never as a bare `f64`.

/// A length in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Millimetres(pub f64);

/// An angle in radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Radians(pub f64);

/// An angle in degrees.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Degrees(pub f64);

impl Degrees {
    /// Converts to radians.
    pub fn to_radians(self) -> Radians {
        Radians(self.0.to_radians())
    }
}

impl Radians {
    /// Converts to degrees.
    pub fn to_degrees(self) -> Degrees {
        Degrees(self.0.to_degrees())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_round_trip() {
        let back = Degrees(37.5).to_radians().to_degrees();
        assert!((back.0 - 37.5).abs() < 1e-12);
    }
}
