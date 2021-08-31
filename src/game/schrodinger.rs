use rand::{seq::IteratorRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    hash::Hash,
    result,
};

use super::error::GameError;
use super::GameSetting;
pub use crate::disaster::Disaster;
use disastle_castle_rust::{Action, Castle, Room};

type Result<T> = result::Result<T, GameError>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchrodingerGameState {
    pub shop: Vec<Room>,
    pub discard: Vec<Room>,
    pub previous_disasters: Vec<Disaster>,
    pub queued_disasters: Vec<Disaster>,
    pub round: u8,
    pub setting: GameSetting,
    pub castles: BTreeMap<String, Castle>,
    pub turn_order: Vec<String>,
    pub turn_index: usize,
    pub possible_rooms: BTreeSet<Room>,
    pub possible_disasters: BTreeSet<Disaster>,
}

impl SchrodingerGameState {
    pub fn all_players_possible_actions(&self) -> Vec<(String, Action)> {
        self.turn_order
            .iter()
            .map(|secret| {
                self.possible_actions(secret)
                    .into_iter()
                    .map(move |action| (secret.clone(), action))
            })
            .flatten()
            .collect()
    }
    pub fn possible_actions(&self, player_secret: &str) -> Vec<Action> {
        if let Some(castle) = self.castles.get(player_secret) {
            if castle.damage != 0 || self.is_turn_player(player_secret) {
                return castle.possible_actions(&self.shop);
            }
        }
        return Vec::new();
    }
    pub fn action(&self, player_secret: &str, action: Action) -> Result<SchrodingerGameState> {
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
                    game.castles[player_secret].place_room(room, pos)?,
                );
                game = game.next_turn();
                Ok(game)
            }
            Action::Move(from, to) => {
                let mut game = self.clone();
                game.castles.insert(
                    player_secret.to_string(),
                    game.castles[player_secret].move_room(from, to)?,
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
            let num_disasters_left = if (game.setting.num_safe as usize)
                > self.setting.rooms.len() - self.possible_rooms.len()
            {
                0 // Still safe rooms left
            } else {
                game.setting.num_disasters as usize
                    - game.previous_disasters.len()
                    - game.queued_disasters.len()
                    - disasters.len()
            };
            if thread_rng().gen_ratio(
                num_disasters_left as u32,
                (game.possible_rooms.len() + num_disasters_left) as u32,
            ) {
                let disaster = game
                    .possible_disasters
                    .iter()
                    .choose(&mut thread_rng())
                    .unwrap()
                    .clone();
                game.possible_disasters.remove(&disaster);
                disasters.push(disaster);
            } else {
                let room = game
                    .possible_rooms
                    .iter()
                    .choose(&mut thread_rng())
                    .unwrap()
                    .clone();
                game.possible_rooms.remove(&room);
                game.shop.push(room);
            }
            if !redealt && disasters.len() > 1 {
                // Reshuffle all but the first disaster
                for disaster in disasters.drain(..disasters.len() - 1) {
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
    fn resolve_disaster(&self, disaster: Disaster) -> SchrodingerGameState {
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
        if self.turn_order[self.turn_index] == secret {
            true
        } else if let Some(castle) = self.castles.get(secret) {
            castle.damage > 0
        } else {
            false
        }
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
