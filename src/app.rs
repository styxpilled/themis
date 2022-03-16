use eframe::{egui, epi};
use std::env::{current_dir, set_current_dir};
use std::fs::{read_dir};
use fs_extra::dir::get_size;
use bytesize::ByteSize;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
  // Example stuff:
  label: String,
  path: String,
  saved_path: std::path::PathBuf,
  dir_entries: Vec<DirEntry>,
  // this how you opt-out of serialization of a member
  #[cfg_attr(feature = "persistence", serde(skip))]
  value: f32,
}

impl Default for App {
  fn default() -> Self {
    let mut dir_entries = Vec::new();
    let path = current_dir().unwrap();
    if let Ok(dir) = read_dir(path) {
      for entry in dir {
        let entrypath = entry.unwrap().path();
        let folder_size = get_size(&entrypath).unwrap();
        dir_entries.push(DirEntry {
          name: entrypath.file_name().unwrap().to_str().unwrap().to_owned(),
          path: entrypath,
          size: folder_size,
        });
      }
    }
    Self {
      // Example stuff:
      label: "Hello World!".to_owned(),
      value: 2.7,
      path: current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned(),
      saved_path: current_dir().unwrap(),
      dir_entries,
    }
  }
}

#[derive(Debug)]
enum Error {
  // dont panic
}


#[derive(Clone)]
struct DirEntry {
  path: std::path::PathBuf,
  name: String,
  size: u64,
}
impl Default for DirEntry {
  fn default() -> Self {
    Self {
      path: current_dir().unwrap(),
      name: current_dir().unwrap().to_str().unwrap().split('/').last().unwrap().to_owned(),
      size: get_size(current_dir().unwrap()).unwrap(),
      // contents: Vec::new(),
    }
  }
}


impl epi::App for App {
  fn name(&self) -> &str {
    "eframe template"
  }

  /// Called once before the first frame.
  fn setup(
    &mut self,
    _ctx: &egui::Context,
    _frame: &epi::Frame,
    _storage: Option<&dyn epi::Storage>,
  ) {
    let Self {
      label,
      value,
      path,
      saved_path,
      dir_entries
    } = self;
    // Load previous app state (if any).
    // Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    if let Some(storage) = _storage {
      *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    let dir_path = std::path::Path::new(&path);
    if let Ok(_dir) = read_dir(dir_path) {
      // saved_dir = dir.copy();
      *saved_path = dir_path.to_path_buf();
    }
  }

  /// Called by the frame work to save state before shutdown.
  /// Note that you must enable the `persistence` feature for this to work.
  #[cfg(feature = "persistence")]
  fn save(&mut self, storage: &mut dyn epi::Storage) {
    epi::set_value(storage, epi::APP_KEY, self);
  }

  /// Called each time the UI needs repainting, which may be many times per second.
  /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
  fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
    let Self {
      label,
      value,
      path,
      saved_path,
      dir_entries
    } = self;

    // Examples of how to create different panels and windows.
    // Pick whichever suits you.
    // Tip: a good default choice is to just keep the `CentralPanel`.
    // For inspiration and more examples, go to https://emilk.github.io/egui

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
      // The top panel is often a good place for a menu bar:
      egui::menu::bar(ui, |ui| {
        ui.menu_button("File", |ui| {
          if ui.button("Quit").clicked() {
            frame.quit();
          }
        });
      });
    });

    egui::SidePanel::left("side_panel").show(ctx, |ui| {
      ui.heading("Side Panel");

      ui.horizontal(|ui| {
        ui.label("Write something: ");
        ui.text_edit_singleline(label);
      });

      ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
      if ui.button("Increment").clicked() {
        *value += 1.0;
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
      // The central panel the region left after adding TopPanel's and SidePanel's

      ui.heading(path.clone());
      let search = ui.text_edit_singleline(path);
      // create a variable to hold the dir content that we set later
      if search.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
        let dir_path = std::path::Path::new(&path);
        if let Ok(dir) = read_dir(dir_path) {
          // saved_dir = dir.copy();
          set_current_dir(dir_path).unwrap();
          *saved_path = dir_path.to_path_buf();
          *dir_entries = Vec::new();
          
          for entry in dir {
            let entrypath = entry.unwrap().path();
            let folder_size = get_size(&entrypath).unwrap();
            dir_entries.push(DirEntry {
              name: entrypath.file_name().unwrap().to_str().unwrap().to_owned(),
              path: entrypath,
              size: folder_size,
            });
          }
        }
      }

      for entry in dir_entries {
        let name = entry.name.clone();
        let _path = entry.path.clone();
        let is_dir = entry.path.is_dir();
        let folder_size = ByteSize(entry.size);
        let label = if is_dir {
          format!("{}/", name)
        } else {
          name.to_owned()
        };
        ui.horizontal(|ui| {
          // ui.label(path.to_str().unwrap());
          ui.label(label);
          ui.label(folder_size.to_string());
        });
      }
    });

    if false {
      egui::Window::new("Window").show(ctx, |ui| {
        ui.label("Windows can be moved by dragging them.");
        ui.label("They are automatically sized based on contents.");
        ui.label("You can turn on resizing and scrolling if you like.");
        ui.label("You would normally chose either panels OR windows.");
      });
    }
  }
}
