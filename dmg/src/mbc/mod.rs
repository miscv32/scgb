use crate::gb::GameBoy;

#[derive(Default, Debug, PartialEq)]
pub(crate) enum CartridgeType {
    #[default]
    NoMBC,
    MBC1,
    MBC3, // NO RTC
    MBC5,
}
#[derive(Default, Debug)]
pub struct MBC {
    pub(crate) cartridge_type: CartridgeType,
    pub(crate) rom_size: usize,
    pub(crate) ram_size: usize,
    has_battery: bool,
    pub(crate) has_ram: bool,
    pub(crate) ram_enabled: bool,
    pub(crate) rom_bank_number: u8,
    pub(crate) ram_bank_number: u8,
    pub(crate) banking_mode_1_select: bool,
    pub rom_bank_high: u8,
    pub rom_bank_low: u8,
}
impl GameBoy {
    pub fn detect_mbc(&mut self) -> MBC {
        let cartridge_type;
        let rom_size: usize;
        let ram_size: usize;
        let has_battery: bool;
        let has_ram: bool;
        match self.memory.cartridge[0x147] {
            0 => {
                cartridge_type = CartridgeType::NoMBC;
                has_battery = false;
                has_ram = false;
            }
            1 => {
                cartridge_type = CartridgeType::MBC1;
                has_battery = false;
                has_ram = false;
            }
            2 => {
                cartridge_type = CartridgeType::MBC1;
                has_battery = false;
                has_ram = true;
            }
            3 => {
                cartridge_type = CartridgeType::MBC1;
                has_battery = true;
                has_ram = true;
            }
            0x11 => {
                cartridge_type = CartridgeType::MBC3;
                has_battery = false;
                has_ram = false;
            }
            0x12 => {
                cartridge_type = CartridgeType::MBC3;
                has_battery = false;
                has_ram = true;
            }
            0x13 => {
                // MBC3 + RAM + Battery
                cartridge_type = CartridgeType::MBC3;
                has_battery = true;
                has_ram = true;
            }
            0x1A => {
                cartridge_type = CartridgeType::MBC5;
                has_battery = false;
                has_ram = true;
            }
            0x1B => {
                cartridge_type = CartridgeType::MBC5;
                has_battery = true;
                has_ram = true;
            }
            _ => {
                self.logger.log_info(&format!("{:#x}", self.memory.cartridge[0x147]));
                unimplemented!()
            }
        }
        match self.memory.cartridge[0x148] {
            0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 | 0x08 => {
                rom_size = 32 * 1024 * (1 << self.memory.cartridge[0x148] as usize);
            }
            0x52 | 0x53 | 0x54 => unimplemented!(),
            _ => panic!("Invalid ROM size (cartridge byte 0x148)"),
        }
        match self.memory.cartridge[0x149] {
            0x00 => {
                ram_size = 0;
            }
            0x01 => unimplemented!(), // PD homebrew ROMs according to Pan Docs
            0x02 => {
                ram_size = 8 * 1024;
            }
            0x03 => {
                ram_size = 32 * 1024;
            }
            0x04 => {
                ram_size = 128 * 1024;
            }
            0x05 => {
                ram_size = 64 * 1024;
            }
            _ => panic!("Invalid RAM size (cartridge byte 0x149)")
        }
        self.memory.switchable_ram = vec![0; self.mbc.ram_size];

        MBC {
            cartridge_type,
            rom_size,
            ram_size,
            has_battery,
            has_ram,
            ram_enabled: false,
            rom_bank_number: 1,
            ram_bank_number: 0,
            banking_mode_1_select: false,
            rom_bank_high: 0,
            rom_bank_low: 0,
        }
    }

    pub fn mbc_rom_bank_0(&self) -> &[u8] {
        match self.mbc.cartridge_type {
            CartridgeType::NoMBC | CartridgeType::MBC1 | CartridgeType::MBC3 => &self.memory.cartridge[0x0..=0x3FFF],
            _ => unimplemented!(),
        }
    }
    pub fn mbc_switchable_rom(&self) -> &[u8] {
        match self.mbc.cartridge_type {
            CartridgeType::NoMBC => &self.memory.cartridge[0x4000..=0x7FFF],
            CartridgeType::MBC1 | CartridgeType::MBC3 | CartridgeType::MBC5 => {
                let off = self.mbc.rom_bank_number as usize * 16*1024;
                &self.memory.cartridge[off .. off + 16*1024]
            },
        }
    }

    pub fn mbc_switchable_ram(&mut self) -> &mut [u8] {
        match self.mbc.has_ram {
            false => panic!("mbc_switchable_ram can only be called if MBC has RAM associated with it."),
            true  => &mut self.memory.switchable_ram[self.mbc.ram_bank_number as usize*8*1024..(self.mbc.ram_bank_number as usize+1)*8*1024],
        }
    }
}