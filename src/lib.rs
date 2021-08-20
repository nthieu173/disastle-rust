pub mod disaster;
pub mod game;

use disaster::SimpleDisaster;
use disastle_castle_rust::SimpleRoom;
pub use ron;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    result,
};

pub fn load_disasters(path: &Path) -> result::Result<Vec<SimpleDisaster>, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    println!("{}", content);
    match ron::from_str(&content) {
        Ok(disasters) => Ok(disasters),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

pub fn load_rooms(path: &Path) -> result::Result<Vec<SimpleRoom>, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    match ron::from_str(&content) {
        Ok(rooms) => Ok(rooms),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::{load_disasters, load_rooms};
    use std::path::Path;
    #[test]
    fn test_deserialize_disasters() {
        let path = Path::new("disasters.ron");
        let result = load_disasters(&path);
        assert!(result.is_ok());
        let disasters = result.unwrap();
        assert_eq!(disasters.len(), 12);
    }
    #[test]
    fn test_deserialize_thrones() {
        let path = Path::new("thrones.ron");
        let result = load_rooms(&path);
        assert!(result.is_ok());
        let disasters = result.unwrap();
        assert_eq!(disasters.len(), 10);
    }
    #[test]
    fn test_deserialize_rooms() {
        let path = Path::new("rooms.ron");
        let result = load_rooms(&path);
        assert!(result.is_ok());
        let disasters = result.unwrap();
        assert_eq!(disasters.len(), 100);
    }
}
