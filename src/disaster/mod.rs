pub mod simple_disaster;

pub trait Disaster: DisasterClone {
    fn diamond_damage(&self, num_previous_disasters: u8) -> u8;
    fn cross_damage(&self, num_previous_disasters: u8) -> u8;
    fn moon_damage(&self, num_previous_disasters: u8) -> u8;
}

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
