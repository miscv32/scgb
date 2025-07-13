pub const GB_RAM_SIZE: usize = 0x10000;
pub const GB_ROM_SIZE: usize = 0x100;
pub fn init() -> FlatRAM {
    [0; GB_RAM_SIZE]
}

pub type FlatRAM = [u8; GB_RAM_SIZE];

#[derive(PartialEq)]
pub enum MappingType {
    Flat, // all addresses readable and writable
    Default, // normal DMG behaviour with no MBCs (wip)
}
pub struct MappedRAM {
    pub mapping_type: MappingType,
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
            if self.mapping_type == MappingType::Default {
                if self.main[0xFF50] == 0 && (address as usize) < GB_ROM_SIZE{
                    return self.rom[address as usize];
                }
            }
            return self.main[address as usize];
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if (address as usize) >= GB_RAM_SIZE {
            ()
        } else {
            // check we are not trying to write to a cartridge or otherwise illegal area
            if self.mapping_type == MappingType::Default {
                if (self.main[0xFF50] != 0) && ((address <= 0x7FFF) || (address >= 0xE000 && address <= 0xFDFF) || (address >= 0xFEA0 && address <= 0xFEFF)) {
                    return;
                };
                // ignore writes to joypad reg lower nibble.
                if address == 0xFF00 {
                    self.main[address as usize] = data & 0xF0;
                };
                // reset DIV on write attempt
                if address == 0xFF04 {
                    self.main[address as usize] = 0;
                }
                // NB, in real hardware FF07 write may increase TIMA, but we arent emulating this for now
            }
            self.main[address as usize] = data
        }
    }
}
