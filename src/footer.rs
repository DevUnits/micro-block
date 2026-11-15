use eframe::egui::{self, RichText};

use crate::theme;

pub const URL: &str = "https://github.com/DevUnits";

pub fn show(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        let link = egui::Link::new(RichText::new("Powered by DevUnits").small().color(theme::MUTED));
        let response = ui.add(link).on_hover_cursor(egui::CursorIcon::PointingHand);
        if response.clicked() {
            let _ = webbrowser::open(URL);
        }
    });
}
