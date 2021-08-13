mod error;

pub use error::GameError;
mod card;
mod player;

pub use crate::disaster::{Disaster, SimpleDisaster};
use card::Card;
use disastle_castle_rust::{Action, Castle, Room, SimpleRoom};
use player::PlayerInfo;

use rand::{prelude::IteratorRandom, rngs::ThreadRng, seq::SliceRandom};

use std::{collections::HashMap, iter::Iterator, result};

type Result<T> = result::Result<T, GameError>;

#[derive(Clone)]
pub struct GameState {
    players: HashMap<String, PlayerInfo>,
    castles: HashMap<String, Castle>,
    shop: Vec<Box<dyn Room>>,
    discard: Vec<Box<dyn Room>>,
    previous_disasters: Vec<Box<dyn Disaster>>,
    queued_disasters: Vec<Box<dyn Disaster>>,
    deck: Vec<Card>,
    turn_order: Vec<String>,
    turn_index: usize,
    round: u8,
    rng: ThreadRng,
    setting: GameSetting,
}

#[derive(Clone)]
pub struct GameSetting {
    pub num_safe: u8,
    pub num_shop: u8,
    pub num_disasters: u8,
}

impl GameState {
    pub fn new(
        players: Vec<PlayerInfo>,
        rooms: Vec<Box<dyn Room>>,
        disasters: Vec<Box<dyn Disaster>>,
        setting: GameSetting,
    ) -> GameState {
        let mut players_map = HashMap::new();
        for player in players {
            players_map.insert(player.secret.clone(), player);
        }
        let players = players_map;
        let mut thrones = Vec::new();
        let mut rng = rand::thread_rng();
        let mut deck: Vec<Box<dyn Room>> = rooms
            .into_iter()
            .filter(|room| {
                if !room.is_throne() {
                    true
                } else {
                    thrones.push(room.clone());
                    false
                }
            })
            .collect();
        deck.shuffle(&mut rng);
        let mut safe = deck
            .drain(deck.len() - setting.num_safe as usize..)
            .map(|r| Card::Room(r))
            .collect();
        let mut deck: Vec<Card> = deck.into_iter().map(|r| Card::Room(r)).collect();
        let mut disasters: Vec<Card> = disasters
            .into_iter()
            .choose_multiple(&mut rng, setting.num_disasters as usize)
            .into_iter()
            .map(|d| Card::Disaster(d))
            .collect();
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
                }
                Card::Disaster(_) => {
                    unreachable!("Disaster should not be dealt in the first shop");
                }
            }
        }
        let mut thrones: Vec<Box<dyn Room>> =
            thrones.into_iter().choose_multiple(&mut rng, players.len());
        let mut castles = HashMap::new();
        let mut turn_order = Vec::new();
        for secret in players.keys() {
            castles.insert(secret.clone(), Castle::new(thrones.pop().unwrap()));
            turn_order.push(secret.clone());
        }
        turn_order.shuffle(&mut rng);
        GameState {
            players,
            castles,
            shop,
            discard: Vec::new(),
            previous_disasters: Vec::new(),
            queued_disasters: Vec::new(),
            deck,
            turn_order,
            turn_index: 0,
            round: 0,
            rng,
            setting,
        }
    }
    pub fn action(&mut self, player_secret: &str, action: Action) -> Result<GameState> {
        if let Some(castle) = self.castles.get(player_secret) {
            if castle.get_damage() == 0 && !self.is_turn_player(player_secret) {
                return Err(GameError::NotTurnPlayer);
            }
        } else {
            return Err(GameError::InvalidPlayer);
        }
        match action {
            Action::Place(index, pos) => {
                if index >= self.shop.len() {
                    return Err(GameError::InvalidShopIndex);
                }
                let mut game = self.clone();
                let room = game.shop.remove(index);
                game.castles.insert(
                    player_secret.to_string(),
                    game.castles
                        .get(player_secret)
                        .unwrap()
                        .place_room(room, pos)?,
                );
                game = game.next_turn();
                Ok(game)
            }
            Action::Move(from, to) => {
                let mut game = self.clone();
                game.castles.insert(
                    player_secret.to_string(),
                    game.castles
                        .get(player_secret)
                        .unwrap()
                        .move_room(from, to)?,
                );
                game = game.next_turn();
                Ok(game)
            }
            Action::Swap(pos1, pos2) => {
                let mut game = self.clone();
                game.castles.insert(
                    player_secret.to_string(),
                    game.castles
                        .get(player_secret)
                        .unwrap()
                        .swap_room(pos1, pos2)?,
                );
                game = game.next_turn();
                Ok(game)
            }
            Action::Discard(pos) => {
                let mut game = self.clone();
                let (castle, room) = game.castles.get(player_secret).unwrap().discard_room(pos)?;
                game.castles.insert(player_secret.to_string(), castle);
                game.discard.push(room);
                if game.castles.values().all(|c| c.get_damage() == 0)
                    && game.queued_disasters.len() > 0
                {
                    let disaster = game.queued_disasters.pop().unwrap();
                    game = game.resolve_disaster(disaster);
                }
                Ok(game)
            }
        }
    }
    pub fn next_turn(&self) -> GameState {
        let mut game = self.clone();
        game.turn_index += 1;
        if game.turn_index >= game.turn_order.len() {
            game.turn_index = 0;
            game.turn_order.rotate_left(1);
            game = game.next_round()
        }
        game
    }
    pub fn next_round(&self) -> GameState {
        let mut game = self.clone();
        game.round += 1;
        game.discard.append(&mut game.shop);
        let mut disasters = Vec::new();
        let mut redealt = false;
        while game.shop.len() < game.setting.num_shop as usize && game.deck.len() > 0 {
            match game.deck.pop().unwrap() {
                Card::Room(room) => {
                    game.shop.push(room);
                }
                Card::Disaster(disaster) => {
                    disasters.push(disaster);
                }
            }
            if !redealt && disasters.len() > 1 {
                let mut card_disasters = disasters
                    .drain(..disasters.len() - 1)
                    .map(|d| Card::Disaster(d))
                    .collect();
                game.deck.append(&mut card_disasters);
                game.deck.shuffle(&mut game.rng);
                redealt = true;
            }
        }
        if disasters.len() == 0 {
            return game;
        }
        let disaster = disasters.pop().unwrap();
        game = game.resolve_disaster(disaster);
        game.queued_disasters = disasters;
        game
    }
    fn resolve_disaster(&self, disaster: Box<dyn Disaster>) -> GameState {
        let mut game = self.clone();
        let diamond = disaster.diamond_damage(game.previous_disasters.len() as u8);
        let cross = disaster.cross_damage(game.previous_disasters.len() as u8);
        let moon = disaster.moon_damage(game.previous_disasters.len() as u8);
        // Removing lost players from the turn_order
        game.turn_order = game
            .turn_order
            .clone()
            .iter()
            .enumerate()
            .filter_map(|(index, secret)| {
                let castle = game.castles.get_mut(secret).unwrap();
                *castle = castle.deal_damage(diamond, cross, moon);
                if castle.is_lost() {
                    if index < game.turn_index {
                        game.turn_index -= 1;
                    }
                    return None;
                }
                Some((*secret).clone())
            })
            .collect();
        if game.turn_index >= game.turn_order.len() {
            game.turn_index = 0;
        }
        game.previous_disasters.push(disaster);
        game
    }
}

impl GameState {
    pub fn get_setting(&self) -> &GameSetting {
        &self.setting
    }
    pub fn is_player(&self, secret: &str) -> bool {
        self.players.contains_key(secret)
    }
    pub fn is_turn_player(&self, secret: &str) -> bool {
        self.turn_player() == secret
    }
    pub fn turn_player(&self) -> &str {
        &self.turn_order[self.turn_index]
    }
    pub fn view_shop(&self) -> Vec<SimpleRoom> {
        self.shop
            .iter()
            .map(|r| SimpleRoom::from_room(r.as_ref()))
            .collect()
    }
    pub fn view_discard(&self) -> Vec<SimpleRoom> {
        self.discard
            .iter()
            .map(|r| SimpleRoom::from_room(r.as_ref()))
            .collect()
    }
    pub fn view_previous_disasters(&self) -> Vec<SimpleDisaster> {
        self.previous_disasters
            .iter()
            .map(|r| SimpleDisaster::from_disaster(r.as_ref()))
            .collect()
    }
    pub fn view_queued_disasters(&self) -> Vec<SimpleDisaster> {
        self.queued_disasters
            .iter()
            .map(|r| SimpleDisaster::from_disaster(r.as_ref()))
            .collect()
    }
}
