mod card;
mod error;
mod schrodinger;

use rand::{prelude::IteratorRandom, seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    hash::Hash,
    iter::Iterator,
    result,
};

pub use error::GameError;

pub use crate::disaster::Disaster;
use card::Card;
use disastle_castle_rust::{Action, Castle, Room};
pub use schrodinger::SchrodingerGameState;

type Result<T> = result::Result<T, GameError>;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GameState {
    pub shop: Vec<Room>,
    pub discard: Vec<Room>,
    pub previous_disasters: Vec<Disaster>,
    pub queued_disasters: Vec<Disaster>,
    pub round: u8,
    pub setting: GameSetting,
    castles: BTreeMap<String, Castle>,
    deck: Vec<Card>,
    turn_order: Vec<String>,
    turn_index: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GameSetting {
    pub num_safe: u8,
    pub num_shop: u8,
    pub num_disasters: u8,
    pub thrones: BTreeSet<Room>,
    pub rooms: BTreeSet<Room>,
    pub disasters: BTreeSet<Disaster>,
}

impl GameState {
    pub fn new(players: Vec<String>, setting: GameSetting) -> GameState {
        let mut rng = rand::thread_rng();
        let mut deck: Vec<Room> = setting.rooms.clone().into_iter().collect();
        deck.shuffle(&mut rng);
        let mut safe = deck
            .drain(deck.len() - setting.num_safe as usize..)
            .map(|r| Card::Room(r))
            .collect();
        let mut deck: Vec<Card> = deck.into_iter().map(|r| Card::Room(r)).collect();
        let mut disasters: Vec<Card> = setting
            .clone()
            .disasters
            .into_iter()
            .choose_multiple(&mut rng, setting.num_disasters as usize)
            .into_iter()
            .map(|d| Card::Disaster(d))
            .collect();
        deck.append(&mut disasters);
        deck.shuffle(&mut rng);
        deck.append(&mut safe);
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
        let mut thrones: Vec<Room> = setting
            .thrones
            .clone()
            .into_iter()
            .choose_multiple(&mut rng, players.len());
        let mut castles = BTreeMap::new();
        let mut turn_order = Vec::new();
        for secret in players {
            castles.insert(secret.clone(), Castle::new(thrones.pop().unwrap()));
            turn_order.push(secret);
        }
        turn_order.shuffle(&mut rng);
        GameState {
            castles,
            shop,
            discard: Vec::new(),
            previous_disasters: Vec::new(),
            queued_disasters: Vec::new(),
            deck,
            turn_order,
            turn_index: 0,
            round: 0,
            setting,
        }
    }
    pub fn to_schrodinger(&self) -> SchrodingerGameState {
        let mut new_turn_order = Vec::new();
        let mut new_castles = BTreeMap::new();
        let mut possible_rooms = self.setting.rooms.clone();
        for room in self.discard.iter() {
            possible_rooms.remove(room);
        }
        for (index, secret) in self.turn_order.iter().enumerate() {
            new_turn_order.push(index.to_string());
            let castle = self.castles.get(secret).unwrap().clone();
            for room in castle.rooms.values() {
                possible_rooms.remove(room);
            }
            new_castles.insert(index.to_string(), castle);
        }
        let mut possible_disasters = self.setting.disasters.clone();
        for disaster in self.previous_disasters.iter() {
            possible_disasters.remove(disaster);
        }
        for disaster in self.queued_disasters.iter() {
            possible_disasters.remove(disaster);
        }
        SchrodingerGameState {
            castles: new_castles,
            shop: self.shop.clone(),
            discard: self.discard.clone(),
            possible_rooms,
            previous_disasters: self.previous_disasters.clone(),
            queued_disasters: self.queued_disasters.clone(),
            possible_disasters,
            turn_order: new_turn_order,
            turn_index: self.turn_index,
            round: self.round,
            setting: self.setting.clone(),
        }
    }
    pub fn possible_actions(&self, player_secret: &str) -> Vec<Action> {
        if let Some(castle) = self.castles.get(player_secret) {
            if castle.damage != 0 || self.is_turn_player(player_secret) {
                return castle.possible_actions(&self.shop);
            }
        }
        return Vec::new();
    }
    pub fn action(&self, player_secret: &str, action: Action) -> Result<GameState> {
        if let Some(castle) = self.castles.get(player_secret) {
            if castle.damage == 0 && !self.is_turn_player(player_secret) {
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
                let (mut castle, room) = game.castles[player_secret].discard_room(pos)?;
                game.discard.push(room);
                if castle.is_lost() {
                    // Castle has discarded its last throne room
                    // Removing lost players from the turn_order
                    let index = game.get_player_turn_index(player_secret).unwrap();
                    game.turn_order.remove(index);
                    if index < game.turn_index {
                        game.turn_index -= 1;
                    }
                    if game.turn_index >= game.turn_order.len() {
                        game.round += 1;
                        game.turn_index = 0;
                    }
                    castle = castle.clear_rooms();
                    castle.damage = 0;
                }
                game.castles.insert(player_secret.to_string(), castle);
                if game.castles.values().all(|c| c.damage == 0 || c.is_lost())
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
                game.deck.shuffle(&mut thread_rng());
                redealt = true;
            }
        }
        if let Some(disaster) = disasters.pop() {
            game = game.resolve_disaster(disaster);
            game.queued_disasters = disasters;
        }
        game
    }
    fn resolve_disaster(&self, disaster: Disaster) -> GameState {
        let mut game = self.clone();
        let diamond = disaster.diamond_damage(game.previous_disasters.len() as u8);
        let cross = disaster.cross_damage(game.previous_disasters.len() as u8);
        let moon = disaster.moon_damage(game.previous_disasters.len() as u8);
        // Removing lost players from the turn_order
        game.turn_order = game
            .turn_order
            .clone()
            .into_iter()
            .enumerate()
            .filter_map(|(index, secret)| {
                let castle = game.castles.get_mut(&secret).unwrap();
                *castle = castle.deal_damage(diamond, cross, moon);
                if castle.is_lost() {
                    for room in castle.rooms.values() {
                        game.discard.push(room.clone());
                    }
                    *castle = castle.clear_rooms();
                    if index < game.turn_index {
                        game.turn_index -= 1;
                    }
                    return None;
                }
                Some(secret)
            })
            .collect();
        if game.turn_index >= game.turn_order.len() {
            game.round += 1;
            game.turn_index = 0;
        }
        game.previous_disasters.push(disaster);
        game
    }
}

fn compare_game_state(a: &Castle, b: &Castle) -> Ordering {
    if !a.is_lost() && b.is_lost() {
        return Ordering::Greater;
    } else if a.is_lost() && !b.is_lost() {
        return Ordering::Less;
    } else if a.get_treasure() > b.get_treasure() {
        return Ordering::Greater;
    } else if a.get_treasure() < b.get_treasure() {
        return Ordering::Less;
    } else {
        if a.rooms.len() > b.rooms.len() {
            return Ordering::Greater;
        } else if a.rooms.len() < b.rooms.len() {
            return Ordering::Less;
        } else {
            let (diamond, cross, moon, wild) = a.get_links();
            let a_links = diamond + cross + moon + wild;
            let (diamond, cross, moon, wild) = b.get_links();
            let b_links = diamond + cross + moon + wild;
            if a_links > b_links {
                return Ordering::Greater;
            } else if a_links < b_links {
                return Ordering::Less;
            } else {
                return Ordering::Equal;
            }
        }
    }
}

impl GameState {
    pub fn is_over(&self) -> bool {
        self.turn_order.len() <= 1
            || self.previous_disasters.len() == self.setting.num_disasters as usize
    }
    pub fn is_victorious(&self, secret: &str) -> bool {
        let mut castles: Vec<(&String, &Castle)> = self.castles.iter().collect();
        castles.sort_unstable_by(|(_, a), (_, b)| compare_game_state(b, a)); // Reversed for descending order
        let winner = castles.first();
        if winner.is_none() {
            return false;
        }
        let winner = winner.unwrap().clone();
        let winners: Vec<(&String, &Castle)> = castles
            .into_iter()
            .clone()
            .filter(|(_, castle)| matches!(compare_game_state(castle, winner.1), Ordering::Equal))
            .collect();
        winners.iter().any(|(s, _)| s == &secret)
    }
    pub fn is_player(&self, secret: &str) -> bool {
        self.castles.contains_key(secret)
    }
    pub fn is_turn_player(&self, secret: &str) -> bool {
        if self.turn_order.len() > 0 && self.turn_order[self.turn_index] == secret {
            true
        } else if let Some(castle) = self.castles.get(secret) {
            castle.damage > 0 && !castle.is_lost()
        } else {
            false
        }
    }
    pub fn get_player_turn_index(&self, secret: &str) -> Option<usize> {
        self.turn_order.iter().position(|s| s == secret)
    }
}
