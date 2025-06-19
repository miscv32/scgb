
use dmg::memory::Memory;

use crate::{renderer::Renderer, ScgbGui};

impl ScgbGui {
    pub fn draw(&mut self) {
        let buf = self.renderer.buffer_mut();
            
        for y in 0..144 {
            for x in 0..160 {
                buf[y * 160 + x] = 3;
            }
        }
        
        self.renderer.update().unwrap();
    }
}

