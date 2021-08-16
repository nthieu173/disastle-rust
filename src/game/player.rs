#[derive(Clone)]
pub struct PlayerInfo {
    pub name: String,
    pub secret: String,
}

impl PlayerInfo {
    pub fn new(name: &str, secret: &str) -> PlayerInfo {
        PlayerInfo {
            name: name.to_owned(),
            secret: secret.to_owned(),
        }
    }
}
