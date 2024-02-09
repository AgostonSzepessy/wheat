use std::fs::File;
use std::io::Read;
use std::path::Path;

use chip8::traits::Rom;

pub struct RomDriver {
    pub rom: Vec<u8>,
}

impl RomDriver {
    pub fn new(filename: &str) -> Self {
        let path = Path::new(filename);

        let mut file = File::open(path).expect("File does not exist");
        let mut rom = Vec::new();
        file.read(&mut rom).unwrap();

        Self { rom }
    }
}

impl Rom for RomDriver {
    fn data(&self) -> &Vec<u8> {
        &self.rom
    }
}
