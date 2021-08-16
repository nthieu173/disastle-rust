use crate::disaster::Disaster;

#[derive(Clone)]
pub struct SimpleDisaster {
    name: String,
    diamond: DamageCalculation,
    cross: DamageCalculation,
    moon: DamageCalculation,
}

impl SimpleDisaster {
    pub fn new(
        name: String,
        diamond_m: u8,
        diamond_a: u8,
        cross_m: u8,
        cross_a: u8,
        moon_m: u8,
        moon_a: u8,
    ) -> SimpleDisaster {
        SimpleDisaster {
            name,
            diamond: DamageCalculation {
                multiplier: diamond_m,
                addition: diamond_a,
            },
            cross: DamageCalculation {
                multiplier: cross_m,
                addition: cross_a,
            },
            moon: DamageCalculation {
                multiplier: moon_m,
                addition: moon_a,
            },
        }
    }
}

impl Disaster for SimpleDisaster {
    fn get_name(&self) -> &str {
        &self.name
    }
    fn diamond_multiplier(&self) -> u8 {
        self.diamond.multiplier
    }
    fn diamond_addition(&self) -> u8 {
        self.diamond.addition
    }
    fn cross_multiplier(&self) -> u8 {
        self.cross.multiplier
    }
    fn cross_addition(&self) -> u8 {
        self.cross.addition
    }
    fn moon_multiplier(&self) -> u8 {
        self.moon.multiplier
    }
    fn moon_addition(&self) -> u8 {
        self.moon.addition
    }
}

impl SimpleDisaster {
    pub fn from_disaster(disaster: &dyn Disaster) -> SimpleDisaster {
        let d_add = disaster.diamond_damage(0);
        let d_mult = disaster.diamond_damage(1) - d_add;
        let c_add = disaster.cross_damage(0);
        let c_mult = disaster.cross_damage(1) - c_add;
        let m_add = disaster.moon_damage(0);
        let m_mult = disaster.moon_damage(1) - m_add;
        SimpleDisaster {
            name: disaster.get_name().to_string(),
            diamond: DamageCalculation {
                multiplier: d_mult,
                addition: d_add,
            },
            cross: DamageCalculation {
                multiplier: c_mult,
                addition: c_add,
            },
            moon: DamageCalculation {
                multiplier: m_mult,
                addition: m_add,
            },
        }
    }
}

#[derive(Clone)]
struct DamageCalculation {
    multiplier: u8,
    addition: u8,
}
