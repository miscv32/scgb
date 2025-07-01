pub const GB_RAM_SIZE: usize = 0x10000;
pub const GB_ROM_SIZE: usize = 0x100;
pub fn init() -> FlatRAM {
    [0; GB_RAM_SIZE]
}

pub type FlatRAM = [u8; GB_RAM_SIZE];
pub struct MappedRAM {
    pub main: [u8; GB_RAM_SIZE],
    pub rom: [u8; GB_ROM_SIZE],
}
pub trait Memory {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

impl Memory for MappedRAM {
    fn read(&self, address: u16) -> u8 {
        if (address as usize) >= GB_RAM_SIZE {
            0
        } else {
            if self.main[0xFF50] == 0 && (address as usize) < GB_ROM_SIZE{
                return self.rom[address as usize];
            }
            if address == 0xFF00 {
                return 0xFF // TODO implement input
            }
            return self.main[address as usize];
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if (address as usize) >= GB_RAM_SIZE {
            ()
        } else {
            self.main[address as usize] = data
        }
    }
}
