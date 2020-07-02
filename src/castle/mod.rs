mod error;
pub mod room;

pub use error::CastleError;
use room::connection::Link;
use room::Room;

use petgraph::{
    algo,
    graph::{Graph, NodeIndex},
    Undirected,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    result,
};

type Result<T> = result::Result<T, CastleError>;
type Pos = (i32, i32);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Castle {
    rooms: HashMap<Pos, Room>,
    throne_rooms: Vec<Pos>,
    connections: Graph<Pos, u8, Undirected>,
}

impl Castle {
    pub fn new(throne_room: Room) -> Self {
        let mut rooms = HashMap::new();
        rooms.insert((0, 0), throne_room);
        Castle {
            rooms,
            throne_rooms: vec![(0, 0)],
            connections: Graph::new_undirected(),
        }
    }
    pub fn num_rooms(&self) -> usize {
        self.rooms.len()
    }
    pub fn free_pos(&self) -> HashSet<Pos> {
        let mut result = HashSet::new();
        for (pos, room) in self.rooms.iter() {
            room.connecting(*pos)
                .iter()
                .filter(|p| !self.rooms.contains_key(p))
                .for_each(|p| {
                    result.insert(*p);
                });
        }
        result
    }
    pub fn is_powered(&self, (x, y): Pos) -> Result<bool> {
        if let Some(room) = self.rooms.get(&(x, y)) {
            let mut result = true;
            if let Some(up_room) = self.rooms.get(&(x, y + 1)) {
                if let Some(up_result) = room.up_powered(up_room) {
                    result &= up_result;
                }
            }
            if let Some(right_room) = self.rooms.get(&(x + 1, y)) {
                if let Some(right_result) = room.up_powered(right_room) {
                    result &= right_result;
                }
            }
            if let Some(down_room) = self.rooms.get(&(x, y - 1)) {
                if let Some(down_result) = room.up_powered(down_room) {
                    result &= down_result;
                }
            }
            if let Some(left_room) = self.rooms.get(&(x - 1, y)) {
                if let Some(left_result) = room.up_powered(left_room) {
                    result &= left_result;
                }
            }
            Ok(result)
        } else {
            Err(CastleError::InvalidPos)
        }
    }
    pub fn links(&self) -> (u32, u32, u32, u32) {
        let mut any = 0;
        let mut diamond = 0;
        let mut cross = 0;
        let mut moon = 0;
        for ((x, y), room) in self.rooms.iter() {
            if let Some(up_room) = self.rooms.get(&(*x, y + 1)) {
                if let Some(up_result) = room.up_link(up_room) {
                    match up_result {
                        Link::Any => any += 1,
                        Link::Diamond => diamond += 1,
                        Link::Cross => cross += 1,
                        Link::Moon => moon += 1,
                    }
                }
            }
            if let Some(right_room) = self.rooms.get(&(x + 1, *y)) {
                if let Some(right_result) = room.right_link(right_room) {
                    match right_result {
                        Link::Any => any += 1,
                        Link::Diamond => diamond += 1,
                        Link::Cross => cross += 1,
                        Link::Moon => moon += 1,
                    }
                }
            }
            if let Some(down_room) = self.rooms.get(&(*x, y - 1)) {
                if let Some(down_result) = room.down_link(down_room) {
                    match down_result {
                        Link::Any => any += 1,
                        Link::Diamond => diamond += 1,
                        Link::Cross => cross += 1,
                        Link::Moon => moon += 1,
                    }
                }
            }
            if let Some(left_room) = self.rooms.get(&(x - 1, *y)) {
                if let Some(left_result) = room.left_link(left_room) {
                    match left_result {
                        Link::Any => any += 1,
                        Link::Diamond => diamond += 1,
                        Link::Cross => cross += 1,
                        Link::Moon => moon += 1,
                    }
                }
            }
        }
        (any / 2, diamond / 2, cross / 2, moon / 2)
    }
}

impl Castle {
    pub fn move_outer(&mut self, pos_from: Pos, pos_to: Pos) -> Result<()> {
        if !self.move_outer_valid(&pos_from, &pos_to) {
            if let Some(room) = self.rooms.remove(&pos_from) {
                self.connections.remove_node(
                    self.find_node_pos(&pos_from)
                        .expect("connections should reflect rooms existance"),
                );
                match self.place(room, pos_to) {
                    Ok(()) => Ok(()),
                    Err(_) => unreachable!("move_valid should have tested valid place"),
                }
            } else {
                unreachable!("move_valid should have tested valid remove")
            }
        } else {
            Err(CastleError::InvalidMove)
        }
    }
    pub fn place(&mut self, room: Room, pos: Pos) -> Result<()> {
        if !self.rooms.contains_key(&pos) && self.place_valid(&room, &pos) {
            let connected_pos: Vec<Pos> = room
                .connecting(pos)
                .iter()
                .filter(|p| self.rooms.contains_key(p))
                .map(|p| *p)
                .collect();
            let connected_index: Vec<NodeIndex> = self
                .connections
                .node_indices()
                .filter_map(|i| match self.connections.node_weight(i) {
                    None => None,
                    Some(w) => {
                        if connected_pos.contains(w) {
                            Some(i)
                        } else {
                            None
                        }
                    }
                })
                .collect();
            let room_index = self.connections.add_node(pos);
            for c_index in connected_index {
                self.connections.add_edge(room_index, c_index, 1);
            }
            self.rooms.insert(pos, room);
            Ok(())
        } else {
            Err(CastleError::InvalidPlace)
        }
    }
    pub fn remove(&mut self, pos: Pos) -> Result<()> {
        if self.remove_valid(&pos) {
            if let Some(_) = self.rooms.remove(&pos) {
                self.connections.remove_node(self.find_node_pos(&pos)
                        .expect("connections should reflect rooms existance"));
                Ok(())
            } else {
                unreachable!("remove_valid should have checked room existence")
            }
        } else {
            Err(CastleError::InvalidRemove)
        }
    }
    pub fn swap(&mut self, pos_1: Pos, pos_2: Pos) -> Result<()> {
        if !self.swap_valid(&pos_1, &pos_2) {
            let room_1 = self.rooms.remove(&pos_1).unwrap();
            let room_2 = self.rooms.remove(&pos_2).unwrap();
            self.rooms.insert(pos_1, room_2);
            self.rooms.insert(pos_2, room_1);
            Ok(())
        } else {
            Err(CastleError::InvalidSwap)
        }
    }
    pub fn move_outer_valid(&self, pos_from: &Pos, pos_to: &Pos) -> bool {
        if let Some(room) = self.rooms.get(pos_from) {
            room.connecting(*pos_from)
                .iter()
                .filter(|p| self.rooms.contains_key(p))
                .count()
                == 1
                && self.remove_valid(pos_from)
                && !self.rooms.contains_key(pos_to)
                && self.place_valid(room, pos_from)
        } else {
            false
        }
    }
    pub fn place_valid(&self, room: &Room, (x, y): &Pos) -> bool {
        let up = self.rooms.get(&(*x, y + 1));
        let right = self.rooms.get(&(x + 1, *y));
        let down = self.rooms.get(&(*x, y - 1));
        let left = self.rooms.get(&(x - 1, *y));
        (up.is_none() || up.unwrap().down.connect(&room.up))
            && (right.is_none() || right.unwrap().left.connect(&room.right))
            && (down.is_none() || down.unwrap().up.connect(&room.down))
            && (left.is_none() || left.unwrap().right.connect(&room.left))
    }
    pub fn remove_valid(&self, pos: &Pos) -> bool {
        let mut test_connections = self.connections.clone();
        if let Some(remove_index) =
            test_connections
                .node_indices()
                .find_map(|i| match self.connections.node_weight(i) {
                    None => None,
                    Some(w) => {
                        if w == pos {
                            Some(i)
                        } else {
                            None
                        }
                    }
                })
        {
            test_connections.remove_node(remove_index);
            for n_index in test_connections.neighbors(remove_index) {
                let mut orphaned = true;
                for (t_x, t_y) in self.throne_rooms.iter() {
                    if let Some(throne_index) = test_connections.node_indices().find_map(|i| {
                        match test_connections.node_weight(i) {
                            None => None,
                            Some(w) => {
                                if w == &(*t_x, *t_y) {
                                    Some(i)
                                } else {
                                    None
                                }
                            }
                        }
                    }) {
                        if let Some(_) = algo::astar(
                            &test_connections,
                            throne_index,
                            |i| i == n_index,
                            |_| 1,
                            |i| {
                                if let Some((x, y)) = test_connections.node_weight(i) {
                                    (x - t_x).abs() + (y - t_y).abs()
                                } else {
                                    999
                                }
                            },
                        ) {
                            orphaned = false;
                            break;
                        }
                    }
                }
                if orphaned {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
    pub fn swap_valid(&self, pos_1: &Pos, pos_2: &Pos) -> bool {
        let room_1 = self.rooms.get(&pos_1);
        let room_2 = self.rooms.get(&pos_2);
        pos_1 != pos_2
            && room_1.is_some()
            && room_2.is_some()
            && self.place_valid(room_1.unwrap(), pos_2)
            && self.place_valid(room_2.unwrap(), pos_1)
    }
    fn find_node_pos(&self, pos: &Pos) -> Option<NodeIndex> {
        self.connections
            .node_indices()
            .find_map(|i| match self.connections.node_weight(i) {
                None => None,
                Some(w) => {
                    if w == pos {
                        Some(i)
                    } else {
                        None
                    }
                }
            })
    }
}
