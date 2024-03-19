use std::{array::TryFromSliceError, num::ParseIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("invalid format character: '%{0}'")]
    InvalidFormatCharacter(char),
    #[error("invalid hex character '{0}'")]
    InvalidHexCharacter(char),
    #[error("invalid format, unmatched character '{0}'")]
    UnmatchedCharacter(String),
    #[error("invalid format placement specifier '{0}%'")]
    InvalidFormatPlacement(char),
    #[error("wildcard cannot precede any bytes or templates")]
    WildcardNotLast,
    #[error("invalid hex number in '{0}'")]
    InvalidHexNumber(String),
}

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("invalid format character: {0}")]
    InvalidFormatCharacter(char),
    #[error("invalid hex character")]
    InvalidHexCharacter(#[from] ParseIntError),
    #[error("format string contains '{0}', but buffer does not contain this value")]
    MissingValue(u8),
    #[error("expected '{0}' but got '{1}'")]
    MismatchedValue(u8, u8),
    #[error("invalid format, unmatched character '{0}'")]
    UnmatchedCharacter(String),
    #[error("data buffer contains unmatched residual data")]
    ResidualData,
    #[error("internal: missing specifier")]
    InternalMissingSpecifier,
    #[error("missing byte %b")]
    UnmatchedByte,
    #[error("missing word %w")]
    UnmatchedWord,
    #[error("missing double %d")]
    UnmatchedDouble,
    #[error("missing quad %q")]
    UnmatchedQuad,
    #[error("missing rest bytes %*")]
    UnmatchedRestBytes,
    //
    #[error("attempted to get data buffer at index {0}, but the value is missing")]
    InternalMissingValueAtIndex(usize),
    #[error("try error")]
    InternalTryError(#[from] TryFromSliceError),
}
