use crate::{gb::GameBoy};

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
            true => self.r.f |= 0x80,
            false => self.r.f &= 0x7F,
        }
    }

    pub fn set_flag_n(&mut self, value: bool) {
        match value {
            true => self.r.f |= 0x40,
            false => self.r.f &= 0xBF,
        }
    }

    pub fn set_flag_h(&mut self, value: bool) {
        match value {
            true => self.r.f |= 0x20,
            false => self.r.f &= 0xDF,
        }
    }

    pub fn set_flag_c(&mut self, value: bool) {
        match value {
            true => self.r.f |= 0x10,
            false => self.r.f &= 0xEF,
        }
    }

    pub fn get_flag_c(&self) -> u8 {
        match self.r.f & 0x10 == 0 {
            true => 0,
            false => 1,
        }
    }

    pub fn get_flag_z(&self) -> u8 {
        match self.r.f & 0x80 == 0 {
            true => 0,
            false => 1,
        }
    }

    pub fn get_flag_n(&self) -> u8 {
        match self.r.f & 0x40 == 0 {
            true => 0,
            false => 1,
        }
    }

    pub fn get_flag_h(&self) -> u8 {
        match self.r.f & 0x20 == 0 {
            true => 0,
            false => 1,
        }
    }
    pub fn get_hl(&self) -> u16 {
        unsigned_16(self.r.h, self.r.l)
    }

    pub fn get_af(&self) -> u16 {
        unsigned_16(self.r.a, self.r.f)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.r.h = ((value & 0xFF00) >> 8) as u8;
        self.r.l = value as u8;
    }

    pub fn get_bc(&self) -> u16 {
        unsigned_16(self.r.b, self.r.c)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.r.b = ((value & 0xFF00) >> 8) as u8;
        self.r.c = value as u8;
    }

    pub fn get_de(&self) -> u16 {
        unsigned_16(self.r.d, self.r.e)
    }

    pub fn set_de(&mut self, value: u16) {
        self.r.d = ((value & 0xFF00) >> 8) as u8;
        self.r.e = value as u8;
    }

    pub fn set_af(&mut self, value: u16) {
        self.r.a = ((value & 0xFF00) >> 8) as u8;
        self.r.f = value as u8;
    }

    pub fn set_r8(&mut self, r8: u8, data: u8) {
        match r8 {
            0 => self.r.b = data,
            1 => self.r.c = data,
            2 => self.r.d = data,
            3 => self.r.e = data,
            4 => self.r.h = data,
            5 => self.r.l = data,
            6 => self.write(self.get_hl(), data),
            7 => self.r.a = data,
            _ => (),
        }
    }

    pub fn get_r8(&self, value: u8) -> u8 {
        match value & 0b111 {
            0 => self.r.b,
            1 => self.r.c,
            2 => self.r.d,
            3 => self.r.e,
            4 => self.r.h,
            5 => self.r.l,
            6 => self.read(self.get_hl()),
            7 => self.r.a,
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
            3 => self.r.sp,
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
            3 => self.r.sp = value,
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
    
    pub fn map_background_palette(&self, data: u8) -> u8 {
        (self.r.bg_pal >> (data << 1)) & 0b11
    }

    pub fn map_sprite_palette(&self, palette: u8, colour: u8) -> u8 {
        (self.read(0xFF48 + palette as u16) >> (colour << 1)) & 0b11
    }

    pub fn oam(&self, offset: u8) -> u8 {
        self.read(self.oam_base + offset as u16)
    }
}
