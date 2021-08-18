use rand::{rngs::ThreadRng, seq::IteratorRandom, Rng};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    result,
};

use super::error::GameError;
use super::GameSetting;
pub use crate::disaster::Disaster;
use disastle_castle_rust::{Action, Castle, Room};

type Result<T> = result::Result<T, GameError>;

#[derive(Clone)]
pub struct SchrodingerGameState {
    pub castles: HashMap<String, Castle>,
    pub shop: Vec<Box<dyn Room>>,
    pub discard: Vec<Box<dyn Room>>,
    pub possible_rooms: HashSet<Box<dyn Room>>,
    pub previous_disasters: Vec<Box<dyn Disaster>>,
    pub queued_disasters: Vec<Box<dyn Disaster>>,
    pub possible_disasters: HashSet<Box<dyn Disaster>>,
    pub turn_order: Vec<String>,
    pub turn_index: usize,
    pub round: u8,
    pub rng: ThreadRng,
    pub setting: GameSetting,
}

impl SchrodingerGameState {
    pub fn possible_actions(&self, player_secret: &str) -> Vec<Action> {
        if let Some(castle) = self.castles.get(player_secret) {
            if castle.get_damage() != 0 || self.is_turn_player(player_secret) {
                return castle.possible_actions(&self.shop);
            }
        }
        return Vec::new();
    }
    pub fn action(&self, player_secret: &str, action: Action) -> Result<SchrodingerGameState> {
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
    pub fn next_turn(&self) -> SchrodingerGameState {
        let mut game = self.clone();
        game.turn_index += 1;
        if game.turn_index >= game.turn_order.len() {
            game.turn_index = 0;
            game.turn_order.rotate_left(1);
            game = game.next_round()
        }
        game
    }
    pub fn next_round(&self) -> SchrodingerGameState {
        let mut game = self.clone();
        game.round += 1;
        game.discard.append(&mut game.shop);
        let mut disasters = Vec::new();
        let mut redealt = false;
        while game.shop.len() < game.setting.num_shop as usize
            && game.setting.num_disasters as usize
                - game.previous_disasters.len()
                - game.queued_disasters.len()
                - disasters.len()
                > 0
        {
            let num_disasters_left = game.setting.num_disasters as usize
                - game.previous_disasters.len()
                - game.queued_disasters.len()
                - disasters.len();
            if game.rng.gen_ratio(
                num_disasters_left as u32,
                (game.possible_rooms.len() + num_disasters_left) as u32,
            ) {
                let disaster = game
                    .possible_disasters
                    .iter()
                    .choose(&mut game.rng)
                    .unwrap()
                    .clone();
                game.possible_disasters.remove(&disaster);
                disasters.push(disaster);
            } else {
                let room = game
                    .possible_rooms
                    .iter()
                    .choose(&mut game.rng)
                    .unwrap()
                    .clone();
                game.possible_rooms.remove(&room);
                game.shop.push(room);
            }
            if !redealt && disasters.len() > 1 {
                while let Some(disaster) = disasters.pop() {
                    game.possible_disasters.insert(disaster);
                }
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
    fn resolve_disaster(&self, disaster: Box<dyn Disaster>) -> SchrodingerGameState {
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
        if a.get_rooms().len() > b.get_rooms().len() {
            return Ordering::Greater;
        } else if a.get_rooms().len() < b.get_rooms().len() {
            return Ordering::Less;
        } else {
            let (diamond, cross, moon, any) = a.get_links();
            let a_links = diamond + cross + moon + any;
            let (diamond, cross, moon, any) = b.get_links();
            let b_links = diamond + cross + moon + any;
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

impl SchrodingerGameState {
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
        self.turn_order[self.turn_index] == secret
    }
    pub fn get_turn_index(&self) -> usize {
        self.turn_index
    }
    pub fn get_player_turn_index(&self, secret: &str) -> Result<usize> {
        if !self.castles.contains_key(secret) {
            return Err(GameError::InvalidPlayer);
        }
        Ok(self.turn_order.iter().position(|s| s == secret).unwrap())
    }
}
