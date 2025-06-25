const GB_RAM_SIZE: usize = 0x10000;

pub fn init() -> FlatRAM {
    [0; GB_RAM_SIZE]
}

pub type FlatRAM = [u8; GB_RAM_SIZE];

pub trait Memory {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

impl Memory for FlatRAM {
    fn read(&self, address: u16) -> u8 {
        if (address as usize) >= GB_RAM_SIZE {
            0
        } else {
            self[address as usize]
        }
    }

    fn write(&mut self, address: u16, data: u8) {
        if (address as usize) >= GB_RAM_SIZE {
            ()
        } else {
            if address == 0xFF44 {
                println!("wrote {} to 0xFF44", data)
            }
            self[address as usize] = data
        }
    }
}
