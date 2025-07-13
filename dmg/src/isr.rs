use crate::gb::GameBoy;

#[derive(PartialEq, Debug)]
pub enum State {
    ReadIF,
    ReadIE,
    Push1,
    Push2,
    Jump,
    None,
}

pub struct Isr {
    pub state: State,
    pub iflag: u8,
    pub ienable: u8,
    pub ir_addr: u16,
}

impl GameBoy {
    pub fn trigger_interrupts(&mut self) {
        if self.registers.ly == self.registers.lyc {
            self.registers.stat |= 0b100;
        }
        if ((self.registers.stat >> 6) & (self.registers.stat >> 2) & 1) != 0 {
            self.logger.log_info("LY=LYC interrupt triggered");
            self.registers.r#if |= 0b10;
        }
        if (self.registers.stat & 0b11) == 0b01 {
            self.logger.log_info("VBlank interrupt triggered");
            self.registers.r#if |= 0b01;
            if ((self.registers.stat >> 4) & 1) != 0 {
                self.registers.r#if |= 0b10;
            }
            return;
        }
    }

    pub fn handle_interrupt(&mut self) {
        match self.isr.state {
            State::ReadIF => {
                self.isr.iflag = self.registers.r#if;
                self.isr.state = State::ReadIE;
            }
            State::ReadIE => {
                self.isr.ienable = self.registers.ie;
                // get highest priority interrupt
                for i in 0..5 {
                    if (((self.isr.iflag >> i) & 1) & ((self.isr.ienable >> i) & 1)) != 0 {
                        self.registers.r#if &= !(1 << i);
                        self.isr.ir_addr = [0x40, 0x48, 0x50, 0x58, 0x60][i];
                        break;
                    }
                }
                if self.isr.ir_addr == 0 {
                    self.isr.state = State::None
                }
                self.isr.state = State::Push1;
                self.ime = false;
            }
            State::Push1 => {
                self.registers.sp -= 1;
                self.write(self.registers.sp, ((self.registers.pc & 0xff00) >> 8) as u8);
                self.isr.state = State::Push2;
            }
            State::Push2 => {
                self.registers.sp -= 1;
                self.write(self.registers.sp, (self.registers.pc & 0xff) as u8);
                self.isr.state = State::Jump;
            }
            State::Jump => {
                self.registers.pc = self.isr.ir_addr;
                self.logger
                    .log_info(&format!("ISR: Jumping to: {:#x}", self.isr.ir_addr));
                self.isr.state = State::None;
                self.isr.ir_addr = 0;
            }
            State::None => {
                self.logger
                    .log_error("handle_interrupts called when isr.state == State::None");
            }
        }
    }
    pub fn update_ime(&mut self, after_fde: bool) {
        if let Some(id) = self.ime_dispatch {
            if id > 0 {
                self.ime_dispatch = Some(id - 1);
            } else {
                match after_fde {
                    false => self
                        .logger
                        .log_error("attempt to set IME at wrong part of cycle"),
                    true => self.ime = true,
                }
                self.ime_dispatch = None;
            }
        }
    }
}