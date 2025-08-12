use crate::gb::GameBoy;

#[derive(Default)]
enum CartridgeType {
    #[default]
    NoMBC,
    MBC1,
}
#[derive(Default)]
pub(crate) struct MBC {
    cartridge_type: CartridgeType,
    rom_size: usize,
    ram_size: usize,
    has_battery: bool,
    pub(crate) has_ram: bool,
}
impl GameBoy {
    pub(crate) fn detect_mbc(&self) -> MBC {
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
            _ => unimplemented!(),
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

        MBC {
            cartridge_type,
            rom_size,
            ram_size,
            has_battery,
            has_ram,
        }
    }

    pub fn mbc_rom_bank_0(&self) -> &[u8] {
        match self.mbc.cartridge_type {
            CartridgeType::NoMBC => &self.memory.cartridge[0x0..=0x3FFF],
            _ => unimplemented!(),
        }
    }
    pub fn mbc_switchable_rom(&self) -> &[u8] {
        match self.mbc.cartridge_type {
            CartridgeType::NoMBC => &self.memory.cartridge[0x4000..=0x7FFF],
            _ => unimplemented!(),
        }
    }

    pub fn mbc_switchable_ram(&mut self) -> &mut [u8] {
        match self.mbc.has_ram {
            false => panic!("mbc_switchable_ram can only be called if MBC has RAM associated with it."),
            true  => unimplemented!(),
        }
    }
}