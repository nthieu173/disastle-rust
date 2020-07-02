use crate::game::GameState;

use serde::{Deserialize, Serialize, Serializer};
use std::cmp;

pub trait Disaster: DisasterExt {
    fn name(&self) -> &str;
    fn damage(&self, num_prev_disasters: u32, links: (u32, u32, u32, u32)) -> u32;
    fn effect(&self, game_state: &GameState) -> GameState;
}

impl PartialEq for dyn Disaster {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for dyn Disaster {}

impl Clone for Box<dyn Disaster> {
    fn clone(&self) -> Box<dyn Disaster> {
        self.clone_box()
    }
}

impl Serialize for Box<dyn Disaster> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

pub trait DisasterExt {
    fn clone_box(&self) -> Box<dyn Disaster>;
}

impl<T> DisasterExt for T
where
    T: 'static + Disaster + Clone,
{
    fn clone_box(&self) -> Box<dyn Disaster> {
        Box::new(self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NormalDisaster {
    pub disaster_name: String,
    pub diamond_damage: (u32, u32),
    pub cross_damage: (u32, u32),
    pub moon_damage: (u32, u32),
}

impl Disaster for NormalDisaster {
    fn name(&self) -> &str {
        &self.disaster_name
    }
    fn damage(
        &self,
        num_prev_disasters: u32,
        (diamond_link, cross_link, moon_link, any_link): (u32, u32, u32, u32),
    ) -> u32 {
        let (diamond_base, diamond_mult) = self.diamond_damage;
        let (cross_base, cross_mult) = self.cross_damage;
        let (moon_base, moon_mult) = self.moon_damage;
        cmp::max(
            0,
            cmp::max(
                0,
                diamond_base + num_prev_disasters * diamond_mult - diamond_link,
            ) + cmp::max(0, cross_base + num_prev_disasters * cross_mult - cross_link)
                + cmp::max(0, moon_base + num_prev_disasters * moon_mult - moon_link)
                - any_link,
        )
    }
    fn effect(&self, game_state: &GameState) -> GameState {
        game_state.clone()
    }
}
