use eframe::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, TextStyle};

pub const BG: Color32 = Color32::from_rgb(13, 16, 14);
pub const CARD: Color32 = Color32::from_rgb(22, 26, 24);
pub const TEXT: Color32 = Color32::from_rgb(241, 245, 249);
pub const MUTED: Color32 = Color32::from_rgb(148, 163, 184);

pub const ACTIVE: Color32 = Color32::from_rgb(34, 197, 94);
pub const ACTIVE_HOVER: Color32 = Color32::from_rgb(74, 222, 128);
pub const LOCKED: Color32 = Color32::from_rgb(220, 38, 38);
pub const LOCKED_HOVER: Color32 = Color32::from_rgb(248, 80, 80);

pub const ERROR: Color32 = Color32::from_rgb(248, 113, 113);

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let candidates = [
        ("segoe", r"C:\Windows\Fonts\segoeui.ttf"),
        ("segoe-semibold", r"C:\Windows\Fonts\seguisb.ttf"),
        ("segoe-symbol", r"C:\Windows\Fonts\seguisym.ttf"),
    ];
    let mut loaded = Vec::new();
    for (name, path) in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.to_string(), std::sync::Arc::new(FontData::from_owned(bytes)));
            loaded.push(name.to_string());
        }
    }
    if !loaded.is_empty() {
        let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
        for name in loaded.iter().rev() {
            proportional.insert(0, name.clone());
        }
        if loaded.iter().any(|name| name == "segoe-semibold") {
            fonts.families.insert(
                FontFamily::Name("semibold".into()),
                vec!["segoe-semibold".into(), "segoe".into()],
            );
        }
        ctx.set_fonts(fonts);
    }
}

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = CARD;
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = MUTED;
    visuals.window_corner_radius = CornerRadius::same(12);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
    ] {
        widget.corner_radius = CornerRadius::same(8);
    }
    style.visuals = visuals;
    style.text_styles.insert(TextStyle::Heading, FontId::proportional(24.0));
    style.text_styles.insert(TextStyle::Body, FontId::proportional(15.0));
    style.text_styles.insert(TextStyle::Button, FontId::proportional(15.0));
    style.text_styles.insert(TextStyle::Small, FontId::proportional(13.0));
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    ctx.set_global_style(style);
}

pub fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color32::from_rgb(
        mix(from.r(), to.r()),
        mix(from.g(), to.g()),
        mix(from.b(), to.b()),
    )
}
