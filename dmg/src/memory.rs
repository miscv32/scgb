use crate::gb::GameBoy;

pub const GB_RAM_SIZE: usize = 0x10000;
pub const GB_ROM_SIZE: usize = 0x100;
pub fn init() -> FlatRAM {
    [0; GB_RAM_SIZE]
}

pub type FlatRAM = [u8; GB_RAM_SIZE];

#[derive(PartialEq)]
pub enum MappingType {
    Flat,    // all addresses readable and writable (for SST)
    Default, // normal DMG behaviour with no MBCs (wip)
}
pub struct MappedRAM {
    pub mapping_type: MappingType,
    pub main: [u8; GB_RAM_SIZE],
    pub rom: [u8; GB_ROM_SIZE],
}
impl GameBoy {
    pub fn read(&self, address: u16) -> u8 {
        if (address as usize) >= GB_RAM_SIZE {
            0
        } else {
            if self.memory.mapping_type == MappingType::Default {
                if self.memory.main[0xFF50] == 0 && (address as usize) < GB_ROM_SIZE {
                    return self.memory.rom[address as usize];
                } else if address == 0xFF05 {
                    return self.r.tima;
                } else if address == 0xFF06 {
                    return self.r.tma;
                } else if address == 0xFF07 {
                    return self.r.tac;
                } else if address == 0xFF04 {
                    return self.r.div;
                } else if address == 0xFF44 {
                    return self.r.ly;
                } else if address == 0xFF40 {
                    return self.r.lcdc;
                } else if address == 0xFF4A {
                    return self.r.wy;
                } else if address == 0xFF4B {
                    return self.r.wx;
                } else if address == 0xFF42 {
                    return self.r.scy;
                } else if address == 0xFF43 {
                    return self.r.scx;
                } else if address == 0xFF47 {
                    return self.r.bg_pal;
                } else if address == 0xFFFF {
                    return self.r.ie;
                } else if address == 0xFF0F {
                    return self.r.r#if;
                } else if address == 0xFF45 {
                    return self.r.lyc;
                } else if address == 0xFF41 {
                    return self.r.stat;
                } else if address == 0xFF00 {
                    return self.r.joypad;
                }
            }
            return self.memory.main[address as usize];
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        if (address as usize) >= GB_RAM_SIZE {
            ()
        } else {
            if self.memory.mapping_type == MappingType::Default {
                if (self.memory.main[0xFF50] != 0)
                    && ((address <= 0x7FFF)
                        || (address >= 0xE000 && address <= 0xFDFF)
                        || (address >= 0xFEA0 && address <= 0xFEFF))
                {
                    return;
                } else if address == 0xFF05 {
                    self.r.tima = data
                } else if address == 0xFF06 {
                    self.r.tma = data;
                } else if address == 0xFF07 {
                    self.r.tac = data;
                } else if address == 0xFF04 {
                    self.r.div = 0;
                } else if address == 0xFF44 {
                    self.r.ly = data;
                } else if address == 0xFF40 {
                    self.r.lcdc = data;
                } else if address == 0xFF4A {
                    self.r.wy = data;
                } else if address == 0xFF4B {
                    self.r.wx = data;
                } else if address == 0xFF42 {
                    self.r.scy = data;
                } else if address == 0xFF43 {
                    self.r.scx = data;
                } else if address == 0xFF47 {
                    self.r.bg_pal = data;
                } else if address == 0xFFFF {
                    self.r.ie = data;
                } else if address == 0xFF0F {
                    self.r.r#if = data;
                } else if address == 0xFF45 {
                    self.r.lyc = data;
                } else if address == 0xFF41 {
                    self.r.stat = data;
                } else if address == 0xFF00 {
                    self.r.joypad = data & 0xF0;
                }
            }
            self.memory.main[address as usize] = data
        }
    }
}
