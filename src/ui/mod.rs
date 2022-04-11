use crate::app::{PanelOpen, Themis};
use eframe::egui;

mod file_menu;
mod main;
pub mod settings;
use file_menu::file_menu;

pub fn main(ctx: &egui::Context, state: &mut Themis) {
  egui::TopBottomPanel::top("top_pannel").show(ctx, |ui| {
    if state.panel_open == PanelOpen::Main {
      if ui.button("Settings").clicked() {
        state.panel_open = PanelOpen::Settings;
      }
    } else if state.panel_open == PanelOpen::Settings {
      if ui.button("File Menu").clicked() {
        state.panel_open = PanelOpen::Main;
      }
    }
  });

  if state.panel_open == PanelOpen::Main {
    main::main(ctx, state);
  } else if state.panel_open == PanelOpen::Settings {
    settings::main(ctx, state);
  }
}