use crate::{castle::Castle, disaster::Disaster, game::Card};

use serde::Serialize;

type Pos = (i32, i32);

#[derive(Clone, Serialize)]
pub enum PlayerState {
    Admin {
        name: String,
    },
    Lobby {
        name: String,
    },
    Wait {
        name: String,
        castle: Castle,
    },
    Disaster {
        name: String,
        castle: Castle,
        num_previous_disasters: u32,
        disasters: Vec<Box<dyn Disaster>>,
        remove_queue: Vec<Pos>,
        damage: u32,
    },
    Action {
        name: String,
        castle: Castle,
        limbo: Vec<Card>,
    },
    End {
        name: String,
        treasures: u32,
        rooms: u32,
        links: u32,
    },
    Dead {
        name: String,
    },
}

impl PlayerState {
    pub fn get_name(&self) -> &str {
        match self {
            PlayerState::Admin { name } => name,
            PlayerState::Lobby { name } => name,
            PlayerState::Wait { name, .. } => name,
            PlayerState::Disaster { name, .. } => name,
            PlayerState::Action { name, .. } => name,
            PlayerState::End { name, .. } => name,
            PlayerState::Dead { name } => name,
        }
    }
}
