pub mod simple_disaster;

use std::cmp::min;

pub trait Disaster {
    fn diamond_damage(&self, num_previous_disasters: u8) -> u8;
    fn cross_damage(&self, num_previous_disasters: u8) -> u8;
    fn moon_damage(&self, num_previous_disasters: u8) -> u8;
    fn damage(&self, num_previous_disasters: u8, diamond_link: u8, cross_link: u8, moon_link: u8, any_link: u8) -> u8 {
        min(0, min(0, self.diamond_damage(num_previous_disasters) - diamond_link)
        + min(0, self.cross_damage(num_previous_disasters) - cross_link)
        + min(0, self.moon_damage(num_previous_disasters) - moon_link)
        - any_link)
    }
}