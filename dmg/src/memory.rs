use crate::gb::GameBoy;
use crate::mbc;
use std::cmp::{max, min, PartialEq};

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
        match address {
            0xFF00 => {
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
                (self.r.joypad & 0xF0) | lower_nibble
            }
            0xFF04 => (self.r.div_16 >> 8) as u8,
            0xFF05 => self.r.tima,
            0xFF06 => self.r.tma,
            0xFF07 => self.r.tac,
            0xFF40 => self.r.lcdc,
            0xFF41 => self.r.stat,
            0xFF42 => self.r.scy,
            0xFF43 => self.r.scx,
            0xFF44 => self.r.ly,
            0xFF45 => self.r.lyc,
            0xFF47 => self.r.bg_pal,
            0xFF4A => self.r.wy,
            0xFF4B => self.r.wx,
            0xFF50 => self.r.bank,
            0xFF0F => self.r.r#if,
            0xFFFF => self.r.ie,
            _ => match self.memory.mapping_type {
                MappingType::Flat => self.memory.main[address as usize],
                MappingType::Default => {
                    if self.r.bank == 0 && (address as usize) < GB_ROM_SIZE {
                        // boot ROM if mapped
                        self.memory.boot_rom[address as usize]
                    } else if (0x0000..=0x3FFF).contains(&address) {
                        // ROM bank 0
                        self.mbc_rom_bank_0()[address as usize]
                    } else if (0x4000..=0x7FFF).contains(&address) {
                        // Switchable ROM banks 01-NN
                        self.mbc_switchable_rom()[address as usize - 0x4000]
                    } else if (0x8000..=0x9FFF).contains(&address) {
                        // VRAM
                        self.memory.main[address as usize]
                    } else if (0xA000..=0xBFFF).contains(&address) {
                        // External RAM
                        if self.mbc.has_ram {
                            self.mbc_switchable_ram()[address as usize - 0xA000]
                        } else {
                            0xFF
                        }
                    } else if (0xC000..=0xDFFF).contains(&address) {
                        // Work RAM
                        self.memory.main[address as usize]
                    } else if (0xE000..=0xFDFF).contains(&address) {
                        // Echo RAM
                        self.memory.main[address as usize - 0x2000]
                    } else if (0xFE00..=0xFE9F).contains(&address) {
                        // OAM
                        self.memory.main[address as usize]
                    } else if (0xFEA0..=0xFEFF).contains(&address) {
                        self.logger
                            .log_error("Read from 0xFEA0-0xFEFF prohibited memory area");
                        0xFF
                    } else if (0xFF00..=0xFF7F).contains(&address) {
                        // IO registers
                        self.logger.log_error(&format!(
                            "Read from unimplemented IO register at address {:#x}",
                            address
                        ));
                        0xFF
                    } else if (0xFF80..=0xFFFE).contains(&address) {
                        // HRAM
                        self.memory.main[address as usize]
                    } else {
                        0xFF
                    }
                }
            },
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0xFF05 => {
                self.timer.wait_reload = 0;
                self.r.tima = data;
            }
            0xFF06 => self.r.tma = data,
            0xFF07 => self.r.tac = data,
            0xFF04 => self.r.div_16 = 0,
            0xFF44 => {
                self.r.ly = data;
                self.check_and_trigger_ly_coincidence();
            }
            0xFF40 => self.r.lcdc = data,
            0xFF4A => self.r.wy = data,
            0xFF4B => self.r.wx = data,
            0xFF42 => self.r.scy = data,
            0xFF43 => self.r.scx = data,
            0xFF47 => self.r.bg_pal = data,
            0xFFFF => self.r.ie = data,
            0xFF0F => self.r.r#if = data,
            0xFF45 => {
                self.r.lyc = data;
                self.check_and_trigger_ly_coincidence();
            }
            0xFF41 => self.r.stat = data,
            0xFF00 => self.r.joypad = data & 0xF0,
            0xFF46 => {
                self.dma_base = ((data as u16) << 8) as usize;
                for i in 0..160 {
                    self.memory.main[0xFE00 + i] = self.read((self.dma_base + i) as u16)
                }
            }
            0xFF50 => self.r.bank = data,
            _ => match self.memory.mapping_type {
                MappingType::Flat => {}
                MappingType::Default => match self.mbc.cartridge_type {
                    mbc::CartridgeType::NoMBC => {}
                    mbc::CartridgeType::MBC1 => {
                        if address <= 0x1FFF {
                            self.mbc.ram_enabled = (data & 0x0F) == 0x0A;
                        } else if (0x2000..=0x3FFF).contains(&address) {
                            self.mbc.rom_bank_number = max(data & 0x1F, 1);
                            let num_banks = self.mbc.rom_size / (16 * 1024);
                            if self.mbc.rom_bank_number as usize > num_banks {
                                self.mbc.rom_bank_number &= (min(num_banks, 256) - 1) as u8;
                            }
                            if self.mbc.banking_mode_1_select {
                                self.mbc.rom_bank_number |= self.mbc.ram_bank_number << 5;
                            }
                        } else if (0x4000..=0x5FFF).contains(&address) {
                            self.mbc.ram_bank_number = data & 0x03;
                        } else if (0x6000..=0x7FFF).contains(&address) {
                            self.mbc.banking_mode_1_select = data == 1;
                        }

                        if self.mbc.has_ram && (0xA000..=0xBFFF).contains(&address) {
                            self.mbc_switchable_ram()[address as usize - 0xA000] = data;
                        }
                    }
                },
            },
        }
        // Not all of self.memory.main is mapped to CPU accessible addresses, so this is safe.
        self.memory.main[address as usize] = data;
    }
}
