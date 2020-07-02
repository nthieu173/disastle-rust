mod error;
pub use error::ServerError;

use crate::{
    castle::{room::Room, Castle},
    game::{GameLobby, GamePlay, GameState, PlayerState},
};

use rand;
use serde::{self, Deserialize, Serialize};
use serde_json;
use std::{collections::HashMap, result};

type Result<T> = result::Result<T, ServerError>;

struct LocalServer {
    storage: HashMap<u32, String>,
}

impl LocalServer {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
    fn get_lobby(&self, id: u32) -> Option<&GameLobby> {
        if let Some(game) = self.storage.get(&id) {
            let game = serde_json::from_str(game).expect("cannot fail");
            if let GameState::Lobby(lobby) = game {
                return Some(lobby);
            }
        }
        None
    }
    fn get_lobby_mut(&mut self, id: u32) -> Option<&mut GameLobby> {
        if let Some(game) = self.storage.get_mut(&id) {
            if let GameState::Lobby(lobby) = game {
                return Some(lobby);
            }
        }
        None
    }
    fn get_play(&self, id: u32) -> Option<&GamePlay> {
        if let Some(game) = self.storage.get(&id) {
            if let GameState::Play(play) = game {
                return Some(play);
            }
        }
        None
    }
    fn get_play_mut(&mut self, id: u32) -> Option<&mut GamePlay> {
        if let Some(game) = self.storage.get_mut(&id) {
            if let GameState::Play(play) = game {
                return Some(play);
            }
        }
        None
    }
}

impl LocalServer {
    pub fn get_action(&self, action: GetAction) -> Result<String> {
        match action {
            GetAction::Info { id, secret } => {
                if let Some(game) = self.storage.get(&id) {
                    match game {
                        GameState::Lobby(lobby) => {
                            if lobby.is_player(secret) {
                                #[derive(Serialize)]
                                struct LobbyOutput {
                                    players: Vec<String>,
                                    num_safe: u32,
                                    num_shop: u32,
                                    num_disasters: u32,
                                    rooms: Vec<Room>,
                                    disasters: Vec<String>,
                                    locked_disasters: Vec<String>,
                                }
                                Ok(serde_json::to_string(&LobbyOutput {
                                    players: lobby.players_names(),
                                    num_safe: lobby.num_safe,
                                    num_shop: lobby.num_shop,
                                    num_disasters: lobby.num_disasters,
                                    rooms: lobby.rooms.clone(),
                                    disasters: lobby.disasters_names(),
                                    locked_disasters: lobby.locked_disasters_names(),
                                })
                                .expect("cannot fail"))
                            } else {
                                Err(ServerError::InvalidGame)
                            }
                        }
                        GameState::Play(play) => {
                            #[derive(Serialize)]
                            struct PlayOutput {
                                player: PlayerState,
                                turns: Vec<PlayerState>,
                                shop: Vec<Room>,
                                discard: Vec<Room>,
                                previous_disasters: Vec<String>,
                            }
                            if let Some(player) = play.get_player(secret) {
                                Ok(serde_json::to_string(&PlayOutput {
                                    player: player,
                                    turns: play.turns(),
                                    shop: play.shop.clone(),
                                    discard: play.discard.clone(),
                                    previous_disasters: play
                                        .previous_disasters
                                        .iter()
                                        .map(|d| d.name().to_string())
                                        .collect(),
                                })
                                .expect("cannot fail"))
                            } else {
                                Err(ServerError::InvalidGame)
                            }
                        }
                        GameState::End(end) => {
                            Ok(serde_json::to_string(&end).expect("cannot fail"))
                        }
                    }
                } else {
                    Err(ServerError::InvalidGame)
                }
            }
        }
    }
    pub fn post_action(&mut self, action: PostAction) -> Result<String> {
        match action {
            PostAction::Create { name } => {
                let mut lobby = GameLobby::new();
                let secret = lobby.add_player(name)?;
                let id = rand::random();
                self.storage.insert(id, GameState::Lobby(lobby));
                #[derive(Serialize)]
                struct CreateOutput {
                    secret: u32,
                    id: u32,
                };
                Ok(serde_json::to_string(&CreateOutput { secret, id }).expect("cannot fail"))
            }
            PostAction::Join { id, name } => {
                if let Some(lobby) = self.get_lobby_mut(id) {
                    let secret = lobby.add_player(name)?;
                    #[derive(Serialize)]
                    struct JoinOutput {
                        secret: u32,
                    };
                    Ok(serde_json::to_string(&JoinOutput { secret }).expect("cannot fail"))
                } else {
                    Err(ServerError::InvalidGame)
                }
            }
            PostAction::Start { id, secret } => {
                if let Some(game) = self.storage.get_mut(&id) {
                    if let GameState::Lobby(lobby) = game {
                        if lobby.is_admin(secret) {
                            *game = GameState::Play(lobby.start_game()?);
                            return Ok("".to_string());
                        }

                        return Err(ServerError::Permision);
                    }
                }
                Err(ServerError::InvalidGame)
            }
            PostAction::MoveOuter {
                id,
                secret,
                pos_from,
                pos_to,
            } => {
                if let Some(play) = self.get_play_mut(id) {
                    play.move_outer(secret, pos_from, pos_to)?;
                    Ok("".to_string())
                } else {
                    Err(ServerError::InvalidGame)
                }
            }
            PostAction::Place {
                id,
                secret,
                shop_index,
                pos,
            } => {
                if let Some(play) = self.get_play_mut(id) {
                    play.place(secret, shop_index as usize, pos)?;
                    Ok("".to_string())
                } else {
                    Err(ServerError::InvalidGame)
                }
            }
            PostAction::Remove { id, secret, pos } => {
                if let Some(play) = self.get_play_mut(id) {
                    play.remove(secret, pos)?;
                    Ok("".to_string())
                } else {
                    Err(ServerError::InvalidGame)
                }
            }
            PostAction::Swap {
                id,
                secret,
                pos_from,
                pos_to,
            } => {
                if let Some(play) = self.get_play_mut(id) {
                    play.swap(secret, pos_from, pos_to)?;
                    Ok("".to_string())
                } else {
                    Err(ServerError::InvalidGame)
                }
            }
        }
    }
}

#[derive(Deserialize)]
pub enum GetAction {
    Info { id: u32, secret: u32 },
}

#[derive(Deserialize)]
pub enum PostAction {
    Create {
        name: String,
    },
    Join {
        id: u32,
        name: String,
    },
    Start {
        id: u32,
        secret: u32,
    },
    MoveOuter {
        id: u32,
        secret: u32,
        pos_from: (i32, i32),
        pos_to: (i32, i32),
    },
    Place {
        id: u32,
        secret: u32,
        shop_index: u32,
        pos: (i32, i32),
    },
    Remove {
        id: u32,
        secret: u32,
        pos: (i32, i32),
    },
    Swap {
        id: u32,
        secret: u32,
        pos_from: (i32, i32),
        pos_to: (i32, i32),
    },
}
