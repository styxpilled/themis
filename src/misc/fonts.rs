use eframe::egui;
use egui::FontFamily::{Proportional, Monospace};
use egui::FontData;

pub fn setup_custom_fonts(ctx: &egui::Context) {
  // Start with the default fonts.
  let mut fonts = egui::FontDefinitions::default();

  // Install Inter:
  fonts.font_data.insert(
    "Inter".to_owned(),
    FontData::from_static(include_bytes!("../../fonts/inter/static/Inter-Regular.ttf")),
  );

  // Put Inter first (highest priority) for proportional text:
  fonts
    .families
    .entry(Proportional)
    .or_default()
    .insert(0, "Inter".to_owned());

  // Put Inter as last fallback for monospace:
  fonts
    .families
    .entry(Monospace)
    .or_default()
    .push("Inter".to_owned());

  // Install Twemoji
  fonts.font_data.insert(
    "Twemoji".to_owned(),
    FontData::from_static(include_bytes!("../../fonts/Twemoji.ttf")),
  );

  // Put Twemoji as last fallback for proportional text:
  fonts
    .families
    .entry(Proportional)
    .or_default()
    .push("Twemoji".to_owned());

  // Put Twemoji as last fallback for monospace:
  fonts
    .families
    .entry(Monospace)
    .or_default()
    .push("Twemoji".to_owned());

  // Tell egui to use these fonts:
  ctx.set_fonts(fonts);
}
