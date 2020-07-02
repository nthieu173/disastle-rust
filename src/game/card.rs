mod crate::castle::room::Room;
mod crate::disaster::Disaster;

pub enum Card {
    Room(Room),
    Disaster(Disaster),
}