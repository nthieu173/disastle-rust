use disastle_castle_rust::CastleError;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum GameError {
    FullPlayers,
    InvalidAction,
    InvalidPlayer,
    InvalidDisaster,
    InvalidShopIndex,
    InvalidRoomIndex,
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
            GameError::FullPlayers => write!(f, "full players error"),
            GameError::InvalidAction => write!(f, "invalid action error"),
            GameError::InvalidPlayer => write!(f, "There is no player with matching secret in game."),
            GameError::InvalidDisaster => write!(f, "invalid disaster error"),
            GameError::InvalidShopIndex => write!(f, "invalid shop index error"),
            GameError::InvalidRoomIndex => write!(f, "invalid room index error"),
            GameError::CastleError(e) => write!(f, "castle error: {}", e),
        }
    }
}

impl Error for GameError {}
