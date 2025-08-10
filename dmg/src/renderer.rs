use crate::gb::{GameBoy, InterruptType, Sprite};
use std::cmp::max;
use std::mem::swap;

enum LCDStatus {
    PPUModeDrawing,
    PPUModeHBlank,
    PPUModeVBlank,
    PPUModeOAMScan,
}

impl GameBoy {
    pub fn renderer(&mut self) {
        let old_stat = self.r.stat;
        if self.clock % 114 == 0 && self.r.ly == 144 {
            self.update_stat(LCDStatus::PPUModeVBlank);
            self.request_interrupt(InterruptType::VBlank);
            self.backbuf_id += 1;
            self.backbuf_id %= 2;
            self.r.ly = 0;
            self.check_and_trigger_ly_coincidence();
            self.window_line_counter = 0;
        } else if self.clock % 114 == 0 && self.r.ly < 144 {
            self.update_stat(LCDStatus::PPUModeOAMScan);
        } else if self.clock % 114 == 20 && self.r.ly < 144 {
            self.render_scanline();

            self.r.ly += 1;

            self.check_and_trigger_ly_coincidence();

            self.update_stat(LCDStatus::PPUModeDrawing)
        } else if self.clock % 114 == 63 {
            self.update_stat(LCDStatus::PPUModeHBlank);
        } else if self.clock % 114 == 113 && self.r.ly == 154 {
            // self.backbuf_id += 1;
            // self.backbuf_id %= 2;
            // self.r.ly = 0;
            // self.window_line_counter = 0;
        }
        // check if theres any interrupts we need to trigger based on state changes
        if (((self.r.stat >> 5) & 1) != 0) && ((self.r.stat & 3) == 2) && ((old_stat & 3) != 2) {
            self.request_interrupt(InterruptType::LCD);
        }
        if (((self.r.stat >> 4) & 1) != 0) && ((self.r.stat & 3) == 1) && ((old_stat & 3) != 1) {
            self.request_interrupt(InterruptType::LCD);
        }
        if (((self.r.stat >> 3) & 1) != 0) && ((self.r.stat & 3) == 0) && ((old_stat & 3) != 0) {
            self.request_interrupt(InterruptType::LCD);
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

    pub fn display(&self) -> &[u8; 160 * 144] {
        match self.backbuf_id {
            0 => &self.displaybuf_1,
            1 => &self.displaybuf_0,
            _ => panic!("self.backbuf_id has invalid value {:#x}", self.backbuf_id),
        }
    }

    fn scan_oam(&mut self) -> Vec<Sprite> {
        self.oam_base = 0xFE00;
        self.num_sprites = 0;
        let mut sprite_buffer = vec![];
        for _ in 0..40 {
            let sprite = Sprite {
                size_y: (((self.r.lcdc >> 2) & 1) + 1) * 8,
                x: self.oam(1) as i16 - 8,
                y: self.oam(0) as i16 - 16,
                x_flip: (self.oam(3) >> 5) & 1,
                y_flip: (self.oam(3) >> 6) & 1,
                pal: (self.oam(3) >> 4) & 1,
                priority: (self.oam(3) >> 7) & 1,
                tile_num: self.oam(2),
            };
            if (sprite.x > 0)
                && (self.r.ly as i16 >= sprite.y)
                && ((self.r.ly as i16) < sprite.y + sprite.size_y as i16)
                && (sprite_buffer.len() < 10)
            {
                sprite_buffer.push(sprite);
            }
            self.oam_base += 4;
        }
        sprite_buffer
    }

    fn render_sprite(&mut self, sprite: Sprite) {
        let mut tile_num_top = sprite.tile_num;
        let mut tile_num_bottom = sprite.tile_num;

        if sprite.size_y == 16 {
            tile_num_top = sprite.tile_num & !1; // The top tile num, For the bottom tile we add 1
            tile_num_bottom = sprite.tile_num | 1;
        }

        if sprite.y_flip != 0 {
            swap(&mut tile_num_top, &mut tile_num_bottom);
        }

        let mut tile_addr_top = 0x8000 + tile_num_top as u16 * 16;

        let mut sprite_y = self.r.ly as i16 - sprite.y; // should be non-negative and less than sprite.size_y

        if sprite.y_flip != 0 {
            sprite_y = 7 - sprite_y;
        }

        tile_addr_top += 2 * sprite_y as u16;

        let tile_data_top_low_bits = self.read(tile_addr_top);
        let tile_data_top_high_bits = self.read(tile_addr_top + 1);

        self.render_sprite_tile(sprite, tile_data_top_low_bits, tile_data_top_high_bits);
    }

    fn render_sprite_tile(
        &mut self,
        sprite: Sprite,
        tile_data_low_bits: u8,
        tile_data_high_bits: u8,
    ) {
        for x in 0..8 {
            let screen_x = sprite.x + x;

            if screen_x < 0 {
                continue;
            }

            let shift = if sprite.x_flip != 0 { x } else { 7 - x };

            let pixel_data_low = (tile_data_low_bits >> (shift)) & 1;
            let pixel_data_high = (tile_data_high_bits >> (shift)) & 1;

            let index = (self.r.ly as usize) * 160 + screen_x as usize; // screen_x should be non-negative
            let colour = pixel_data_low | (pixel_data_high << 1);

            if index < 160 * 144 && colour != 0 {
                if sprite.priority == 0 {
                    self.backbuf()[index] = self.map_sprite_palette(sprite.pal, colour);
                } else if self.backbuf()[index] == 0 {
                    self.backbuf()[index] = self.map_sprite_palette(sprite.pal, colour);
                }
            }
        }
    }

    fn render_background(&mut self) {
        let tilemap_base_addr: u16 = match (self.r.lcdc >> 3) & 1 {
            0 => 0x9800,
            1 => 0x9C00,
            _ => unreachable!(),
        };

        for screen_x in 0..160 {
            let tile_y = (self.r.ly + self.r.scy) % 8;

            let x_off: u16 = ((screen_x as u16 + self.r.scx as u16) / 8) & 0x1F;
            let y_off: u16 = 32 * (((self.r.ly as u16 + self.r.scy as u16) & 0xFF) / 8);
            let tile_num_addr = tilemap_base_addr + ((x_off + y_off) & 0x3ff);
            let mut tile_num = self.read(tile_num_addr);

            let mut tile_addr: u16;
            let mut tile_data_base = 0;
            self.select_tile_addressing_method(&mut tile_num, &mut tile_data_base);
            tile_addr = tile_data_base + (tile_num as u16) * 16;

            tile_addr += 2 * tile_y as u16;

            let tile_data_low = self.read(tile_addr);
            let tile_data_high = self.read(tile_addr + 1);

            let tile_x = (screen_x + self.r.scx) % 8;

            let pixel_data_low = (tile_data_low >> (7 - tile_x)) & 1;
            let pixel_data_high = (tile_data_high >> (7 - tile_x)) & 1;

            let index: usize = self.r.ly as usize * 160 + screen_x as usize;

            if index < 160 * 144 {
                self.backbuf()[index] =
                    self.map_background_palette(pixel_data_low | (pixel_data_high << 1))
            }
        }
    }

    fn render_window(&mut self) {
        // I think i'm fetching the corect tile data but drawing it to the wrong place
        if self.r.ly < self.r.wy {
            return;
        }

        let tilemap_base_addr: u16 = match (self.r.lcdc >> 6) & 1 {
            0 => 0x9800,
            1 => 0x9C00,
            _ => unreachable!(),
        };

        let lb = max(0, self.r.wx as i16 - 7);
        let mut drew_pixels = false;
        for screen_x in lb..160 {
            drew_pixels = true;
            let index: usize = (self.r.ly as usize) * 160 + screen_x as usize;
            let x_off: u16 = (((screen_x - lb) as u16) / 8) & 0x1f;
            let y_off: u16 = 32 * ((self.window_line_counter as u16) / 8);
            let tile_num_addr = tilemap_base_addr + ((x_off + y_off) & 0x3ff);
            let mut tile_num = self.read(tile_num_addr);

            let mut tile_addr: u16;
            let mut tile_data_base = 0;
            self.select_tile_addressing_method(&mut tile_num, &mut tile_data_base);
            tile_addr = tile_data_base + (tile_num as u16) * 16;
            tile_addr += 2 * (self.window_line_counter as u16 % 8);

            let tile_data_low = self.read(tile_addr);
            let tile_data_high = self.read(tile_addr + 1);

            let tile_x = (screen_x) % 8;

            let pixel_data_low = (tile_data_low >> (7 - tile_x)) & 1;
            let pixel_data_high = (tile_data_high >> (7 - tile_x)) & 1;

            if index < 160 * 144 {
                self.backbuf()[index] =
                    self.map_background_palette(pixel_data_low | (pixel_data_high << 1))
            }
        }
        if drew_pixels {
            self.window_line_counter += 1;
        }
    }

    fn select_tile_addressing_method(&mut self, tile_num: &mut u8, tile_data_base: &mut u16) {
        match self.r.lcdc & 0x10 {
            0 => {
                if *tile_num <= 127 {
                    *tile_data_base = 0x9000;
                } else {
                    *tile_data_base = 0x8800;
                    *tile_num -= 128;
                }
            }
            _ => {
                *tile_data_base = 0x8000;
            }
        };
    }

    fn render_scanline(&mut self) {
        if ((self.r.lcdc >> 7) & 1) == 0 {
            return;
        }

        if (self.r.lcdc & 1) != 0 {
            self.render_background();
            if (self.r.lcdc >> 5) & 1 != 0 {
                self.render_window();
            }
        }

        if ((self.r.lcdc >> 1) & 1) != 0 {
            let sprites = self.scan_oam();
            for sprite in sprites {
                self.render_sprite(sprite)
            }
        }
    }
}
