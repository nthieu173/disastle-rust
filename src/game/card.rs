use crate::disaster::Disaster;
use disastle_castle_rust::Room;

#[derive(Clone)]
pub enum Card {
    Room(Room),
    Disaster(Disaster),
}
