use disastle_castle_rust::CastleError;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum GameError {
    InvalidPlayer,
    NotTurnPlayer,
    InvalidShopIndex,
    CastleError(CastleError),
}

impl From<CastleError> for GameError {
    fn from(error: CastleError) -> Self {
        Self::CastleError(error)
    }
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::InvalidPlayer => {
                write!(f, "There is no player with matching secret in game.")
            }
            GameError::NotTurnPlayer => {
                write!(f, "It is not the turn of the player yet.")
            }
            GameError::InvalidShopIndex => write!(f, "Shop index is out of bounds"),
            GameError::CastleError(e) => write!(f, "Castle error: {}", e),
        }
    }
}

impl Error for GameError {}
