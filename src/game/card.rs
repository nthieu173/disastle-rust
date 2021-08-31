use crate::disaster::Disaster;
use disastle_castle_rust::Room;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Card {
    Room(Room),
    Disaster(Disaster),
}
