use crate::{renderer::Renderer, ScgbGui};

impl ScgbGui {
    pub fn draw(&mut self) {
        let buf = self.renderer.buffer_mut();

        for _ in 0..175560 {
            self.gameboy.tick();
        }
        for y in 0..144 {
            for x in 0..160 {
                buf[y * 160 + x] = self.gameboy.display[y * 160 + x];
            }
        }
        self.renderer.update().unwrap();
    }
}
