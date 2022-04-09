use crate::app::Themis;
use eframe::egui;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
  pub search_mode: SearchMode,
  pub show_francis: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      search_mode: SearchMode::Glob,
      show_francis: true,
    }
  }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub enum SearchMode {
  Glob,
  Regex,
  Contains,
}

pub fn main(ctx: &egui::Context, state: &mut Themis) {
  egui::SidePanel::left("side_panel").show(ctx, |ui| {
    ui.heading("( ._.)");

    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
      ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
      });
      egui::warn_if_debug_build(ui);
    });
  });

  egui::CentralPanel::default().show(ctx, |ui| {
    ui.spacing_mut().item_spacing.x = 1.5;
    egui::ComboBox::from_label("Take your pick")
      .selected_text(format!("{:?}", state.settings.search_mode))
      .show_ui(ui, |ui| {
        ui.selectable_value(
          &mut state.settings.search_mode,
          SearchMode::Glob,
          "Glob",
        );
        ui.selectable_value(
          &mut state.settings.search_mode,
          SearchMode::Regex,
          "Regex",
        );
        ui.selectable_value(
          &mut state.settings.search_mode,
          SearchMode::Contains,
          "Contains",
        );
      });
    ui.checkbox( &mut state.settings.show_francis, "Show Francis");
  });
}
