use std::{error::Error, fmt};

#[derive(Debug)]
pub enum CastleError {
    InvalidPos,
    InvalidMove,
    InvalidPlace,
    InvalidRemove,
    InvalidSwap,
}

impl fmt::Display for CastleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CastleError::InvalidPos => write!(f, "invalid position error"),
            CastleError::InvalidMove => write!(f, "invalid move error"),
            CastleError::InvalidPlace => write!(f, "invalid place error"),
            CastleError::InvalidRemove => write!(f, "invalid remove error"),
            CastleError::InvalidSwap => write!(f, "invalid swap error"),
        }
    }
}

impl Error for CastleError {}
