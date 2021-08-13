use crate::disaster::Disaster;
use disastle_castle_rust::Room;

#[derive(Clone)]
pub enum Card {
    Room(Box<dyn Room>),
    Disaster(Box<dyn Disaster>),
}
