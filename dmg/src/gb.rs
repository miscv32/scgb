// refactoring TODOs
// restructure gb struct to be less disorganised

use crate::isr::{self, Isr};
use crate::log;
use crate::memory::{self, MappedRAM, MappingType};
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
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub div: u8,
    pub ly: u8,
    pub lcdc: u8,
    pub wy: u8,
    pub wx: u8,
    pub scy: u8,
    pub scx: u8,
    pub bg_pal: u8,
    pub ie: u8,
    pub r#if: u8,
    pub lyc: u8,
    pub stat: u8,
    pub joypad: u8,
}

pub struct GameBoy {
    pub clock: u128, // m-cycles
    pub running: bool,
    pub registers: Registers,
    pub cycles_to_idle: Option<u8>,
    pub memory: memory::MappedRAM,
    pub ime: bool,
    pub ime_dispatch: Option<u8>,
    pub displaybuf_0: [u8; 160 * 144],
    pub displaybuf_1: [u8; 160 * 144], // TODO remove copy by swapping buffers.
    pub backbuf_id: u8,
    pub logger: log::Logger,
    pub isr: Isr,
    pub(crate) window_line_counter: u8,
    pub test_mode: bool,
    keys_ssba: u8,
    keys_dulr: u8,
    tima_period: u16,
    tma_old: u8,
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
        tima: 0,
        tma: 0,
        tac: 0,
        div: 0,
        ly: 0,
        lcdc: 0,
        wy: 0,
        wx: 0,
        scy: 0,
        scx: 0,
        bg_pal: 0,
        ie: 0,
        r#if: 0,
        lyc: 0,
        stat: 0,
        joypad: 0,
    };

    let memory: MappedRAM = MappedRAM {
        main: [0u8; memory::GB_RAM_SIZE],
        rom: [0; memory::GB_ROM_SIZE],
        mapping_type: MappingType::Default,
    };

    let logger = log::Logger {
        level: log::LogLevel::Info,
    };

    let isr = Isr {
        state: isr::State::None,
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
        displaybuf_0: [0; 160 * 144],
        displaybuf_1: [0; 160 * 144],
        backbuf_id: 0,
        logger: logger,
        isr: isr,
        window_line_counter: 0,
        test_mode: false,
        keys_ssba: 0xF,
        keys_dulr: 0xF,
        tima_period: 256, 
        tma_old: 0,
    }
}

impl GameBoy {
    pub fn tick(&mut self) {
        // This should be called once every M-cycle.
        // Current behaviour is M-cycle faking, i.e. all work is done in first M-cycle
        // CPU & RAM idle for the rest of the instruction's M-cycles
        if self.running {
            self.update_ime(false);
            self.tma_old = self.registers.tma;

            if self.test_mode == false {
                self.trigger_interrupts();

                if self.isr.state != isr::State::None {
                    self.handle_interrupt();
                    return;
                } else if self.ime && ((self.registers.ie & self.registers.r#if) != 0) {
                    self.isr.state = isr::State::ReadIF;
                }
            }

            if let Some(cycles_to_idle) = self.cycles_to_idle {
                if cycles_to_idle == 0 {
                    let opcode: u8 = self.read(self.registers.pc);
                    self.registers.pc += 1;
                    self.cycles_to_idle = self.fetch_decode_execute(opcode);
                } else {
                    self.cycles_to_idle = Some(self.cycles_to_idle.unwrap() - 1);
                }
            }

            self.update_ime(true);

            if self.test_mode == false {
                self.renderer();

                let buttons = (self.registers.joypad >> 5) == 0;
                let dpad = (self.registers.joypad >> 4) == 0;
                let lower_nibble: u8;
                if !buttons && !dpad {
                    lower_nibble = 0xF;
                } else if buttons && dpad {
                    lower_nibble = self.keys_ssba | self.keys_dulr;
                } else if buttons {
                    lower_nibble = self.keys_ssba;
                } else {
                    lower_nibble = self.keys_dulr;
                }
                let old_jp = self.registers.joypad;
                self.registers.joypad = (old_jp & 0xF0) | lower_nibble;
                if self.registers.joypad & 0x0F != old_jp & 0x0F {
                    self.registers.r#if |= 1 << 4;
                }

                self.update_timers();
            };

            self.clock += 1;
        }
    }

    pub fn update_timers(&mut self) {
        if self.clock % 64 == 0 {
            self.registers.div += 1;
        }
        if (self.registers.tac >> 2) & 1 == 1 {
            self.tima_period = match self.registers.tac & 0x3 {
                0 => 256,
                1 => 4,
                2 => 16,
                3 => 64,
                _ => panic!("Should be mathematically impossible"),
            }
        }
        if (self.clock % self.tima_period as u128) == 0 {
            if (self.registers.tima as u16 + 1) > 0xFF {
                self.registers.tima = self.tma_old;
                self.registers.r#if |= 0x4;
            } else {
                self.registers.tima += 1;
            }
        }
    }

    pub fn press_key(&mut self, mut key_id: u8) {
        // 3: start, 2: select 1: b, 0: a, 7: down, 6: up, 5: left, 4: right
        let mut keys: &mut u8 = &mut self.keys_ssba;
        if key_id > 3 {
            keys = &mut self.keys_dulr;
            key_id -= 4;
        };
        *keys &= !1 << key_id;
    }

    pub fn unpress_key(&mut self, mut key_id: u8) {
        let mut keys: &mut u8 = &mut self.keys_ssba;
        if key_id > 3 {
            keys = &mut self.keys_dulr;
            key_id -= 4;
        };
        *keys |= 1 << key_id;
    }
}
