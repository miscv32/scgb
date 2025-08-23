use std::cmp::{max, min, PartialEq};
use crate::gb::GameBoy;
use crate::mbc;

pub const GB_RAM_SIZE: usize = 0x10000;
pub const GB_ROM_SIZE: usize = 0x100;
pub fn init() -> FlatRAM {
    [0; GB_RAM_SIZE]
}

pub type FlatRAM = [u8; GB_RAM_SIZE];

#[derive(PartialEq, Debug)]
pub enum MappingType {
    Flat,    // all addresses readable and writable (for SST)
    Default, // normal DMG behaviour with no MBCs (wip)
}
pub struct MappedRAM {
    pub mapping_type: MappingType,
    pub main: [u8; GB_RAM_SIZE], // VRAM, work RAM, etc.
    pub boot_rom: [u8; GB_ROM_SIZE],
    pub cartridge: Vec<u8>,
}

impl GameBoy {
    pub fn read(&mut self, address: u16) -> u8 {
        if (address as usize) >= GB_RAM_SIZE {
            0
        } else {
            if self.memory.mapping_type == MappingType::Default {
                // BOOT ROM
                if self.r.bank == 0 && (address as usize) < GB_ROM_SIZE {
                    return self.memory.boot_rom[address as usize]
                }
                // ROM bank 00
                else if address <= 0x3FFF {
                    return self.mbc_rom_bank_0()[address as usize];
                }
                // Switchable ROM bank
                else if address >= 0x4000 && address <= 0x7FFF {
                    return self.mbc_switchable_rom()[address as usize - 0x4000];
                }
                // Switchable external RAM bank
                else if address >= 0xA000 && address <= 0xBFFF {
                    return if self.mbc.has_ram {
                        self.mbc_switchable_ram()[address as usize - 0xA000]
                    } else {
                        0xFF
                    }
                }
                // IO Registers
                else if address == 0xFF05 {
                    return self.r.tima;
                } else if address == 0xFF06 {
                    return self.r.tma;
                } else if address == 0xFF07 {
                    return self.r.tac;
                } else if address == 0xFF04 {
                    return (self.r.div_16 >> 8) as u8;
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
                } else if address == 0xFF02 {
                    return 0xFF;
                } else if address == 0xFF00 {
                    let not_select_buttons = if (self.r.joypad >> 5) & 1 == 0 {
                        0
                    } else {
                        0xFF
                    };
                    let not_select_dpad = if (self.r.joypad >> 4) & 1 == 0 {
                        0
                    } else {
                        0xFF
                    };
                    let lower_nibble = ((not_select_buttons & self.keys_dulr)
                        | (not_select_dpad & self.keys_ssba))
                        & 0x0F;
                    return (self.r.joypad & 0xF0) | lower_nibble;
                } else if address == 0xFF50 {
                    return self.r.bank
                }
            }
            self.memory.main[address as usize]
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        if (address as usize) >= GB_RAM_SIZE {
            ()
        } else {
            if self.memory.mapping_type == MappingType::Default {
                if self.mbc.cartridge_type == mbc::CartridgeType::MBC1 && self.r.bank != 0 {
                    if address <= 0x1FFF {
                        if data & 0x0F == 0x0A {
                            self.mbc.ram_enabled = true;
                        } else {
                            self.mbc.ram_enabled = false;
                        }
                    }
                    else if address >= 0x2000 && address <= 0x3FFF {
                        self.mbc.rom_bank_number = max(data & 0x1F, 1);
                        let num_banks = self.mbc.rom_size / 16 * 1024;
                        if self.mbc.rom_bank_number as usize > num_banks {
                            // mask to number of bits needed to select num_banks
                            // assuming num_banks is a power of 2
                            self.mbc.rom_bank_number &= (min(num_banks, 256) - 1) as u8;
                        }
                        if self.mbc.banking_mode_1_select {
                            self.mbc.rom_bank_number |= self.mbc.ram_bank_number << 5;
                        }
                    }
                    else if address >= 0x4000 && address <= 0x5FFF {
                        self.mbc.ram_bank_number = data & 0x03;
                    }
                    else if address >= 0x6000 && address <= 0x7FFF {
                        self.mbc.banking_mode_1_select = data == 1;
                    }
                }

                else if (address <= 0x7FFF)
                        || (!self.mbc.has_ram && (address >= 0xA000 && address <= 0xBFFF))
                        || (address >= 0xE000 && address <= 0xFDFF)
                        || (address >= 0xFEA0 && address <= 0xFEFF)
                {
                    return;
                }
                if self.mbc.has_ram && (address >= 0xA000 && address <= 0xBFFF) {
                    self.mbc_switchable_ram()[address as usize - 0xA000] = data;
                }
                else if address == 0xFF05 {
                    self.timer.wait_reload = 0;
                    self.r.tima = data
                } else if address == 0xFF06 {
                    self.r.tma = data;
                } else if address == 0xFF07 {
                    self.r.tac = data;
                } else if address == 0xFF04 {
                    self.r.div_16 = 0;
                } else if address == 0xFF44 {
                    self.r.ly = data;
                    self.check_and_trigger_ly_coincidence();
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
                    println!("Write to IF (manual interrupt trigger) :P {:#x}", data);
                    self.r.r#if = data;
                } else if address == 0xFF45 {
                    self.r.lyc = data;
                    self.check_and_trigger_ly_coincidence();
                } else if address == 0xFF41 {
                    self.r.stat = data;
                } else if address == 0xFF00 {
                    self.r.joypad = data & 0xF0;
                } else if address == 0xFF46 {
                    // start doing DMA transfer
                    self.dma_base = ((data as u16) << 8) as usize;
                    for i in 0..160 {
                        self.memory.main[0xFE00 + i] = self.read((self.dma_base + i) as u16)
                    }
                } else if address == 0xFF50 {
                    self.r.bank = data;
                }
            }
            self.memory.main[address as usize] = data
        }
    }
}
