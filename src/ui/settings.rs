use eframe::egui;
use std::env::current_dir;
use std::path::PathBuf;

use crate::app::Themis;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Settings {
  pub search: SearchSettings,
  pub save_load: SaveLoadSettings,
  pub show_francis: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      search: SearchSettings::default(),
      save_load: SaveLoadSettings::default(),
      show_francis: true,
    }
  }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct SaveLoadSettings {
  pub location_input: String,
  pub location_is_valid: bool,
  pub location: PathBuf,
}

impl Default for SaveLoadSettings {
  fn default() -> Self {
    Self {
      location_input: "".to_owned(),
      location_is_valid: true,
      location: current_dir().unwrap(),
    }
  }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct SearchSettings {
  pub mode: SearchMode,
  pub sensitive: bool,
  pub recursive: bool,
  pub strict: bool,
}

impl Default for SearchSettings {
  fn default() -> Self {
    Self {
      mode: SearchMode::Glob,
      sensitive: false,
      recursive: false,
      strict: false,
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
    if state.settings.show_francis {
      ui.heading("( ._.)");
    }

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
    egui::ComboBox::from_label("Search Mode")
      .selected_text(format!("{:?}", state.settings.search.mode))
      .show_ui(ui, |ui| {
        ui.selectable_value(&mut state.settings.search.mode, SearchMode::Glob, "Glob");
        ui.selectable_value(&mut state.settings.search.mode, SearchMode::Regex, "Regex");
        ui.selectable_value(
          &mut state.settings.search.mode,
          SearchMode::Contains,
          "Contains",
        );
      });
    ui.checkbox(
      &mut state.settings.search.sensitive,
      "Search case sensitivity",
    );
    ui.checkbox(&mut state.settings.search.recursive, "Search recursive");
    ui.checkbox(&mut state.settings.search.strict, "Search strict");


    ui.horizontal(|ui| {
      if state.settings.save_load.location_is_valid {
        ui.visuals_mut().override_text_color = Some(egui::Color32::LIGHT_GREEN);
      }
      else {
        ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
      }
      let location_input = ui.text_edit_singleline(&mut state.settings.save_load.location_input);

      ui.label("Save/Load location");

      if location_input.changed() {
        // validate path
        let path = PathBuf::from(state.settings.save_load.location_input.clone());
        println!("{:?}", path);
        if path.is_dir() {
          state.settings.save_load.location = path;
          state.settings.save_load.location_is_valid = true;
        } else {
          state.settings.save_load.location_is_valid = false;
        }
      }
    });

    ui.checkbox(&mut state.settings.show_francis, "Show Francis");
  });
}
