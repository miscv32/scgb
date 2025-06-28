use crate::memory::{self, MappedRAM};
use crate::memory::Memory;
use crate::{log, util};
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

pub enum PPUState {
    OAMScan,
    Drawing,
    HBlank,
    VBlank,
}

pub struct GameBoy {
    pub clock: u128, // measured in m-cycles, NOT T-cycles.
    pub running: bool,
    pub registers: Registers,
    pub cycles_to_idle: Option<u8>,
    pub memory: memory::MappedRAM,
    pub ime: bool,
    pub ime_dispatch: Option<u8>,
    pub display_temp: [u8; 160*144], // contents of frame as ppu draws before vblank
    pub display: [u8; 160*144], // after vblank
    pub log_level: log::LogLevel,
    isr: bool,
}

pub fn init() -> GameBoy {
    let registers: Registers = Registers {
        a: 0,
        f: 0,
        b: 0,
        c: 0,
        d: 0,
        e: 0,
        h: 0,
        l: 0,
        sp: 0,
        pc: 0,
    };

    let memory: MappedRAM = MappedRAM {
        main: [0u8; memory::GB_RAM_SIZE],
        rom: [0; memory::GB_ROM_SIZE],
    };

    GameBoy {
        clock: 0,
        running: true,
        registers: registers,
        cycles_to_idle: Some(0),
        memory: memory,
        ime: false,
        ime_dispatch: None,
        display_temp: [0; 160*144],
        display: [0; 160*144],
        log_level: log::LogLevel::Info,
        isr: false,
    }
}

impl GameBoy {
    pub fn tick(&mut self) {
        // This should be called once every M-cycle.
        // Current behaviour is M-cycle faking, i.e. all work is done in first M-cycle
        // CPU & RAM idle for the rest of the instruction's M-cycles
        if self.running {
            
            
            if let Some(id) = self.ime_dispatch {
                if id > 0 {
                    self.ime_dispatch = Some(id - 1);
                } else {
                    self.log_error("attempt to set IME at wrong part of cycle");
                    self.ime_dispatch = None;
                }
            }

            // check for interrupts
            if self.ime && (self.get_ie() & self.get_if() != 0) {
                if !self.isr {
                    self.cycles_to_idle = Some(5);
                    self.isr = true; 
                    self.log_info("Entered ISR");
                } else {
                    if let Some(cycles_to_idle) = self.cycles_to_idle {
                        if cycles_to_idle == 3 {
                            // TODO split up fde.rs into functions so we can call without copying code
                            self.log_info("ISR: Pushing PC to stack");
                            self.registers.sp -= 1;
                            self.memory.write(self.registers.sp, util::msb(self.registers.pc));
                            self.registers.sp -= 1;
                            self.memory.write(self.registers.sp, util::lsb(self.registers.pc));
                            println!("{:#x}", self.registers.sp);
                        }
                        if cycles_to_idle == 1 {
                            self.log_info("ISR: setting PC to IH");
                            let ih_index = (self.get_ie() & self.get_if()).trailing_zeros();
                            self.registers.pc = [0x40, 0x48, 0x50, 0x58, 0x60][ih_index as usize];
                            self.isr = false;
                            self.log_info("ISR: exiting");
                        }
                    }
                    self.cycles_to_idle = Some(self.cycles_to_idle.unwrap() - 1);
                    println!("{}", self.cycles_to_idle.unwrap());
                }
            } else if let Some(cycles_to_idle) = self.cycles_to_idle {
                if cycles_to_idle == 0 {
                    let opcode: u8 = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    self.cycles_to_idle = self.fetch_decode_execute(opcode);
                } else {   
                    self.cycles_to_idle = Some(self.cycles_to_idle.unwrap() - 1);
                }
            }

            if let Some(id) = self.ime_dispatch {
                if id > 0 {
                    self.ime_dispatch = Some(id - 1);
                } else {
                    self.ime = true;
                    self.ime_dispatch = None;
                }
            }
            
            // TODO implement interrupt system
            
            if self.clock % 114 == 0 {
                if self.get_ly() == 154 {
                    self.set_ly(0);
                }

                if self.get_ly() < 144 {
                    self.render_scanline();
                } else if self.get_ly() == 144 {
                    self.display = self.display_temp;
                }

                self.set_ly(self.get_ly() + 1);
                
            }
            self.clock += 1;
        }
    }

    fn render_background(&mut self) {
        let tilemap: u16 = match self.get_lcdc() & 8 {
            0 => 0x9800,
            _ => 0x9c00,
        };
        for x in 0..160 {
            let x_plus_scroll =( x as u16 + self.get_scx() as u16 ) % 256;
            let y_plus_scroll = (self.get_ly() as u16 + self.get_scy() as u16) % 256;
            let tile_x = x_plus_scroll % 8;
            let tile_y = y_plus_scroll % 8;

            let tile_num = self.memory.read(tilemap + x_plus_scroll / 8 + (y_plus_scroll/ 8) * 32);

            let mut tile_addr: u16;
            match self.get_lcdc() & 0x10 {
                0 => {
                    tile_addr = (0x9000 + (tile_num as i32 * 16)) as u16;
                }
                _ => {
                    tile_addr = 0x8000 + (tile_num as u16) * 16;
                }
            };

            tile_addr += 2 * tile_y;
            let data_low = (self.memory.read(tile_addr) >> (7 - tile_x)) & 1;
            let data_high = (self.memory.read(tile_addr + 1) >> (7 - tile_x)) & 1;
            
            self.display_temp[self.get_ly() as usize * 160 + x as usize] = self.map_background_palette(data_low | (data_high << 1));

        };

    }

    fn render_window(&mut self) {
        
    }
    fn _render_sprite_init(&mut self) {
        
    }
    fn _render_sprite(&mut self) {

    }

    fn render_scanline(&mut self) {
        
        if self.get_lcdc() & 1 != 0 {
            self.render_background();
        }
        if (self.get_lcdc()&0x20 != 0) && self.get_wx() < 166 && self.get_ly() >= self.get_wy() {
            self.render_window();
        }
    }
    fn get_ly(&self) -> u8 {
        self.memory.read(0xFF44)
    }

    fn set_ly(&mut self, data: u8) {
        self.memory.write(0xFF44, data)
    }

    fn get_lcdc(&self) -> u8 {
        self.memory.read(0xFF40)
    }

    fn get_wy(&self) -> u8 {
        self.memory.read(0xFF4A)
    }
    fn get_wx(&self) -> u8 {
        self.memory.read(0xFF4B)
    }
    fn get_scy(&self) -> u8 {
        self.memory.read(0xFF42)
    }
    fn get_scx(&self) -> u8 {
        self.memory.read(0xFF43)
    }
    fn get_background_palette(&self) -> u8 {
        self.memory.read(0xFF47)
    }

    fn map_background_palette(&self, data: u8) -> u8 {
        (self.get_background_palette() >> (data<<1)) & 0b11
    }

    fn get_ie(&self) -> u8 {
        self.memory.read(0xFFFF) & 0b11111
    }

    fn get_if(&self) -> u8 {
        self.memory.read(0xFF0F) & 0b11111
    }



}