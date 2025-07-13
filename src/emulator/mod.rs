use egui::Key;

use crate::{renderer::Renderer, ScgbGui};

impl ScgbGui {
    pub fn draw(&mut self, ctx: &egui::Context) {
        let buf = self.renderer.buffer_mut();
        // TODO put the keymap somewhere reasonable
        let keymap: [Key; 8] = [Key::J, Key::K, Key::Num1, Key::Space, Key::D, Key::A, Key::W, Key::S];
        for j in 0..keymap.len() {    
            if ctx.input(|i| i.key_pressed(keymap[j])) {
                self.gameboy.press_key(j as u8);
                println!("key {} pressed", j);
            } else if ctx.input(|i| i.key_released(keymap[j])) {
                self.gameboy.unpress_key(j as u8);
                println!("key {} not pressed", j);
            }
        }
        for _ in 0..(17556) {
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
