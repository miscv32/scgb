use crate::memory::{self, MappedRAM};
use crate::memory::Memory;
use crate::log;
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


#[derive(PartialEq)]
#[derive(Debug)]
pub enum IsrState {
    ReadIF,
    ReadIE,
    Push1,
    Push2,
    Jump,
    None,
}

pub struct Isr {
    state: IsrState,
    iflag: u8,
    ienable: u8,
    ir_addr: u16,
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
    pub logger: log::Logger,
    pub isr: Isr,
    window_line_counter: u8,
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

    let logger = log::Logger {
        level: log::LogLevel::Error,
    };

    let isr = Isr {
        state: IsrState::None,
        iflag: 0,
        ienable: 0,
        ir_addr: 0,
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
        logger: logger,
        isr: isr,
        window_line_counter: 0,
    }
}

impl GameBoy {
    pub fn tick(&mut self) {
        // This should be called once every M-cycle.
        // Current behaviour is M-cycle faking, i.e. all work is done in first M-cycle
        // CPU & RAM idle for the rest of the instruction's M-cycles
        if self.running {
            
            self.update_ime(false);
            
            self.trigger_interrupts();

            if self.isr.state != IsrState::None {
                self.handle_interrupt();
                return;
            } else if self.ime && ((self.get_ie() & self.get_if()) != 0) {
                self.isr.state = IsrState::ReadIF;    
            }

            if let Some(cycles_to_idle) = self.cycles_to_idle {
                if cycles_to_idle == 0 {
                    let opcode: u8 = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    self.cycles_to_idle = self.fetch_decode_execute(opcode);
                } else {   
                    self.cycles_to_idle = Some(self.cycles_to_idle.unwrap() - 1);
                }
            }

            self.update_ime(true);

            

            self.renderer();
            
            self.clock += 1;
        }
    }

    fn trigger_interrupts(&mut self) {
        if self.get_ly() == self.get_lyc() {
            self.set_stat(self.get_stat() | 0b100);
            // TODO implement other interrupts than LY == LYC
        }
        if  ((self.get_stat() >> 6) & (self.get_stat() >> 2) & 1) != 0 {
            self.logger.log_info("LY=LYC interrupt triggered");
            self.set_if(self.get_if() | 0b10);
        }
        if (self.get_stat() & 0b11) == 0b01 {
            self.logger.log_info("VBlank interrupt triggered");
            self.set_if(self.get_if() | 0b01);
            if ((self.get_stat() >> 4) & 1) != 0 {
                self.set_if(self.get_if() | 0b10);
            }
            return;
        }
        
    }

    fn handle_interrupt(&mut self) {
        match self.isr.state {
            IsrState::ReadIF => {
                self.logger.log_info("ISR: ReadIF");
                self.isr.iflag = self.get_if();
                self.isr.state = IsrState::ReadIE;
            }
            IsrState::ReadIE => {
                self.logger.log_info("ISR: ReadIE");
                self.isr.ienable = self.get_ie();
                // get highest priority interrupt
                for i in 0..5 {
                    if (((self.isr.iflag >> i) & 1) & ((self.isr.ienable >> i) & 1)) != 0 {
                        self.set_if(self.get_if() & !(1 << i)); // clear interrupt
                        self.isr.ir_addr = [0x40, 0x48, 0x50, 0x58, 0x60][i];
                        break;
                    }
                }
                // if no interrupt requested then set IsrState::None
                if self.isr.ir_addr == 0 {
                    self.isr.state = IsrState::None
                }
                // else set IsrState::Push1, clear interrupt and set ime to false
                self.isr.state = IsrState::Push1;
                self.ime = false;
            }
            IsrState::Push1 => {
                self.logger.log_info("ISR: Push1");
                self.registers.sp -= 1;
                self.memory.write(self.registers.sp, ((self.registers.pc & 0xff00) >> 8) as u8);
                self.isr.state = IsrState::Push2;
            }
            IsrState::Push2 => {
                self.logger.log_info("ISR: Push2");
                self.registers.sp -= 1;
                self.memory.write(self.registers.sp, (self.registers.pc & 0xff) as u8);
                self.isr.state = IsrState::Jump;
            }
            IsrState::Jump => {
                self.logger.log_info("ISR: Jump");
                self.registers.pc = self.isr.ir_addr;
                self.logger.log_info(&format!("ISR: Jumping to: {:#x}", self.isr.ir_addr));
                self.isr.state = IsrState::None;
                self.isr.ir_addr = 0;
            }
            IsrState::None => {
                self.logger.log_error("handle_interrupts called when isr.state == IsrState::None");
            }

        }
    }
    fn update_ime(&mut self, after_fde: bool)
    {
        if let Some(id) = self.ime_dispatch {
            if id > 0 {
                self.ime_dispatch = Some(id - 1);
            } else {
                match after_fde {
                    false =>  self.logger.log_error("attempt to set IME at wrong part of cycle"),
                    true => self.ime = true
                }
                self.ime_dispatch = None;
            }
        }
    }
    
    fn renderer(&mut self) {
        if self.clock % 114 == 0 {
            if self.get_ly() == 144 { // VBlank entered
                self.window_line_counter = 0;
                self.set_stat(self.get_stat() & 0b11111101);
                self.set_if(self.get_if() | 1);
                self.logger.log_info("Renderer: Entered VBlank");
            } else {
                self.set_stat(self.get_stat() & 0b11111110);
            
            }
        }
        else if self.clock % 114 == 20 {
            if self.get_ly() < 144 { //Drawing
                self.render_scanline();
                self.set_stat(self.get_stat() & 0b11111111);
            
            }
        } else if self.clock % 114 == 63 {
            // HBlank
            self.set_stat(self.get_stat() & 0b11111100);
            self.set_ly(self.get_ly() + 1);
        } else if self.clock % 114 == 113 {
            if self.get_ly() == 154 { // VBlank exited
                self.display = self.display_temp;
                self.set_ly(0);
            }
        }
    }
    
    fn render_background(&mut self) {
        if self.get_lcdc() & 1 == 0 {
            return
        }
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
        let tilemap: u16 = match self.get_lcdc() & 8 {
            0 => 0x9800,
            _ => 0x9c00,
        };
        for x in 0..160 {
            let x_plus_scroll =( x as u16 + self.get_scx() as u16 ) % 256;
            let y_plus_scroll = self.window_line_counter as u16;
            let tile_x = x_plus_scroll % 8;
            let tile_y = y_plus_scroll % 8;

            let tile_num = self.memory.read(tilemap + x_plus_scroll / 8 + 32 * (y_plus_scroll as u16 / 8));

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
            self.window_line_counter += 1;
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
        self.memory.read(0xFFFF)
    }

    fn get_if(&self) -> u8 {
        self.memory.read(0xFF0F)
    }
    fn set_if(&mut self, data: u8) {
        self.memory.write(0xFF0F, data);
    }
    
    fn get_lyc(&self) -> u8 {
        self.memory.read(0xFF45)
    }
    
    fn set_stat(&mut self, data: u8) {
        self.memory.write(0xFF41, data)
    }
    
    fn get_stat(&self) -> u8 {
        self.memory.read(0xFF41)
    }

}