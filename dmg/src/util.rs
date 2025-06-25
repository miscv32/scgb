use crate::{gb::GameBoy, memory::Memory};

pub fn msb(two_bytes: u16) -> u8 {
    (two_bytes >> 8) as u8
}

pub fn lsb(two_bytes: u16) -> u8 {
    (two_bytes & 0xFF) as u8
}

pub fn unsigned_16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

impl GameBoy {
    pub fn set_flag_z(&mut self, value: bool) {
        match value {
            true => self.registers.f |= 0x80,
            false => self.registers.f &= 0x7F,
        }
    }

    pub fn set_flag_n(&mut self, value: bool) {
        match value {
            true => self.registers.f |= 0x40,
            false => self.registers.f &= 0xBF,
        }
    }

    pub fn set_flag_h(&mut self, value: bool) {
        match value {
            true => self.registers.f |= 0x20,
            false => self.registers.f &= 0xDF,
        }
    }

    pub fn set_flag_c(&mut self, value: bool) {
        match value {
            true => self.registers.f |= 0x10,
            false => self.registers.f &= 0xEF,
        }
    }

    pub fn get_flag_c(&self) -> u8 {
        match self.registers.f & 0x10 == 0 {
            true => 0,
            false => 1,
        }
    }

    pub fn get_flag_z(&self) -> u8 {
        match self.registers.f & 0x80 == 0 {
            true => 0,
            false => 1,
        }
    }

    pub fn get_flag_n(&self) -> u8 {
        match self.registers.f & 0x40 == 0 {
            true => 0,
            false => 1,
        }
    }

    pub fn get_flag_h(&self) -> u8 {
        match self.registers.f & 0x20 == 0 {
            true => 0,
            false => 1,
        }
    }
    pub fn get_hl(&self) -> u16 {
        unsigned_16(self.registers.h, self.registers.l)
    }

    pub fn get_af(&self) -> u16 {
        unsigned_16(self.registers.a, self.registers.f)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.registers.h = ((value & 0xFF00) >> 8) as u8;
        self.registers.l = value as u8;
    }

    pub fn get_bc(&self) -> u16 {
        unsigned_16(self.registers.b, self.registers.c)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.registers.b = ((value & 0xFF00) >> 8) as u8;
        self.registers.c = value as u8;
    }

    pub fn get_de(&self) -> u16 {
        unsigned_16(self.registers.d, self.registers.e)
    }

    pub fn set_de(&mut self, value: u16) {
        self.registers.d = ((value & 0xFF00) >> 8) as u8;
        self.registers.e = value as u8;
    }

    pub fn set_af(&mut self, value: u16) {
        self.registers.a = ((value & 0xFF00) >> 8) as u8;
        self.registers.f = value as u8;
    }

    pub fn set_r8(&mut self, r8: u8, data: u8) {
        match r8 {
            0 => self.registers.b = data,
            1 => self.registers.c = data,
            2 => self.registers.d = data,
            3 => self.registers.e = data,
            4 => self.registers.h = data,
            5 => self.registers.l = data,
            6 => self.memory.write(self.get_hl(), data),
            7 => self.registers.a = data,
            _ => (),
        }
    }

    pub fn get_r8(&self, value: u8) -> u8 {
        match value & 0b111 {
            0 => self.registers.b,
            1 => self.registers.c,
            2 => self.registers.d,
            3 => self.registers.e,
            4 => self.registers.h,
            5 => self.registers.l,
            6 => self.memory.read(self.get_hl()),
            7 => self.registers.a,
            _ => panic!("get_r8"),
        }
    }
    pub fn get_r16_group_2(&mut self, r16: u8) -> u16 {
        match r16 {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => {
                let hl = self.get_hl();
                self.set_hl(hl + 1);
                hl
            }
            3 => {
                let hl = self.get_hl();
                self.set_hl(hl - 1);
                hl
            }
            _ => panic!("get_r16_group_2 recieved illegal value"),
        }
    }

    pub fn get_r16_group_1(&mut self, r16: u8) -> u16 {
        match r16 {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.registers.sp,
            _ => panic!("get_r16_group_1 recieved illegal value"),
        }
    }

    pub fn get_r16_group_3(&mut self, r16: u8) -> u16 {
        match r16 {
            0 => self.get_bc(),
            1 => self.get_de(),
            2 => self.get_hl(),
            3 => self.get_af(),
            _ => panic!("get_r16_group_1 recieved illegal value"),
        }
    }

    pub fn set_r16_group_1(&mut self, r16: u8, value: u16) {
        match r16 {
            0 => self.set_bc(value),
            1 => self.set_de(value),
            2 => self.set_hl(value),
            3 => self.registers.sp = value,
            _ => panic!("get_r16_group_2 recieved illegal value"),
        }
    }

    pub fn set_r16_group_3(&mut self, r16: u8, value: u16) {
        match r16 {
            0 => self.set_bc(value),
            1 => self.set_de(value),
            2 => self.set_hl(value),
            3 => self.set_af(value),
            _ => panic!("get_r16_group_2 recieved illegal value"),
        }
    }
}
