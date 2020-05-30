//! Note:
//!
//! Only double quoted string can be escaped.
//!
//! Must be escaped:
//! - single-line: quotation mark, backslash, and the control characters other
//!   than tab (U+0000 to U+0008, U+000A to U+001F, U+007F).
//! - multi-line: backslash and the control characters other than tab, line
//!   feed, and carriage return (U+0000 to U+0008, U+000B, U+000C, U+000E to
//!   U+001F, U+007F).

pub use self::{
    error::{Error, Result},
    escape::escape,
    unescape::unescape,
};

/// Flags.
#[derive(Clone, Copy, Debug, Default)]
pub struct Flags {
    pub is_quoted: bool,
    pub has_lf_or_cr: bool,
    // Ascii control characters other than HT, LF, CR.
    pub has_escape: bool,
    pub has_apostrophe: bool,
}

impl Flags {
    pub fn parse(input: &str) -> Self {
        let mut flags = Self::default();
        for c in input.chars() {
            if !flags.is_quoted {
                flags.is_quoted = !c.is_ascii_alphanumeric() && c != '_' && c != '-';
            }
            if !flags.has_lf_or_cr {
                flags.has_lf_or_cr = c == '\n' || c == '\r';
            }
            if !flags.has_escape {
                flags.has_escape = c.is_ascii_control() && c != '\t' && c != '\n' && c != '\r';
            }
            if !flags.has_apostrophe {
                flags.has_apostrophe = c == '\'';
            }
            if flags.is_quoted && flags.has_lf_or_cr && flags.has_escape && flags.has_apostrophe {
                break;
            }
        }
        flags
    }
}

/// Mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    SingleLine,
    MultiLine,
}

/// A span, designating a range of bytes where a char is located.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    /// The start of the range.
    pub start: usize,
    /// The end of the range (exclusive).
    pub end: usize,
}

impl Span {
    pub fn new() -> Span {
        Self { start: 0, end: 0 }
    }
}

mod error;
mod escape;
mod unescape;
