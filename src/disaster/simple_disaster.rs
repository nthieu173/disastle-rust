use crate::disaster::Disaster;

pub struct SimpleDisaster {
    diamond: DamageCalculation,
    cross: DamageCalculation,
    moon: DamageCalculation,
}

impl SimpleDisaster {
    pub fn new(diamond_m: u8, diamond_a: u8,
            cross_m: u8, cross_a: u8,
            moon_m: u8, moon_a: u8) -> SimpleDisaster {
        SimpleDisaster {
            diamond: DamageCalculation { multiplier: diamond_m, addition: diamond_a },
            cross: DamageCalculation { multiplier: cross_m, addition: cross_a },
            moon: DamageCalculation { multiplier: moon_m, addition: moon_a },
        }
    }
}


impl Disaster for SimpleDisaster {
    fn diamond_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.diamond.multiplier + self.diamond.addition
    }
    fn cross_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.cross.multiplier + self.cross.addition
    }
    fn moon_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.moon.multiplier + self.moon.addition
    }
}

struct DamageCalculation {
    multiplier: u8,
    addition: u8,
}
