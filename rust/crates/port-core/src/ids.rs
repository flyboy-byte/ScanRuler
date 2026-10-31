// SPDX-License-Identifier: AGPL-3.0-only
//! Identifiers. The original is JavaScript, so ids that cross a project file or the wire
//! must stay inside the JSON-safe integer range; Rust's wider integers must not leak out
//! (ARCHITECTURE.md, "Data and numerical fidelity").

/// Largest integer a JavaScript number holds exactly (2^53 - 1).
pub const MAX_SAFE_ID: u64 = (1 << 53) - 1;

/// An id outside the JSON-safe range was supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdOutOfRange(pub u64);

macro_rules! safe_id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(u64);

        impl $name {
            /// Wraps `raw`, refusing values a JavaScript number cannot hold exactly.
            pub fn new(raw: u64) -> Result<Self, IdOutOfRange> {
                if raw > MAX_SAFE_ID { Err(IdOutOfRange(raw)) } else { Ok(Self(raw)) }
            }

            /// The raw value.
            pub fn get(self) -> u64 {
                self.0
            }
        }
    };
}

safe_id!(
    /// A loaded scan. Not the displayed number and not a kernel handle.
    ScanId
);
safe_id!(
    /// A fitted or constructed element. Not its displayed number.
    ElementId
);
safe_id!(
    /// A reconstruction feature.
    FeatureId
);
safe_id!(
    /// A background job.
    JobId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_safe_boundary() {
        assert!(ElementId::new(MAX_SAFE_ID).is_ok());
        assert_eq!(
            ElementId::new(MAX_SAFE_ID + 1),
            Err(IdOutOfRange(MAX_SAFE_ID + 1))
        );
    }
}
