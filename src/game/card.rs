use crate::disaster::Disaster;
use disastle_castle_rust::Room;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Card {
    Room(Room),
    Disaster(Disaster),
}
