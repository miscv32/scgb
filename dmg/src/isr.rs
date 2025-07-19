use crate::gb::GameBoy;

impl GameBoy {
    pub fn update_ime(&mut self, after_fde: bool) {
        if let Some(id) = self.ime_dispatch {
            if id > 0 {
                self.ime_dispatch = Some(id - 1);
                println!("Updating IME dispatch: {:?}", self.ime_dispatch);
            } else {
                match after_fde {
                    false => self
                        .logger
                        .log_error("attempt to set IME at wrong part of cycle"),
                    true => {
                        println!("IME set");
                        self.ime = true
                    }
                }
                self.ime_dispatch = None;
            }
        }
    }
}