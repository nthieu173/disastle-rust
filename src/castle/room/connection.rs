use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub enum Connection {
    None,
    Any,
    Diamond(bool),
    Cross(bool),
    Moon(bool),
}

#[derive(Clone, Copy)]
pub enum Link {
    Any,
    Diamond,
    Cross,
    Moon,
}

impl Connection {
    pub fn connect(&self, other: &Connection) -> bool {
        matches!((self, &other), (Connection::None, Connection::None))
            || (!matches!(self, Connection::None) && !matches!(other, Connection::None))
    }
    pub fn link(&self, other: &Connection) -> Option<Link> {
        match (self, other) {
            (Connection::Any, Connection::Any) => Some(Link::Any),
            (Connection::Any, Connection::Diamond(_)) => Some(Link::Diamond),
            (Connection::Any, Connection::Cross(_)) => Some(Link::Cross),
            (Connection::Any, Connection::Moon(_)) => Some(Link::Moon),
            (Connection::Diamond(_), Connection::Any) => Some(Link::Diamond),
            (Connection::Cross(_), Connection::Any) => Some(Link::Cross),
            (Connection::Moon(_), Connection::Any) => Some(Link::Moon),
            (Connection::Diamond(_), Connection::Diamond(_)) => Some(Link::Diamond),
            (Connection::Cross(_), Connection::Cross(_)) => Some(Link::Cross),
            (Connection::Moon(_), Connection::Moon(_)) => Some(Link::Moon),
            (_, _) => None,
        }
    }
    pub fn powered(&self, other: &Connection) -> Option<bool> {
        match self {
            Connection::None => None,
            Connection::Any => None,
            Connection::Diamond(gold) => match gold {
                false => None,
                true => Some(matches!(other, Connection::Diamond(_))),
            },
            Connection::Cross(gold) => match gold {
                false => None,
                true => Some(matches!(other, Connection::Cross(_))),
            },
            Connection::Moon(gold) => match gold {
                false => None,
                true => Some(matches!(other, Connection::Moon(_))),
            },
        }
    }
}
