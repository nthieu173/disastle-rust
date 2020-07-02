use crate::game::GameError;

use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ServerError {
    Permision,
    InvalidGame,
    InvalidAction,
    GameError(GameError),
}

impl From<GameError> for ServerError {
    fn from(error: GameError) -> Self {
        Self::GameError(error)
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Permision => write!(f, "permission error"),
            ServerError::InvalidGame => write!(f, "invalid game error"),
            ServerError::InvalidAction => write!(f, "invalid action error"),
            ServerError::GameError(e) => write!(f, "invalid game error: {}", e),
        }
    }
}

impl Error for ServerError {}
