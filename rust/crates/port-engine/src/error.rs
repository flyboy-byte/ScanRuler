// SPDX-License-Identifier: AGPL-3.0-only
//! Typed CommandError construction/wire fields from the pinned original.
//! Registry normalization, arbitrary thrown-value coercion, jobs and history are
//! separate operations. Message storage preserves all original UTF-16 units.

/// The twelve error codes declared in the original command contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandErrorCode {
    /// No command by that name.
    UnknownCommand,
    /// Input fails its schema or cannot be used as named.
    InvalidInput,
    /// The session is already working.
    Busy,
    /// An operation needs a scan and none is open.
    NoScan,
    /// The named item does not exist.
    NotFound,
    /// The operation cannot apply in the current state.
    InvalidState,
    /// The operation ran without succeeding.
    Failed,
    /// This session lacks a required capability.
    Unavailable,
    /// The named contract has no implementation yet.
    NotImplemented,
    /// Execution exceeded its time; partial changes may remain.
    Timeout,
    /// An application fault.
    Internal,
    /// A loaded page no longer matches the deployed app.
    StalePage,
}
impl CommandErrorCode {
    /// Source union order, also used by the independent test input protocol.
    pub const ALL: [Self; 12] = [
        Self::UnknownCommand,
        Self::InvalidInput,
        Self::Busy,
        Self::NoScan,
        Self::NotFound,
        Self::InvalidState,
        Self::Failed,
        Self::Unavailable,
        Self::NotImplemented,
        Self::Timeout,
        Self::Internal,
        Self::StalePage,
    ];
    /// Original wire spelling. No new code or code remapping is introduced.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownCommand => "unknown_command",
            Self::InvalidInput => "invalid_input",
            Self::Busy => "busy",
            Self::NoScan => "no_scan",
            Self::NotFound => "not_found",
            Self::InvalidState => "invalid_state",
            Self::Failed => "failed",
            Self::Unavailable => "unavailable",
            Self::NotImplemented => "not_implemented",
            Self::Timeout => "timeout",
            Self::Internal => "internal",
            Self::StalePage => "stale_page",
        }
    }
}

/// Host allocation/size failure, distinct from the original command's error code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeError {
    /// Message exceeds the explicit caller limit.
    MessageLimit,
    /// A derived output size cannot fit this platform's usize.
    LengthOverflow,
    /// Fallible storage reservation failed.
    Allocation,
}

/// Owned original code/message envelope, without platform stack/prototype state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandError {
    code: CommandErrorCode,
    message: Vec<u16>,
}
impl CommandError {
    /// Copy all original UTF-16 units, including unpaired surrogates, within a
    /// caller-specified budget. Failure publishes no partially constructed object.
    pub fn from_utf16(
        code: CommandErrorCode,
        message: &[u16],
        max_units: usize,
    ) -> Result<Self, EnvelopeError> {
        if message.len() > max_units {
            return Err(EnvelopeError::MessageLimit);
        }
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(message.len())
            .map_err(|_| EnvelopeError::Allocation)?;
        owned.extend_from_slice(message);
        Ok(Self {
            code,
            message: owned,
        })
    }
    /// Construct from valid UTF-8 without changing normalization or original text.
    pub fn from_utf8(
        code: CommandErrorCode,
        message: &str,
        max_units: usize,
    ) -> Result<Self, EnvelopeError> {
        let count = message.encode_utf16().count();
        if count > max_units {
            return Err(EnvelopeError::MessageLimit);
        }
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(count)
            .map_err(|_| EnvelopeError::Allocation)?;
        owned.extend(message.encode_utf16());
        Ok(Self {
            code,
            message: owned,
        })
    }
    /// The code a caller branches on.
    pub const fn code(&self) -> CommandErrorCode {
        self.code
    }
    /// The constructor's original Error name; not included in toJSON.
    pub const fn name(&self) -> &'static str {
        "CommandError"
    }
    /// Lossless original message, not a replacement-character rendering.
    pub fn message_units(&self) -> &[u16] {
        &self.message
    }
    /// The initial original Error.toString as lossless UTF-16 units.
    pub fn display_units(&self) -> Result<Vec<u16>, EnvelopeError> {
        let extra = self.name().len() + if self.message.is_empty() { 0 } else { 2 };
        let size = extra
            .checked_add(self.message.len())
            .ok_or(EnvelopeError::LengthOverflow)?;
        let mut out = Vec::new();
        out.try_reserve_exact(size)
            .map_err(|_| EnvelopeError::Allocation)?;
        out.extend(self.name().encode_utf16());
        if !self.message.is_empty() {
            out.extend(": ".encode_utf16());
            out.extend_from_slice(&self.message);
        }
        Ok(out)
    }
    /// Exact JSON.stringify(toJSON()) with code then message, without name/stack.
    /// JSON controls and unpaired surrogates are escaped; valid pairs stay Unicode.
    pub fn to_json(&self) -> Result<String, EnvelopeError> {
        let size = self
            .message
            .len()
            .checked_mul(6)
            .and_then(|n| n.checked_add(64))
            .ok_or(EnvelopeError::LengthOverflow)?;
        let mut out = String::new();
        out.try_reserve_exact(size)
            .map_err(|_| EnvelopeError::Allocation)?;
        out.push_str("{\"code\":\"");
        out.push_str(self.code.as_str());
        out.push_str("\",\"message\":\"");
        let mut i = 0;
        while i < self.message.len() {
            let u = self.message[i];
            match u {
                0x22 => out.push_str("\\\""),
                0x5c => out.push_str("\\\\"),
                8 => out.push_str("\\b"),
                9 => out.push_str("\\t"),
                10 => out.push_str("\\n"),
                12 => out.push_str("\\f"),
                13 => out.push_str("\\r"),
                0..=31 => escape_unit(&mut out, u),
                0xd800..=0xdbff => {
                    if let Some(&low) = self
                        .message
                        .get(i + 1)
                        .filter(|&&v| (0xdc00..=0xdfff).contains(&v))
                    {
                        let value =
                            0x10000 + ((u32::from(u) - 0xd800) << 10) + u32::from(low) - 0xdc00;
                        out.push(char::from_u32(value).ok_or(EnvelopeError::LengthOverflow)?);
                        i += 1;
                    } else {
                        escape_unit(&mut out, u);
                    }
                }
                0xdc00..=0xdfff => escape_unit(&mut out, u),
                _ => out.push(char::from_u32(u32::from(u)).ok_or(EnvelopeError::LengthOverflow)?),
            }
            i += 1;
        }
        out.push_str("\"}");
        Ok(out)
    }
}
fn escape_unit(out: &mut String, value: u16) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push_str("\\u");
    for shift in [12, 8, 4, 0] {
        out.push(char::from(HEX[usize::from((value >> shift) & 15)]));
    }
}
impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())?;
        if !self.message.is_empty() {
            f.write_str(": ")?;
            for c in char::decode_utf16(self.message.iter().copied()) {
                write!(f, "{}", c.unwrap_or(char::REPLACEMENT_CHARACTER))?;
            }
        }
        Ok(())
    }
}
impl std::error::Error for CommandError {}
