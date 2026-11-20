use eframe::egui::{self, Color32, FontId, TextStyle};
pub const ACCENT: Color32 = Color32::from_rgb(62, 135, 230);
pub const ERROR: Color32 = Color32::from_rgb(255, 151, 151);
pub const WARNING: Color32 = Color32::from_rgb(243, 202, 117);
pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = Color32::from_rgb(24, 28, 35);
    style.visuals.window_fill = Color32::from_rgb(30, 35, 43);
    style.visuals.extreme_bg_color = Color32::from_rgb(18, 22, 28);
    style.visuals.selection.bg_fill = Color32::from_rgb(35, 78, 128);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
    style.visuals.window_corner_radius = egui::CornerRadius::same(8);
    style.visuals.error_fg_color = ERROR;
    style.visuals.warn_fg_color = WARNING;
    for widget in [
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
    ] {
        widget.corner_radius = egui::CornerRadius::same(5);
    }
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.interact_size.y = 30.0;
    style.spacing.window_margin = egui::Margin::same(16);
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(21.0));
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_global_style(style);
}
pub fn primary(text: &str) -> egui::Button<'_> {
    egui::Button::new(egui::RichText::new(text).color(Color32::WHITE)).fill(ACCENT)
}
