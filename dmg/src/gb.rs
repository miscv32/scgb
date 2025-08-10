// TODOs
// add state transition functions to ensure that certain classes of bugs are avoided
// for instance cycles_to_idle must always be 0 if we want to restart in the Execute state.

use crate::memory::{self, MappedRAM, MappingType};
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
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
    pub div_16: u16,
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
#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    pub size_y: u8,
    pub x: i16,
    pub y: i16,
    pub x_flip: u8,
    pub y_flip: u8,
    pub pal: u8,
    pub priority: u8,
    pub tile_num: u8,
}

#[derive(PartialEq, Debug)]
pub enum State {
    Execute,
    Halted,
    Stopped, // yes these are distinct!
    InterruptHandler,
    DmaTransfer,
}

#[derive(PartialEq)]
pub enum IsrState {
    Wait1,
    Wait2,
    PCPush1,
    PCPush2,
    Jump,
}

pub enum InterruptType {
    VBlank = 0,
    LCD = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

pub struct Timer {
    prev_and_result: u8,
    pub(crate) wait_reload: i32,
}

pub struct GameBoy {
    pub clock: u128, // m-cycles
    pub r: Registers,
    pub cycles_to_idle: Option<u8>,
    pub memory: MappedRAM,
    pub ime: bool,
    pub ime_dispatch: Option<u8>,
    pub displaybuf_0: [u8; 160 * 144],
    pub displaybuf_1: [u8; 160 * 144],
    pub backbuf_id: u8,
    pub logger: log::Logger,
    pub isr_state: IsrState,
    pub test_mode: bool,
    pub(crate) keys_ssba: u8,
    pub(crate) keys_dulr: u8,
    pub(crate) oam_base: u16,
    pub sprites: [Sprite; 10],
    pub num_sprites: usize,
    pub background: [u8; 256 * 256],
    pub window: [u8; 256 * 256],
    pub state: State,
    pub timer: Timer,
    pub dma_transfer_bytes_copied: u8,
    pub(crate) dma_base: usize,
    pub(crate) window_line_counter: u8,
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
        div_16: 0,
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
        level: log::LogLevel::None,
    };

    GameBoy {
        clock: 0,
        state: State::Execute,
        r: registers,
        cycles_to_idle: Some(0),
        memory,
        ime: false,
        ime_dispatch: None,
        displaybuf_0: [0; 160 * 144],
        displaybuf_1: [0; 160 * 144],
        backbuf_id: 0,
        logger,
        isr_state: IsrState::Wait1,
        test_mode: false,
        keys_ssba: 0xF,
        keys_dulr: 0xF,
        oam_base: 0,
        sprites: [Sprite {
            size_y: 0,
            x: 0,
            y: 0,
            x_flip: 0,
            y_flip: 0,
            pal: 0,
            priority: 0,
            tile_num: 0,
        }; 10],
        num_sprites: 0,
        background: [0; 256 * 256],
        window: [0; 256 * 256],
        timer: Timer {
            prev_and_result: 0,
            wait_reload: 0,
        },
        dma_transfer_bytes_copied: 0,
        dma_base: 0,
        window_line_counter: 0,
    }
}

impl GameBoy {
    pub fn tick(&mut self) {
        self.update_ime(false);

        self.check_and_trigger_ly_coincidence();

        let wake_up = (self.r.r#if & self.r.ie) != 0;
        let interrupt_requested = self.ime && wake_up;

        if interrupt_requested {
            self.state = State::InterruptHandler;
        }

        if self.state == State::InterruptHandler {
            self.handle_interrupts();
            return;
        }

        for _ in 0..4 {
            self.update_timers();
        }

        if self.state == State::Halted && wake_up {
            self.state = State::Execute;
            self.cycles_to_idle = Some(0);
        }

        if self.state == State::Execute || self.state == State::DmaTransfer {
            self.execute();
        }

        // if self.state == State::DmaTransfer { // TODO allow CPU execution but stop reads/writes to  external memory during DMA transfer. I dont think many games rely on this
        //     if self.dma_transfer_bytes_copied < 160 {
        //         self.memory.main[self.oam_base as usize + self.dma_transfer_bytes_copied as usize] = self.memory.main[self.dma_base as usize + self.dma_transfer_bytes_copied as usize];
        //         self.dma_transfer_bytes_copied += 1;
        //     } else if self.dma_transfer_bytes_copied == 160 {
        //         self.state = State::Execute;
        //         println!("Done DMA transfer")
        //     } else {
        //         panic!("Maybe copied more than 160 bytes in dma transfer. This is a bug!!")
        //     }
        // }

        self.update_ime(true);

        let lcd_enable = (self.r.lcdc >> 7) & 1 != 0;

        if !self.test_mode && lcd_enable {
            self.renderer();
        };

        self.clock += 1;
    }

    pub fn check_and_trigger_ly_coincidence(&mut self) {
        let stat_old = self.r.stat;
        if self.r.ly == self.r.lyc {
            self.r.stat |= 1 << 2;
        } else {
            self.r.stat &= (!1) << 2;
        }
        if (self.r.stat >> 6) & (self.r.stat >> 2) & 1 != 0 && (stat_old >> 2) & 1 == 0 {
            self.request_interrupt(InterruptType::LCD);
        }
    }

    fn update_timers(&mut self) {
        self.r.div_16 += 1;

        if self.timer.wait_reload > 0 && self.timer.wait_reload < 4 {
            self.timer.wait_reload += 1;
            return;
        } else if self.timer.wait_reload == 4 {
            self.r.tima = self.r.tma;
            self.timer.wait_reload = 0;
            self.request_interrupt(InterruptType::Timer);
            return;
        }

        let bit_position = match self.r.tac & 3 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        };

        let bit = (self.r.div_16 >> bit_position) as u8 & 1;
        let timer_enable = (self.r.tac >> 2) & 1;
        let and_result = bit & timer_enable;

        if self.timer.prev_and_result == 1 && and_result == 0 {
            let overflow_check = self.r.tima as u16 + 1;
            if overflow_check > 0xFF {
                self.r.tima = 0;
                self.timer.wait_reload = 1;
            } else {
                self.r.tima = overflow_check as u8;
            }
        }

        self.timer.prev_and_result = and_result;
    }

    fn execute(&mut self) {
        if let Some(cycles_to_idle) = self.cycles_to_idle {
            if cycles_to_idle == 0 {
                let opcode: u8 = self.read(self.r.pc);
                self.r.pc += 1;
                self.cycles_to_idle = self.fetch_decode_execute(opcode);
            } else {
                self.cycles_to_idle = Some(self.cycles_to_idle.unwrap() - 1);
            }
        }
    }

    fn handle_interrupts(&mut self) {
        self.clock += 1;
        match self.isr_state {
            IsrState::Wait1 => {
                self.logger.log_info("ISR Wait1");
                self.isr_state = IsrState::Wait2
            }
            IsrState::Wait2 => {
                self.logger.log_info("ISR Wait2");
                self.isr_state = IsrState::PCPush1
            }
            IsrState::PCPush1 => {
                self.logger.log_info("ISR PCPush1");
                self.r.sp -= 1;
                self.write(self.r.sp, util::msb(self.r.pc));
                self.isr_state = IsrState::PCPush2
            }
            IsrState::PCPush2 => {
                self.logger.log_info("ISR PCPush2");
                self.r.sp -= 1;
                self.write(self.r.sp, util::lsb(self.r.pc));
                self.isr_state = IsrState::Jump;
            }
            IsrState::Jump => {
                self.logger.log_info("ISR Jump");
                let mut interrupt_index = None;
                for i in 0..5 {
                    if (((self.r.ie & self.r.r#if) >> i) & 1) != 0 {
                        interrupt_index = Some(i);
                    }
                }

                if let Some(i) = interrupt_index {
                    self.ime = false;
                    self.r.pc = [0x40, 0x48, 0x50, 0x58, 0x60][i];

                    self.cancel_interrupt_by_index(i as u8);
                }

                self.isr_state = IsrState::Wait1;
                self.state = State::Execute;
                self.cycles_to_idle = Some(0);
            }
        }
    }

    pub fn request_interrupt(&mut self, interrupt_type: InterruptType) {
        self.r.r#if |= 1 << (interrupt_type as u8);
    }

    pub fn cancel_interrupt(&mut self, interrupt_type: InterruptType) {
        self.r.r#if &= !(1 << interrupt_type as u8);
    }

    pub fn cancel_interrupt_by_index(&mut self, interrupt_index: u8) {
        self.r.r#if &= !(1 << interrupt_index);
    }

    pub fn press_key(&mut self, mut key_id: u8) {
        // 3: start, 2: select 1: b, 0: a, 7: down, 6: up, 5: left, 4: right
        let mut keys: &mut u8 = &mut self.keys_ssba;
        if key_id > 3 {
            keys = &mut self.keys_dulr;
            key_id -= 4;
        };
        *keys &= !(1 << key_id);
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
