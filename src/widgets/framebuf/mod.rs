// modelled after https://github.com/twvd/snow/blob/master/frontend_egui/src/widgets/framebuffer.rs

use crate::renderer::DisplayBuf;
use crate::util;
use crossbeam_channel::Receiver;
use eframe::egui;
use eframe::egui::Vec2;

pub struct FrameBufWidget {
    frame: Option<DisplayBuf>,
    frame_recv: Option<Receiver<DisplayBuf>>,
    texture: egui::TextureHandle,
    scale: f32,
    display_size: [u16; 2],
    response: Option<egui::Response>,
}

impl FrameBufWidget {
    pub fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        Self {
            frame: None,
            frame_recv: None,
            texture: ctx.egui_ctx.load_texture(
                "viewport",
                egui::ColorImage::new([0, 0], egui::Color32::BLACK),
                egui::TextureOptions::NEAREST,
            ),
            response: None,
            scale: 4.0,
            display_size: [0, 0],
        }
    }

    pub fn display_size_max_scaled(&self) -> egui::Vec2 {
        egui::Vec2::from(core::array::from_fn(|i| {
            f32::from(self.display_size[i]) * self.scale
        }))
    }

    pub fn scaled_height(&self) -> f32 {
        f32::from(self.display_size[1]) * self.scale
    }

    pub fn connect(&mut self, recv: Receiver<DisplayBuf>) {
        self.frame_recv = Some(recv);
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) -> egui::Response {
        if let Some(ref frame_recv) = self.frame_recv {
            if !frame_recv.is_empty() {
                let frame = frame_recv.recv().unwrap();
                self.display_size = [frame.width, frame.height];
                self.texture.set(
                    egui::ColorImage {
                        size: self.display_size.map(|i| i.into()),
                        pixels: Vec::from_iter(
                            frame.chunks_exact(1).map(|c| util::dmg_colour(c[0])),
                        ),
                    },
                    egui::TextureOptions::NEAREST,
                );
                self.frame = Some(frame);
            }
        }

        let size = self.texture.size_vec2();
        let sized_texture = egui::load::SizedTexture::new(&mut self.texture, size);
        let response = ui.add(
            egui::Image::new(sized_texture)
                .fit_to_fraction(Vec2::new(1.0, 1.0))
                .max_size(self.display_size_max_scaled())
                .maintain_aspect_ratio(true),
        );
        self.response = Some(response.clone());
        response
    }
}
