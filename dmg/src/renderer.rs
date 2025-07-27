use crate::gb::{GameBoy, InterruptType, Sprite};

enum LCDStatus {
    PPUModeDrawing,
    PPUModeHBlank,
    PPUModeVBlank,
    PPUModeOAMScan,
}

impl GameBoy {
    pub fn renderer(&mut self) {
        let old_stat  = self.r.stat;
        if self.clock % 114 == 0 && self.r.ly == 144 {
            self.update_stat(LCDStatus::PPUModeVBlank);
            self.request_interrupt(InterruptType::VBlank);
            self.backbuf_id += 1;
            self.backbuf_id %= 2;
            self.r.ly = 0;
            self.check_and_trigger_ly_coincidence();
            self.window_line_counter = 0;
        } else if self.clock % 114 == 0 && self.r.ly < 144{
            self.update_stat(LCDStatus::PPUModeOAMScan);
        } else if self.clock % 114 == 20 && self.r.ly < 144 {

            self.render_scanline();
            
            self.r.ly += 1;

            self.check_and_trigger_ly_coincidence(); // this is the culprit!

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
        if (((self.r.stat >> 3) & 1) != 0) && ((self.r.stat & 3) == 0) && ((old_stat & 3) != 0){
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

    pub fn display(&self) -> & [u8; 160 * 144] {
        match self.backbuf_id {
            0 => &self.displaybuf_1,
            1 => &self.displaybuf_0,
            _ => panic!("self.backbuf_id has invalid value {:#x}", self.backbuf_id),
        }
    }

    fn render_background(&mut self) {
        let tilemap: u16 = match self.r.lcdc & 8 {
            0 => 0x9800,
            _ => 0x9c00,
        };

        for x in 0..160 {
            let tile_x = (x + self.r.scx) % 8;
            let tile_y = (self.r.ly as u16 + self.r.scy as u16) % 8;

            let x_off: u16 = ((x as u16 + self.r.scx as u16) / 8) & 0x1f;
            let y_off: u16 = 32 * (((self.r.ly as u16 + self.r.scy as u16) & 0xFF) / 8);
            let tile_num_addr = tilemap + ((x_off + y_off) & 0x3ff);
            let tile_num = self.read(tile_num_addr);

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
            
            let index = ly as usize * 160 + x as usize;
            if index < 160 * 140 {
                self.backbuf()[index] =
                    self.map_background_palette(data_low | (data_high << 1));
                self.background[ly as usize * 160 + x as usize] = self.backbuf()[ly as usize * 160 + x as usize];
            }
        }
    }

    fn render_window(&mut self) {
        let tilemap: u16 = match (self.r.lcdc >> 6) & 1 {
            0 => 0x9800,
            _ => 0x9c00,
        };
        for x in 0..160 {
            let x_plus_scroll = (x as u16) % 256;
            let y_plus_scroll = (self.window_line_counter as u16) % 256;
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
            self.window_line_counter += 1;
        }
    }

    fn scan_oam(&mut self) -> Vec<Sprite>{
        self.oam_base = 0xFE00;
        self.num_sprites = 0;
        let mut sprite_buffer = vec![];
        for _ in 0..40 {
            let sprite = Sprite {
                size_y: (((self.r.lcdc >> 2) & 1) + 1) * 8,
                x: self.oam(1) as i16 - 8,
                y: self.oam(0) as i16,
                x_flip: (self.oam(3) >> 5) & 1,
                y_flip: (self.oam(3) >> 6) & 1,
                pal: (self.oam(3) >> 4) & 1,
                priority: (self.oam(3) >> 7) & 1,
                tile_num: self.oam(2),
            };
            if (sprite.x > 0) && (self.r.ly as i16 + 16 >= sprite.y) && (self.r.ly as i16 + 16 < sprite.y + sprite.size_y as i16) && (sprite_buffer.len() < 10) {
                sprite_buffer.push(sprite);
            } 
            self.oam_base += 4;
        }
        sprite_buffer
    }

    fn render_sprite(&mut self, sprite: Sprite) {

        let mut tile_y = ((self.r.ly as i16 + sprite.y) % 8) as u16;
        if sprite.y_flip != 0 {tile_y = 7 - tile_y}

        let tile_num = sprite.tile_num;

        let mut tile_addr: u16 = 0x8000 + (tile_num as u16) * 16;

        tile_addr += 2 * tile_y;

        for x in sprite.x..sprite.x+8 {    
            let mut tile_x = x as u16 - sprite.x as u16;
            if sprite.x_flip != 0 {tile_x = 7 - tile_x}
            let shift = 7 - tile_x;

            let data_low = (self.read(tile_addr) >> shift) & 1;
            let data_high = (self.read(tile_addr + 1) >> shift) & 1;
            let index = (self.r.ly) as usize * 160 + x as usize;
            let should_draw = true; // TODO add some semi-accurate condition here
            if should_draw {
                self.backbuf()[index] =
                    self.map_sprite_palette(sprite.pal, data_low | (data_high << 1));
            }
        }
    
    }

    fn render_scanline(&mut self) {
            if ((self.r.lcdc >> 7) & 1) == 0 {
                return;
            }
            if self.r.lcdc & 1 != 0 {
                self.render_background(); 
                //self.render_window();
            }

            // println!("{:#b}", self.r.lcdc);
            

            if ((self.r.lcdc >> 1) & 1) != 0 {
                let sprites = self.scan_oam();
                for sprite in sprites {
                    self.render_sprite(sprite);
                }
            }
    }
}