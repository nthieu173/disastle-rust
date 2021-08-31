use serde::{Deserialize, Serialize};
use std::{fmt, hash::Hash};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Disaster {
    pub name: String,
    pub diamond: DamageCalculation,
    pub cross: DamageCalculation,
    pub moon: DamageCalculation,
}

impl Disaster {
    pub fn diamond_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.diamond.multiplier + self.diamond.addition
    }
    pub fn cross_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.cross.multiplier + self.cross.addition
    }
    pub fn moon_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.moon.multiplier + self.moon.addition
    }
}

impl fmt::Display for Disaster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Disaster")
            .field("name", &self.name)
            .field(
                "diamond",
                &format!("x{}+{}", &self.diamond.multiplier, &self.diamond.addition),
            )
            .field(
                "cross",
                &format!("x{}+{}", &self.cross.multiplier, &self.cross.addition),
            )
            .field(
                "moon",
                &format!("x{}+{}", &self.moon.multiplier, &self.moon.addition),
            )
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct DamageCalculation {
    pub multiplier: u8,
    pub addition: u8,
}
