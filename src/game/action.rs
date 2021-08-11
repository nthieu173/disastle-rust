use disastle_castle_rust::Pos;

pub enum Action {
    Place(usize, Pos),
    Move(Pos, Pos),
    Swap(Pos, Pos),
    Discard(Pos)
}