use egui::Key;

use crate::{ScgbGui};

impl ScgbGui {
    pub fn draw(&mut self, ctx: &egui::Context) {
        // TODO put the keymap somewhere reasonable
        let keymap: [Key; 8] = [
            Key::J,
            Key::K,
            Key::Num1,
            Key::Space,
            Key::D,
            Key::A,
            Key::W,
            Key::S,
        ];
        for j in 0..keymap.len() {
            if ctx.input(|i| i.key_pressed(keymap[j])) {
                self.gameboy.press_key(j as u8);
                self.gameboy.logger.log_info(&format!("key {} pressed", j));
            } else if ctx.input(|i| i.key_released(keymap[j])) {
                self.gameboy.unpress_key(j as u8);
                self.gameboy.logger.log_info(&format!("key {} released", j));
            }
        }
        for _ in 0..(17556) {
            self.gameboy.tick();
        }
    }
}
