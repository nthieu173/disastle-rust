use nanoid::nanoid;

pub struct PlayerInfo {
    pub name: String,
    pub secret: String,
}

impl PlayerInfo {
    pub fn new(name: &str) -> PlayerInfo {
        PlayerInfo {
            name: name.to_owned(),
            secret: nanoid!(),
        }
    }
}