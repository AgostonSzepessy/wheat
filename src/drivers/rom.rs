use std::fs::{self};

use wheat::traits::Rom;

pub struct RomDriver {
    pub rom: Vec<u8>,
}

impl RomDriver {
    pub fn new(filename: &str) -> Self {
        let rom = fs::read(filename).unwrap();

        Self { rom }
    }
}

impl Rom for RomDriver {
    fn data(&self) -> &Vec<u8> {
        &self.rom
    }
}
