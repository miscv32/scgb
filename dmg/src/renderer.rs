use crate::{gb::GameBoy};

impl GameBoy {
    pub fn renderer(&mut self) {
        if self.clock % 114 == 0 {
            if self.registers.ly == 144 {
                self.window_line_counter = 0;
                self.registers.stat &= 0b11111101;
                self.registers.r#if |= 1;
            } else {
                self.registers.stat &= 0b11111110;
            }
        } else if self.clock % 114 == 20 {
            if self.registers.ly < 144 {
                self.render_scanline();
                self.registers.stat &= 0b11111111;
            }
        } else if self.clock % 114 == 63 {
            self.registers.stat &= 0b11111100;
            self.registers.ly += 1;
        } else if self.clock % 114 == 113 {
            if self.registers.ly == 154 {
                self.backbuf_id += 1;
                self.backbuf_id %= 2;
                self.registers.ly = 0;
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
        if self.registers.lcdc & 1 == 0 {
            return;
        }
        let tilemap: u16 = match self.registers.lcdc & 8 {
            0 => 0x9800,
            _ => 0x9c00,
        };
        for x in 0..160 {
            let x_plus_scroll = (x as u16 + self.registers.scx as u16) % 256;
            let y_plus_scroll = (self.registers.ly as u16 + self.registers.scy as u16) % 256;
            let tile_x = x_plus_scroll % 8;
            let tile_y = y_plus_scroll % 8;

            let tile_num = self.read(tilemap + x_plus_scroll / 8 + (y_plus_scroll / 8) * 32);

            let mut tile_addr: u16;
            match self.registers.lcdc & 0x10 {
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
            let ly = self.registers.ly;
            self.backbuf()[ly as usize * 160 + x as usize] =
                self.map_background_palette(data_low | (data_high << 1));
        }
    }

    fn render_window(&mut self) {
        let tilemap: u16 = match self.registers.lcdc & 8 {
            0 => 0x9800,
            _ => 0x9c00,
        };
        for x in 0..160 {
            let x_plus_scroll = (x as u16 + self.registers.scx as u16) % 256;
            let y_plus_scroll = self.window_line_counter as u16;
            let tile_x = x_plus_scroll % 8;
            let tile_y = y_plus_scroll % 8;

            let tile_num = self.read(tilemap + x_plus_scroll / 8 + 32 * (y_plus_scroll as u16 / 8));

            let mut tile_addr: u16;
            match self.registers.lcdc & 0x10 {
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
            let ly = self.registers.ly;
            self.backbuf()[ly as usize * 160 + x as usize] =
                self.map_background_palette(data_low | (data_high << 1));
        }
    }
    fn _render_sprite_init(&mut self) {}
    fn _render_sprite(&mut self) {}

    fn render_scanline(&mut self) {
        if self.registers.lcdc & 1 != 0 {
            self.render_background();
        }
        if (self.registers.lcdc & 0x20 != 0)
            && self.registers.wx < 166
            && self.registers.ly >= self.registers.wy
        {
            self.render_window();
            self.window_line_counter += 1;
        }
    }
}