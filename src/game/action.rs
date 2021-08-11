use disastle_castle_rust::Pos;

enum Action {
    Place(index: usize),
    Move(Pos, Pos),
    Swap(Pos, Pos),
    Discard()
}