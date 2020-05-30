use super::Span;
use thiserror::Error;

/// Result.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("escape only char")]
    EscapeOnlyChar(Span),
    #[error("incomplete unicode escape")]
    IncompleteUnicodeEscape(Span),
    #[error("invalid char in unicode escape")]
    InvalidCharInUnicodeEscape(Span),
    #[error("invalid escape")]
    InvalidEscape(Span),
    #[error("lone slash")]
    LoneSlash(Span),
    #[error("surrogate unicode escape")]
    SurrogateUnicodeEscape(Span),
    #[error("out of range unicode escape")]
    OutOfRangeUnicodeEscape(Span),
}
