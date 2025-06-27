pub fn dmg_colour(pixel: u8) -> egui::Color32 {
    match pixel {
        0 => egui::Color32::from_rgb(155, 188, 15),
        1 => egui::Color32::from_rgb(139, 172, 15),
        2 => egui::Color32::from_rgb(48, 98, 48),
        3 => egui::Color32::from_rgb(15, 56, 15),
        _ => egui::Color32::from_rgb(15, 56, 15),
    }
}
