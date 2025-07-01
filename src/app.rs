use crate::{
    renderer::{channel::ChannelRenderer, Renderer},
    widgets::framebuf::FrameBufWidget,
};
use dmg::gb;
use dmg::gb::GameBoy;
use dmg::memory::Memory;
use std::fs;
pub struct ScgbGui {
    pub framebuf: FrameBufWidget,
    pub renderer: ChannelRenderer,
    pub gameboy: GameBoy,
}

impl ScgbGui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let renderer = ChannelRenderer::new(160, 144).unwrap();
        let mut framebuf = FrameBufWidget::new(cc);
        framebuf.connect(renderer.get_receiver());
        let mut gameboy = gb::init();

        let data: Vec<u8> = fs::read("/home/spearmint/projects/scgb/test_roms/dmg_boot.bin")
            .expect("couldnt read file");
        let rom: Vec<u8> = fs::read("/home/spearmint/projects/scgb/test_roms/tetris.gb")
            .expect("couldnt read file");
        for i in 0..=0xFF {
            gameboy.memory.rom[i] = data[i];
        }
        for i in 0x00..rom.len() {
            gameboy.memory.write(i as u16, rom[i])
        }

        Self {
            framebuf,
            renderer,
            gameboy,
        }
    }
}

impl eframe::App for ScgbGui {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw();

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
                self.framebuf.draw(ui);
            });

            ctx.request_repaint();
        });
    }
}
