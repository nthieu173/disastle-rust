mod card;
mod error;
pub mod player;
mod schrodinger;

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
    iter::Iterator,
    result,
};

use rand::{prelude::IteratorRandom, rngs::ThreadRng, seq::SliceRandom};

pub use error::GameError;

pub use crate::disaster::Disaster;
use card::Card;
use disastle_castle_rust::{Action, Castle, Room};
use player::PlayerInfo;
pub use schrodinger::SchrodingerGameState;

type Result<T> = result::Result<T, GameError>;

#[derive(Clone)]
pub struct GameState {
    castles: HashMap<String, Castle>,
    shop: Vec<Box<dyn Room>>,
    discard: Vec<Box<dyn Room>>,
    deck: Vec<Card>,
    previous_disasters: Vec<Box<dyn Disaster>>,
    queued_disasters: Vec<Box<dyn Disaster>>,
    turn_order: Vec<String>,
    turn_index: usize,
    round: u8,
    rng: ThreadRng,
    setting: GameSetting,
}

impl Hash for GameState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Use the turn_order for a stable hash. If the turn order is different, the game state is probably different.
        for secret in self.turn_order.iter() {
            self.castles.get(secret).unwrap().hash(state);
        }
        self.shop.hash(state);
        self.discard.hash(state);
        self.previous_disasters.hash(state);
        self.queued_disasters.hash(state);
        self.turn_index.hash(state);
        self.round.hash(state);
        self.setting.hash(state);
    }
}

impl PartialEq for GameState {
    fn eq(&self, other: &GameState) -> bool {
        for secret in self.turn_order.iter() {
            let castle = self.castles.get(secret).unwrap();
            if let Some(other_castle) = other.castles.get(secret) {
                if castle != other_castle {
                    return false;
                }
            } else {
                return false;
            }
        }
        self.shop == other.shop
            && self.discard == other.discard
            && self.previous_disasters == other.previous_disasters
            && self.queued_disasters == other.queued_disasters
            && self.turn_index == other.turn_index
            && self.round == other.round
            && self.setting == other.setting
    }
}

impl Eq for GameState {}

#[derive(Clone)]
pub struct GameSetting {
    pub num_safe: u8,
    pub num_shop: u8,
    pub num_disasters: u8,
    pub rooms: HashSet<Box<dyn Room>>,
    pub disasters: HashSet<Box<dyn Disaster>>,
}

impl PartialEq for GameSetting {
    // Let's hope that rooms and disasters don't change.
    fn eq(&self, other: &GameSetting) -> bool {
        self.num_safe == other.num_safe
            && self.num_shop == other.num_shop
            && self.num_disasters == other.num_disasters
    }
}

impl Eq for GameSetting {}

impl Hash for GameSetting {
    // Let's hope that rooms and disasters don't change.
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.num_safe.hash(state);
        self.num_shop.hash(state);
        self.num_disasters.hash(state);
    }
}

impl GameState {
    pub fn new(players: &Vec<PlayerInfo>, setting: GameSetting) -> GameState {
        let mut players_map = HashMap::new();
        for player in players {
            players_map.insert(player.secret.clone(), player);
        }
        let players = players_map;
        let mut thrones = Vec::new();
        let mut rng = rand::thread_rng();
        let mut deck: Vec<Box<dyn Room>> = setting
            .rooms
            .clone()
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
    pub fn to_schrodinger(&self) -> SchrodingerGameState {
        let mut new_turn_order = Vec::new();
        let mut new_castles = HashMap::new();
        let mut possible_rooms = self.setting.rooms.clone();
        for room in self.discard.iter() {
            possible_rooms.remove(room);
        }
        for (index, secret) in self.turn_order.iter().enumerate() {
            new_turn_order.push(index.to_string());
            let castle = self.castles.get(secret).unwrap().clone();
            for room in castle.get_rooms().values() {
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
            rng: self.rng.clone(),
            setting: self.setting.clone(),
        }
    }
    pub fn get_castles(&self) -> Vec<Castle> {
        self.turn_order
            .iter()
            .map(|secret| self.castles.get(secret).unwrap().clone())
            .collect()
    }
    pub fn possible_actions(&self, player_secret: &str) -> Vec<Action> {
        if let Some(castle) = self.castles.get(player_secret) {
            if castle.get_damage() != 0 || self.is_turn_player(player_secret) {
                return castle.possible_actions(&self.shop);
            }
        }
        return Vec::new();
    }
    pub fn action(&self, player_secret: &str, action: Action) -> Result<GameState> {
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
    pub fn view_shop(&self) -> &Vec<Box<dyn Room>> {
        &self.shop
    }
    pub fn view_discard(&self) -> &Vec<Box<dyn Room>> {
        &self.discard
    }
    pub fn view_previous_disasters(&self) -> &Vec<Box<dyn Disaster>> {
        &self.previous_disasters
    }
    pub fn view_queued_disasters(&self) -> &Vec<Box<dyn Disaster>> {
        &self.queued_disasters
    }
    pub fn get_setting(&self) -> &GameSetting {
        &self.setting
    }
}
