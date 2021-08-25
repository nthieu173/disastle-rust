mod simple_disaster;
pub use simple_disaster::SimpleDisaster;

use std::{
    fmt,
    hash::{Hash, Hasher},
};

pub trait Disaster: DisasterClone {
    fn get_name(&self) -> &str;
    fn diamond_multiplier(&self) -> u8;
    fn diamond_addition(&self) -> u8;
    fn cross_multiplier(&self) -> u8;
    fn cross_addition(&self) -> u8;
    fn moon_multiplier(&self) -> u8;
    fn moon_addition(&self) -> u8;
    fn diamond_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.diamond_multiplier() + self.diamond_addition()
    }
    fn cross_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.cross_multiplier() + self.cross_addition()
    }
    fn moon_damage(&self, num_previous_disasters: u8) -> u8 {
        num_previous_disasters * self.moon_multiplier() + self.moon_addition()
    }
}

impl fmt::Debug for dyn Disaster {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Disaster")
            .field("name", &self.get_name())
            .field(
                "diamond",
                &format!(
                    "x{}+{}",
                    &self.diamond_multiplier(),
                    &self.diamond_addition()
                ),
            )
            .field(
                "cross",
                &format!("x{}+{}", &self.cross_multiplier(), &self.cross_addition()),
            )
            .field(
                "moon",
                &format!("x{}+{}", &self.moon_multiplier(), &self.moon_addition()),
            )
            .finish()
    }
}

impl Hash for dyn Disaster {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_name().hash(state);
        self.diamond_multiplier().hash(state);
        self.diamond_addition().hash(state);
        self.cross_multiplier().hash(state);
        self.cross_addition().hash(state);
        self.moon_multiplier().hash(state);
        self.moon_addition().hash(state);
    }
}

impl PartialEq for dyn Disaster {
    fn eq(&self, other: &dyn Disaster) -> bool {
        self.get_name() == other.get_name()
            && self.diamond_multiplier() == other.diamond_multiplier()
            && self.diamond_addition() == other.diamond_addition()
            && self.cross_multiplier() == other.cross_multiplier()
            && self.cross_addition() == other.cross_addition()
            && self.moon_multiplier() == other.moon_multiplier()
            && self.moon_addition() == other.moon_addition()
    }
}

impl Eq for dyn Disaster {}

pub trait DisasterClone {
    fn clone_box(&self) -> Box<dyn Disaster>;
}

impl<T> DisasterClone for T
where
    T: 'static + Disaster + Clone,
{
    fn clone_box(&self) -> Box<dyn Disaster> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Disaster> {
    fn clone(&self) -> Box<dyn Disaster> {
        self.clone_box()
    }
}
