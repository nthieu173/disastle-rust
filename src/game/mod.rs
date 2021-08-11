mod error;

pub use error::GameError;
mod card;
mod player;

use player::PlayerInfo;
use card::Card;
use disastle_castle_rust::{Castle, Room, Pos};
use crate::disaster::Disaster;

use rand::{rngs::ThreadRng, seq::SliceRandom};

use std::{
    collections::{HashMap},
    iter::{FromIterator, Iterator},
    result,
};

type Result<T> = result::Result<T, GameError>;

pub struct GameState {
    pub players: HashMap<String, PlayerInfo>,
    pub shop: Vec<Box<dyn Room>>,
    pub discard: Vec<Box<dyn Room>>,
    pub previous_disasters: Vec<Box<dyn Disaster>>,
    turn_order: Vec<String>,
    turn_index: usize,
    deck: Vec<Card>,
    rng: ThreadRng,
    setting: GameSetting,
}

pub struct GameSetting {
    pub num_safe: u32,
    pub num_shop: u32,
    pub num_disasters: u32,
}

impl GameState {
    pub fn new(players: Vec<PlayerInfo>, cards: Vec<Card>, setting: GameSetting) -> GameState {
        let mut players_map = HashMap::new();
        for player in players {
            players_map.insert(player.secret.clone(), player);
        }
        let players = players_map;
        let mut thrones = Vec::new();
        let mut deck = Vec::new();
        let mut disasters = Vec::new();
        for card in cards {
            match card {
                Card::Room(room) => {
                    if *room.is_throne() {
                        thrones.push(room);
                    } else {
                        deck.push(room);
                    }
                },
                Card::Disaster(disaster) => disasters.push(disaster),
            }
        }
        let mut rng = rand::thread_rng();
        disasters.shuffle(&mut rng);
        disasters.truncate(setting.num_disasters as usize);
        let mut card_disaster = Vec::new();
        for disaster in disasters {
            card_disaster.push(Card::Disaster(disaster));
        }
        let mut disasters = card_disaster;
        deck.shuffle(&mut rng);
        let mut safe = deck.drain(deck.len()-setting.num_safe as usize..).map(|r| Card::Room(r)).collect();
        let mut card_deck = Vec::new();
        for card in deck {
            card_deck.push(Card::Room(card));
        }
        let mut deck = card_deck;
        deck.append(&mut disasters);
        drop(disasters);
        deck.shuffle(&mut rng);
        deck.append(&mut safe);
        drop(safe);
        let mut shop = Vec::new();
        for _ in 0..setting.num_shop as usize {
            match deck.pop().unwrap() {
                Card::Room(room) => {
                    shop.push(room);
                },
                Card::Disaster(_) => {
                    unreachable!("Disaster should not be dealt in the first shop");
                }
            }
        }
        let mut turn_order: Vec<String> = players.keys().map(|p| (*p).clone()).collect();
        turn_order.shuffle(&mut rng);
        GameState {
            players,
            shop,
            discard: Vec::new(),
            previous_disasters: Vec::new(),
            deck,
            turn_order,
            turn_index: 0,
            rng,
            setting
        }
    }
}

impl GameState {
    pub fn is_player(&self, secret: &str) -> bool {
        self.players.contains_key(secret)
    }
    pub fn turn_player(&self) -> &str {
        &self.turn_order[self.turn_index]
    }
    pub fn next_turn(&mut self) {
        self.turn_index += 1;
        if self.turn_index >= self.turn_order.len() {
            self.turn_index = 0;
            self.turn_order.rotate_left(1);
        }
    }
}