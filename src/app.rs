use crate::widgets::framebuf::FrameBufWidget;
use dmg::gb;
use dmg::gb::GameBoy;
use std::fs;
pub struct ScgbGui {
    pub framebuf: FrameBufWidget,
    pub gameboy: GameBoy,
}

impl ScgbGui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let framebuf = FrameBufWidget::new(cc);
        let mut gameboy = gb::init();

        let data: Vec<u8> = fs::read(
            "C:\\Users\\jodkm\\Documents\\Development\\rust\\scgb\\test_roms\\dmg_boot.bin",
        )
        .expect("couldnt read file");

        let rom: Vec<u8> = fs::read(
            "C:\\Users\\jodkm\\Documents\\Development\\rust\\scgb\\test_roms\\mooneye\\mbc1\\bits_bank2.gb",
        )
        .expect("couldnt read file");

        for i in 0..=0xFF {
            gameboy.memory.boot_rom[i] = data[i];
        }

        gameboy.memory.cartridge = rom;

        gameboy.mbc = gameboy.detect_mbc();
        let cart_decode = gameboy.decode_cart_header();
        gameboy.logger.log_info(&format!("{:?}", cart_decode));

        Self { framebuf, gameboy }
    }
}

impl eframe::App for ScgbGui {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("supercoolgb");

            ui.vertical_centered(|ui| {
                let padding_height = (ui.available_height() - self.framebuf.scaled_height()) / 2.0;
                if padding_height > 0.0 {
                    ui.allocate_space(egui::Vec2::from([1.0, padding_height]));
                }
                self.framebuf.draw(ui, &self.gameboy);
            });

            ctx.request_repaint(); // TODO make this run at exactly 59.7Hz
        });
    }
}
