use eframe::{egui, epi};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
  // Example stuff:
  label: String,
  path: String,
  saved_path: std::path::PathBuf,
  // this how you opt-out of serialization of a member
  #[cfg_attr(feature = "persistence", serde(skip))]
  value: f32,
}

impl Default for App {
  fn default() -> Self {
    Self {
      // Example stuff:
      label: "Hello World!".to_owned(),
      value: 2.7,
      path: std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned(),
      saved_path: std::env::current_dir().unwrap(),
    }
  }
}

#[derive(Debug)]
enum Error {
  // dont panic
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
    // Load previous app state (if any).
    // Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    if let Some(storage) = _storage {
      *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
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
    let Self { label, value, path, saved_path } = self;

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
      });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      // The central panel the region left after adding TopPanel's and SidePanel's

      ui.heading(path.clone());
      ui.text_edit_singleline(path);
      ui.text_edit_singleline(path);
      // create a variable to hold the dir content that we set later
      let dir_path = std::path::Path::new(&path);
      if let Ok(_dir) = std::fs::read_dir(dir_path) {
        // saved_dir = dir.copy();
        *saved_path = dir_path.to_path_buf();
      }
      // check if saved dir has a value
      if let Ok(dir) = std::fs::read_dir(saved_path) {
        for entry in dir {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap();
            let is_dir = path.is_dir();
            let label = if is_dir {
              format!("{}/", name)
            } else {
              name.to_owned()
            };
            ui.label(label);
        }
      }
      // rewrite this to keep a copy of the last valid dir, and only render that
      // if the dir is valid

      egui::warn_if_debug_build(ui);
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
