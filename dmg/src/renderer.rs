use crate::gb::{GameBoy, InterruptType};

enum LCDStatus {
    PPUModeDrawing,
    PPUModeHBlank,
    PPUModeVBlank,
    PPUModeOAMScan,
}

impl GameBoy {
    pub fn renderer(&mut self) {
        if self.clock % 114 == 0 && self.r.ly == 144 {
            self.update_stat(LCDStatus::PPUModeVBlank);
            self.request_interrupt(InterruptType::VBlank);
        } else if self.clock % 114 == 0 && self.r.ly < 144{
            self.update_stat(LCDStatus::PPUModeOAMScan);
        } else if self.clock % 114 == 20 && self.r.ly < 144 {
            self.render_scanline();
            self.update_stat(LCDStatus::PPUModeDrawing)
        } else if self.clock % 114 == 63 {
            self.update_stat(LCDStatus::PPUModeHBlank);
            self.r.ly += 1;
        } else if self.clock % 114 == 113 && self.r.ly == 154 {
            self.backbuf_id += 1;
            self.backbuf_id %= 2;
            self.r.ly = 0;
        }
    }

    fn update_stat(&mut self, lcd_status: LCDStatus) {
        match lcd_status {
            LCDStatus::PPUModeDrawing => {
                                self.r.stat = (self.r.stat & 0xFC) | 3;
                            }
            LCDStatus::PPUModeHBlank => {
                                self.r.stat = self.r.stat & 0xFC;
                            }
            LCDStatus::PPUModeVBlank => {
                                self.r.stat = (self.r.stat & 0xFC) | 1;
                            }
            LCDStatus::PPUModeOAMScan => {
                                self.r.stat = (self.r.stat & 0xFC) | 2;
                    }
        }
    }

    fn backbuf(&mut self) -> &mut [u8; 160 * 144] {
        match self.backbuf_id {
            0 => &mut self.displaybuf_0,
            1 => &mut self.displaybuf_1,
            _ => panic!("self.backbuf_id has invalid value {:#x}", self.backbuf_id),
        }
    }

    pub fn display(&self) -> & [u8; 160 * 144] {
        match self.backbuf_id {
            0 => &self.displaybuf_1,
            1 => &self.displaybuf_0,
            _ => panic!("self.backbuf_id has invalid value {:#x}", self.backbuf_id),
        }
    }

    fn render_background(&mut self) {
        if self.r.lcdc & 1 == 0 {
            return;
        }
        let tilemap: u16 = match self.r.lcdc & 8 {
            0 => 0x9800,
            _ => 0x9c00,
        };
        for x in 0..160 {
            let x_plus_scroll = (x as u16 + self.r.scx as u16) % 256;
            let y_plus_scroll = (self.r.ly as u16 + self.r.scy as u16) % 256;
            let tile_x = x_plus_scroll % 8;
            let tile_y = y_plus_scroll % 8;

            let tile_num = self.read(tilemap + x_plus_scroll / 8 + (y_plus_scroll / 8) * 32);

            let mut tile_addr: u16;
            match self.r.lcdc & 0x10 {
                0 => {
                    tile_addr = (0x9000 + (tile_num as i32 * 16)) as u16;
                }
                _ => {
                    tile_addr = 0x8000 + (tile_num as u16) * 16;
                }
            };

            tile_addr += 2 * tile_y;
            let data_low = (self.read(tile_addr) >> (7 - tile_x)) & 1;
            let data_high = (self.read(tile_addr + 1) >> (7 - tile_x)) & 1;
            let ly = self.r.ly;
            self.backbuf()[ly as usize * 160 + x as usize] =
                self.map_background_palette(data_low | (data_high << 1));
            self.background[ly as usize * 160 + x as usize] = self.backbuf()[ly as usize * 160 + x as usize];
        }
    }

    fn render_scanline(&mut self) {
            self.render_background();
    }
}