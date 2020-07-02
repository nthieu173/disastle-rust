pub mod connection;
use connection::{Connection, Link};

use serde::{Deserialize, Serialize};

type Pos = (i32, i32);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Room {
    id: i32,
    pub name: String,
    pub up: Connection,
    pub right: Connection,
    pub down: Connection,
    pub left: Connection,
}

impl Room {
    pub fn throne_room(id: i32, name: String) -> Self {
        Self {
            id,
            name,
            up: Connection::Any,
            right: Connection::Any,
            down: Connection::Any,
            left: Connection::Any,
        }
    }
    pub fn rotate_right(mut self) {
        let new = Room {
            up: self.left,
            right: self.up,
            down: self.right,
            left: self.down,
            ..self
        };
        self.up = new.up;
        self.right = new.right;
        self.down = new.down;
        self.left = new.left;
    }
    pub fn rotate_left(mut self) {
        let new = Room {
            up: self.right,
            right: self.down,
            down: self.left,
            left: self.up,
            ..self
        };
        self.up = new.up;
        self.right = new.right;
        self.down = new.down;
        self.left = new.left;
    }
    pub fn connecting(&self, (x, y): Pos) -> Vec<Pos> {
        let mut positions = Vec::new();
        if !matches!(self.up, Connection::None) {
            positions.push((x, y + 1))
        }
        if !matches!(self.right, Connection::None) {
            positions.push((x + 1, y))
        }
        if !matches!(self.down, Connection::None) {
            positions.push((x, y - 1))
        }
        if !matches!(self.left, Connection::None) {
            positions.push((x - 1, y))
        }
        positions
    }
    pub fn surrounding((x, y): Pos) -> [Pos; 8] {
        [
            (x, y + 1),
            (x + 1, y + 1),
            (x + 1, y),
            (x + 1, y - 1),
            (x, y - 1),
            (x - 1, y - 1),
            (x - 1, y),
            (x - 1, y + 1),
        ]
    }
    pub fn up_link(&self, room: &Room) -> Option<Link> {
        room.up.link(&room.down)
    }
    pub fn right_link(&self, room: &Room) -> Option<Link> {
        room.right.link(&room.left)
    }
    pub fn down_link(&self, room: &Room) -> Option<Link> {
        room.down.link(&room.up)
    }
    pub fn left_link(&self, room: &Room) -> Option<Link> {
        room.left.link(&room.right)
    }
    pub fn up_powered(&self, room: &Room) -> Option<bool> {
        room.up.powered(&room.down)
    }
    pub fn right_powered(&self, room: &Room) -> Option<bool> {
        room.right.powered(&room.left)
    }
    pub fn down_powered(&self, room: &Room) -> Option<bool> {
        room.down.powered(&room.up)
    }
    pub fn left_powered(&self, room: &Room) -> Option<bool> {
        room.left.powered(&room.right)
    }
}
