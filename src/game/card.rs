use disastle_castle_rust::Room;
use crate::disaster::Disaster;

pub enum Card {
    Room(Box<dyn Room>),
    Disaster(Box<dyn Disaster>),
}