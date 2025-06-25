use crate::memory;
use crate::memory::Memory;
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

enum PPUState {
    OAMScan,
    Drawing,
    HBlank,
    VBlank,
}

enum FetcherState {
    ReadTileId,
    ReadTileData0,
    ReadTileData1,
    PushToFifo,
}
struct Fetcher {
    fifo: [u8; 16], // circular buffer
    tile_index: u8,
    map_addr: u16,
    tile_line: u8,
    state: FetcherState,
    clock: u8,
    tile_id: u8,
    pixel_data: [u8; 8],
    fifo_size: u8,
    fifo_front: u8,
    fifo_back: u8,
}

pub struct GameBoy {
    pub clock: u32, // measured in m-cycles, NOT T-cycles.
    pub running: bool,
    pub registers: Registers,
    pub cycles_to_idle: Option<u8>,
    pub memory: memory::FlatRAM,
    pub ime: bool,
    pub display_temp: [u8; 160*144], // contents of frame as ppu draws before vblank
    pub display: [u8; 160*144], // after vblank
    ppu_state: PPUState,
    pixel_count: u8,
    ppu_clock: u16,
    ly: u8,
    fetcher: Fetcher,
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

    let memory: [u8; 65536] = memory::init();

    GameBoy {
        clock: 0,
        ppu_clock: 0,
        running: true,
        registers: registers,
        cycles_to_idle: Some(0),
        memory: memory,
        ime: false,
        display_temp: [0; 160*144],
        display: [0; 160*144],
        ppu_state: PPUState::OAMScan,
        pixel_count: 0,
        ly: 0,
        fetcher: Fetcher {
            fifo: [0; 16], // circular buffer
            tile_index: 0,
            map_addr: 0,
            tile_line: 0,
            state: FetcherState::ReadTileId,
            clock: 0,
            tile_id: 0,
            pixel_data: [0; 8],
            fifo_size: 0,
            fifo_back: 0,
            fifo_front: 0,
        }
    }
}

impl GameBoy {
    pub fn tick(&mut self) {
        // This should be called once every M-cycle.
        // Current behaviour is M-cycle faking, i.e. all work is done in first M-cycle
        // CPU & RAM idle for the rest of the instruction's M-cycles
        if self.running {
            if let Some(cycles_to_idle) = self.cycles_to_idle {
                if cycles_to_idle == 0 {
                    let opcode: u8 = self.memory.read(self.registers.pc);
                    self.registers.pc += 1;
                    self.cycles_to_idle = self.fetch_decode_execute(opcode);
                }
            }
            self.ppu_tick();
            self.clock += 4;
        }
    }
    fn ppu_tick(&mut self) {
        
        self.ppu_clock += 1;

        match self.ppu_state {
            PPUState::OAMScan => {
                if self.ppu_clock == 80 {
                    self.pixel_count = 0;
                    let tile_line = self.ly % 8;
                    let tilemap_row_addr = 0x9800 + (self.ly as u16 / 8) * 32;
                    self.start_fetcher(tilemap_row_addr, tile_line);
                    self.ppu_state = PPUState::Drawing;  
                }
            }
            PPUState::Drawing => {
                self.tick_fetcher();
                
                if self.fetcher.fifo_size <= 8 {
                    return
                }
                let pixel = self.fifo_pop();
                self.display_write(pixel);
                if pixel != 0 {
                    println!("{}" , pixel)
                }
                self.pixel_count += 1;
            
                if self.pixel_count == 160 {
                    // TODO emit some Hblank signal if this will be useful
                    self.ppu_state = PPUState::HBlank
                }
            }
            PPUState::HBlank => {
                if self.ppu_clock == 456 {
                    self.ppu_clock = 0;
                    self.ly += 1;
                    if self.ly == 144 {
                        self.vblank();
                        self.ppu_state = PPUState::VBlank;
                    } else {
                        self.ppu_state = PPUState::OAMScan;
                    }
                }

            }
            PPUState::VBlank => {
                if self.ppu_clock == 456 {
                    self.ppu_clock = 0;
                    self.ly += 1;
                    if self.ly == 153 {
                        self.ly = 0;
                        self.ppu_state = PPUState::OAMScan;
                    }
                }
            }
        }
    }

    fn tick_fetcher(&mut self) {
        self.fetcher.clock += 1;
        if self.fetcher.clock < 2 {
            return
        }
        self.fetcher.clock = 0;
        match self.fetcher.state {
            FetcherState::ReadTileId => {
                self.fetcher.tile_id = self.memory.read(
                    self.fetcher.map_addr + self.fetcher.tile_index as u16
                );

                self.fetcher.state = FetcherState::ReadTileData0;
            }
            FetcherState::ReadTileData0 => {
                let offset = 0x8000 + (self.fetcher.tile_id as u16) * 16;
                let addr = offset + (self.fetcher.tile_line as u16) * 2;
                
                let data = self.memory.read(addr);
                if data != 0 {
                    println!("{}",data)
                }
                for bit in 0..8 {
                        self.fetcher.pixel_data[bit] = (data >> bit) & 1;
                }
                self.fetcher.state = FetcherState::ReadTileData1;
            }
            FetcherState::ReadTileData1 => {
                let offset = 0x8000 + (self.fetcher.tile_id as u16) * 16;
                let addr = offset + (self.fetcher.tile_line as u16) * 2;
                let data = self.memory.read(addr);
                for bit in 0..8 {
                        self.fetcher.pixel_data[bit] |= ((data >> bit) & 1) << 1; 
                        if self.fetcher.pixel_data[bit] != 0 {
                            println!("{}", self.fetcher.pixel_data[bit])
                        }          
                }
                self.fetcher.state = FetcherState::PushToFifo;
            }
            FetcherState::PushToFifo => {
                if self.fetcher.fifo_size <= 8 {
                    for i in (0..8).rev() {
                        self.fifo_push(self.fetcher.pixel_data[i]);
                    }
                    self.fetcher.tile_index += 1;
                    self.fetcher.state = FetcherState::ReadTileId;
                }
            }
        }
    }

    fn start_fetcher(&mut self, map_addr: u16, tile_line: u8) {
        self.fetcher.tile_index = 0;
        self.fetcher.map_addr = map_addr;
        self.fetcher.tile_line = tile_line;
        self.fetcher.state = FetcherState::ReadTileId;
        self.fetcher.fifo = [0; 16];
        self.fetcher.fifo_front =  0;
        self.fetcher.fifo_back = 0;
        self.fetcher. fifo_size = 0;
        
    }

    // note that the fifo pop and push are not safe
    // you should check the size before you push or pop 
    fn fifo_push(&mut self, data: u8) {
        self.fetcher.fifo[self.fetcher.fifo_back as usize] = data;
        self.fetcher.fifo_back += 1;
        self.fetcher.fifo_back %= 16;
        self.fetcher.fifo_size += 1;
    }
    fn fifo_pop(&mut self) -> u8 {
        let value = self.fetcher.fifo[self.fetcher.fifo_front as usize];
        self.fetcher.fifo_front += 1;
        self.fetcher.fifo_front %= 16;
        self.fetcher.fifo_size -= 1;
        return value;
    }
    fn display_write(&mut self, c: u8) {
        self.display_temp[((self.ly * 160) + self.pixel_count) as usize] = c;
    }
    fn vblank(&mut self) {
        self.display = self.display_temp;
        self.display_temp = [0; 160*144];
    }
}