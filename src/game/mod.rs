mod error;
mod player;
pub use error::GameError;
pub use player::PlayerState;

use crate::{
    castle::{room::Room, Castle},
    disaster::Disaster,
};

use rand::{rngs::ThreadRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, VecDeque},
    iter::{FromIterator, Iterator},
    result,
};

type Result<T> = result::Result<T, GameError>;
type Pos = (i32, i32);

const DEFAULT_NUM_SAFE: u32 = 15;
const DEFAULT_NUM_SHOP: u32 = 5;
const DEFAULT_NUM_DISASTERS: u32 = 6;

#[derive(Clone, Serialize)]
pub enum Card {
    Room(Room),
    Disaster(Box<dyn Disaster>),
}

#[derive(Clone, Deserialize)]
pub enum GameState {
    Lobby(GameLobby),
    Play(GamePlay),
    End(GameEnd),
}

#[derive(Clone, Deserialize)]
pub struct GameLobby {
    players: HashMap<u32, PlayerState>,
    pub num_safe: u32,
    pub num_shop: u32,
    pub num_disasters: u32,
    pub rooms: Vec<Room>,
    disasters: Vec<Box<dyn Disaster>>,
    locked_disasters: Vec<Box<dyn Disaster>>,
    rng: ThreadRng,
}

#[derive(Clone, Deserialize)]
pub struct GamePlay {
    turn: TurnIterator<u32>,
    pub players: HashMap<u32, PlayerState>,
    deck: Vec<Card>,
    dealt_shop: bool,
    num_shop: u32,
    pub shop: Vec<Room>,
    pub discard: Vec<Room>,
    pub previous_disasters: Vec<Box<dyn Disaster>>,
    rng: ThreadRng,
}

#[derive(Clone, Serialize)]
pub struct GameEnd {
    pub players: HashMap<u32, PlayerState>,
}

impl GameLobby {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            num_safe: DEFAULT_NUM_SAFE,
            num_shop: DEFAULT_NUM_SHOP,
            num_disasters: DEFAULT_NUM_DISASTERS,
            rooms: Vec::new(),
            disasters: Vec::new(),
            locked_disasters: Vec::new(),
            rng: rand::thread_rng(),
        }
    }
    pub fn start_game(&mut self) -> Result<GamePlay> {
        let players = self
            .players
            .iter()
            .filter_map(|(k, v)| match v {
                PlayerState::Admin { name } => Some((
                    *k,
                    PlayerState::Wait {
                        name: name.to_string(),
                        castle: Castle::new(Room::throne_room(0, String::from("Throne Room"))),
                    },
                )),
                PlayerState::Lobby { name } => Some((
                    *k,
                    PlayerState::Wait {
                        name: name.to_string(),
                        castle: Castle::new(Room::throne_room(0, String::from("Throne Room"))),
                    },
                )),
                _ => None,
            })
            .collect();
        self.rooms.shuffle(&mut self.rng);
        let mut safe = self
            .rooms
            .split_off(self.num_safe as usize)
            .iter()
            .cloned()
            .map(|r| Card::Room(r))
            .collect();
        let mut deck: Vec<Card> = self.rooms.iter().cloned().map(|r| Card::Room(r)).collect();
        {
            self.disasters.shuffle(&mut self.rng);
            let mut disasters = Vec::new();
            for _ in 0..self.num_disasters {
                if let Some(disaster) = self.locked_disasters.pop() {
                    disasters.push(Card::Disaster(disaster));
                    continue;
                }
                if let Some(disaster) = self.disasters.pop() {
                    if !self
                        .locked_disasters
                        .iter()
                        .any(|d| d.name() == disaster.name())
                    {
                        disasters.push(Card::Disaster(disaster));
                    }
                }
            }
            deck.append(&mut disasters);
        }
        deck.shuffle(&mut self.rng);
        deck.append(&mut safe);
        let mut turn_queue: Vec<u32> = self.players.keys().cloned().collect();
        turn_queue.shuffle(&mut self.rng);
        Ok(GamePlay {
            turn: TurnIterator::<u32>::from_iter(turn_queue),
            players,
            deck,
            dealt_shop: false,
            num_shop: self.num_shop,
            shop: Vec::new(),
            discard: Vec::new(),
            previous_disasters: Vec::new(),
            rng: self.rng,
        })
    }
    pub fn add_player(&mut self, name: String) -> Result<u32> {
        if self.players.len() == 0 {
            let secret = rand::random();
            self.players.insert(secret, PlayerState::Admin { name });
            Ok(secret)
        } else if self.players.len() < 5 {
            let secret = rand::random();
            self.players.insert(secret, PlayerState::Lobby { name });
            Ok(secret)
        } else {
            Err(GameError::FullPlayers)
        }
    }
    pub fn remove_player(&mut self, secret: u32) -> Result<()> {
        if let Some(_) = self.players.remove(&secret) {
            Ok(())
        } else {
            Err(GameError::InvalidPlayer)
        }
    }
    pub fn add_disaster(&mut self, disaster: Box<dyn Disaster>) {
        self.disasters.push(disaster);
    }
    pub fn remove_disaster(&mut self, name: &str) -> Result<()> {
        if let Some((index, _)) = self
            .disasters
            .iter()
            .enumerate()
            .filter(|(_, d)| d.name() == name)
            .next()
        {
            self.disasters.remove(index);
            Ok(())
        } else {
            Err(GameError::InvalidDisaster)
        }
    }
    pub fn lock_disaster(&mut self, name: &str) -> Result<()> {
        if let Some(disaster) = self.disasters.iter().filter(|d| d.name() == name).next() {
            self.locked_disasters.push(disaster.clone());
            Ok(())
        } else {
            Err(GameError::InvalidDisaster)
        }
    }
    pub fn is_player(&self, secret: u32) -> bool {
        self.players.contains_key(&secret)
    }
    pub fn is_admin(&self, secret: u32) -> bool {
        if let Some(player) = self.players.get(&secret) {
            if let PlayerState::Admin { .. } = player {
                return true;
            }
        }
        false
    }
    pub fn players_names(&self) -> Vec<String> {
        self.players
            .values()
            .filter_map(|p| match p {
                PlayerState::Admin { name } => Some(name),
                PlayerState::Lobby { name } => Some(name),
                PlayerState::Wait { name, .. } => Some(name),
                PlayerState::Disaster { name, .. } => Some(name),
                PlayerState::Action { name, .. } => Some(name),
                PlayerState::End { name, .. } => Some(name),
                PlayerState::Dead { name } => Some(name),
            })
            .cloned()
            .collect()
    }
    pub fn disasters_names(&self) -> Vec<String> {
        self.disasters
            .iter()
            .map(|d| d.name().to_string())
            .collect()
    }
    pub fn locked_disasters_names(&self) -> Vec<String> {
        self.locked_disasters
            .iter()
            .map(|d| d.name().to_string())
            .collect()
    }
}

impl GamePlay {
    pub fn update_game_states(&mut self) -> Result<Option<GameEnd>> {
        if self
            .players
            .values()
            .all(|p| matches!(p, PlayerState::Wait{..}))
        {
            if self.turn.is_new_round() && !self.dealt_shop {
                self.discard.append(&mut self.shop);
                let mut mulliganed = false;
                let mut current_disasters = Vec::new();
                while self.shop.len() < self.num_shop as usize {
                    if let Some(card) = self.deck.pop() {
                        match card {
                            Card::Room(room) => self.shop.push(room),
                            Card::Disaster(disaster) => {
                                if current_disasters.len() > 0 && !mulliganed {
                                    let mut cards_shop =
                                        self.shop.iter().cloned().map(|r| Card::Room(r)).collect();
                                    let mut cards_disasters = current_disasters
                                        .iter()
                                        .cloned()
                                        .map(|d| Card::Disaster(d))
                                        .collect();
                                    self.deck.append(&mut cards_shop);
                                    self.deck.append(&mut cards_disasters);
                                    self.deck.shuffle(&mut self.rng);
                                    mulliganed = true;
                                } else {
                                    current_disasters.push(disaster);
                                }
                            }
                        }
                    } else {
                        //Game ends
                        let players =
                            HashMap::from_iter(self.players.iter().filter_map(|(k, p)| match p {
                                PlayerState::Action { name, castle, .. } => Some((
                                    *k,
                                    PlayerState::End {
                                        name: name.clone(),
                                        rooms: castle.num_rooms() as u32,
                                        links: castle.links().0
                                            + castle.links().1
                                            + castle.links().2,
                                        treasures: 0,
                                    },
                                )),
                                PlayerState::Wait { name, castle } => Some((
                                    *k,
                                    PlayerState::End {
                                        name: name.clone(),
                                        rooms: castle.num_rooms() as u32,
                                        links: castle.links().0
                                            + castle.links().1
                                            + castle.links().2,
                                        treasures: 0,
                                    },
                                )),
                                PlayerState::Dead { .. } => Some((*k, p.clone())),
                                _ => None,
                            }));
                        return Ok(Some(GameEnd { players }));
                    }
                }
                self.dealt_shop = true;
                if current_disasters.len() > 0 {
                    for (secret, player) in self.players.iter_mut() {
                        match player {
                            PlayerState::Wait { name, castle } => {
                                let damage = current_disasters[current_disasters.len() - 1]
                                    .damage(self.previous_disasters.len() as u32, castle.links());
                                if damage >= castle.num_rooms() as u32 {
                                    *player = PlayerState::Dead { name: name.clone() };
                                    self.turn.remove_item(*secret);
                                } else {
                                    *player = PlayerState::Disaster {
                                        name: name.clone(),
                                        castle: castle.clone(),
                                        disasters: current_disasters.clone(),
                                        damage,
                                        num_previous_disasters: self.previous_disasters.len()
                                            as u32,
                                        remove_queue: Vec::new(),
                                    };
                                }
                            }
                            PlayerState::Dead { .. } => (),
                            _ => unreachable!("all players should be waiting or dead"),
                        };
                    }
                } else {
                    if let Some(turn_player) = self
                        .players
                        .get_mut(&self.turn.next().expect("never fails"))
                    {
                        if let PlayerState::Wait { name, castle } = turn_player {
                            *turn_player = PlayerState::Action {
                                name: name.clone(),
                                castle: castle.clone(),
                                limbo: Vec::new(),
                            };
                            self.dealt_shop = false;
                        } else {
                            unreachable!("all possible turn players should be waiting");
                        }
                    } else {
                        unreachable!("TurnIterator did not return valid secret");
                    }
                }
            } else {
                if let Some(turn_player) = self
                    .players
                    .get_mut(&self.turn.next().expect("never fails"))
                {
                    if let PlayerState::Wait { name, castle } = turn_player {
                        *turn_player = PlayerState::Action {
                            name: name.clone(),
                            castle: castle.clone(),
                            limbo: Vec::new(),
                        };
                        self.dealt_shop = false;
                    } else {
                        unreachable!("all possible turn players should be waiting");
                    }
                } else {
                    unreachable!("TurnIterator did not return valid secret");
                }
            }
        }
        Ok(None)
    }
    pub fn move_outer(&mut self, sercret: u32, pos_from: Pos, pos_to: Pos) -> Result<()> {
        if let Some(player) = self.players.get_mut(&sercret) {
            if let PlayerState::Action { ref mut castle, .. } = player {
                Ok(castle.move_outer(pos_from, pos_to)?)
            } else {
                Err(GameError::InvalidAction)
            }
        } else {
            Err(GameError::InvalidPlayer)
        }
    }
    pub fn place(&mut self, sercret: u32, shop_index: usize, pos: Pos) -> Result<()> {
        if shop_index < self.shop.len() {
            if let Some(player) = self.players.get_mut(&sercret) {
                if let PlayerState::Action { ref mut castle, .. } = player {
                    Ok(castle.place(self.shop.remove(shop_index), pos)?)
                } else {
                    Err(GameError::InvalidAction)
                }
            } else {
                Err(GameError::InvalidPlayer)
            }
        } else {
            Err(GameError::InvalidShopIndex)
        }
    }
    pub fn remove(&mut self, sercret: u32, pos: Pos) -> Result<()> {
        if let Some(player) = self.players.get_mut(&sercret) {
            if let PlayerState::Disaster {
                name,
                ref mut castle,
                ref mut disasters,
                ref mut remove_queue,
                ref mut num_previous_disasters,
                ref mut damage,
            } = player
            {
                let mut test_castle = castle.clone();
                let mut remove_ok = true;
                for queue_pos in remove_queue.iter() {
                    remove_ok &= matches!(test_castle.remove(*queue_pos), Ok(()))
                }
                if !remove_ok {
                    unreachable!("existing remove queue should be valid");
                }
                remove_ok &= matches!(test_castle.remove(pos), Ok(()));
                if remove_ok {
                    remove_queue.push(pos)
                }
                if remove_queue.len() == *damage as usize {
                    for queue_pos in remove_queue.iter() {
                        match castle.remove(*queue_pos) {
                            Ok(()) => (),
                            Err(_) => unreachable!("already checked in test castle"),
                        }
                    }
                    disasters.pop();
                    *num_previous_disasters += 1;
                    if disasters.len() == 0 {
                        *player = PlayerState::Wait {
                            name: name.clone(),
                            castle: castle.clone(),
                        };
                    } else {
                        *damage = disasters[disasters.len() - 1]
                            .damage(*num_previous_disasters, castle.links())
                    }
                } else if remove_queue.len() > *damage as usize {
                    unreachable!("remove_queue should not be longer than damage")
                }
                Ok(())
            } else {
                Err(GameError::InvalidAction)
            }
        } else {
            Err(GameError::InvalidPlayer)
        }
    }
    pub fn swap(&mut self, sercret: u32, pos_from: Pos, pos_to: Pos) -> Result<()> {
        if let Some(player) = self.players.get_mut(&sercret) {
            if let PlayerState::Action { ref mut castle, .. } = player {
                Ok(castle.swap(pos_from, pos_to)?)
            } else {
                Err(GameError::InvalidAction)
            }
        } else {
            Err(GameError::InvalidPlayer)
        }
    }
    pub fn get_player(&self, secret: u32) -> Option<PlayerState> {
        self.players.get(&secret).cloned()
    }
    pub fn turns(&self) -> Vec<PlayerState> {
        self.turn
            .turns()
            .iter()
            .filter_map(|s| self.players.get(s))
            .cloned()
            .collect()
    }
}

// pub trait VecDequeRandom<T> {
//     fn shuffle(&mut self, rng: &mut dyn RngCore);
// }

// impl<T> VecDequeRandom<T> for VecDeque<T> {
//     fn shuffle(&mut self, rng: &mut dyn RngCore) {
//         for i in 0..self.len() - 2 {
//             self.swap(i, rng.gen_range(0, i + 1));
//         }
//     }
// }

#[derive(Clone)]
struct TurnIterator<T: Clone + PartialEq> {
    turn_index: usize,
    turn_queue: VecDeque<T>,
}

impl<T: Clone + PartialEq> TurnIterator<T> {
    pub fn is_new_round(&self) -> bool {
        self.turn_index == 0
    }
    pub fn remove_item(&mut self, item: T) {
        if let Some(remove_index) =
            self.turn_queue
                .iter()
                .enumerate()
                .find_map(|(i, v)| if v == &item { Some(i) } else { None })
        {
            if remove_index < self.turn_index {
                self.turn_index -= 1;
            }
            self.turn_queue.remove(remove_index);
            if self.turn_index >= self.turn_queue.len() {
                self.turn_index = 0;
                if let Some(first) = self.turn_queue.pop_front() {
                    self.turn_queue.push_back(first);
                }
            }
        }
    }
    pub fn turns(&self) -> Vec<T> {
        self.clone().take(self.turn_queue.len()).collect()
    }
}

impl<T: Clone + PartialEq> FromIterator<T> for TurnIterator<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            turn_index: 0,
            turn_queue: VecDeque::<T>::from_iter(iter),
        }
    }
}

impl<T: Clone + PartialEq> Iterator for TurnIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let turn = self.turn_queue[self.turn_index].clone();
        self.turn_index += 1;
        if self.turn_index >= self.turn_queue.len() {
            self.turn_index = 0;
            if let Some(first) = self.turn_queue.pop_front() {
                self.turn_queue.push_back(first);
            }
        }
        Some(turn)
    }
}
