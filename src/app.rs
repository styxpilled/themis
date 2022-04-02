use bytesize::ByteSize;
use eframe::{egui, epi};
use notify::{Event, RecursiveMode, Watcher};
use std::env::{current_dir, set_current_dir};
use std::ffi::OsString;
use std::fs::read_dir;
use std::sync::mpsc;
use std::thread;
// use std::time::Duration;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct App {
  path_search: String,
  rename_bar: String,
  current_path: std::path::PathBuf,
  pinned_dirs: Vec<std::path::PathBuf>,
  last_path: std::path::PathBuf,
  drive_list: Vec<OsString>,
  #[cfg_attr(feature = "persistence", serde(skip))]
  filesystem: mft_ntfs::Filesystem,
  #[cfg_attr(feature = "persistence", serde(skip))]
  receiver: mpsc::Receiver<mft_ntfs::Filesystem>,
  #[cfg_attr(feature = "persistence", serde(skip))]
  dwatcher: crossbeam_channel::Receiver<Event>,
  dir_entries: Vec<DirEntry>,
}

impl Default for App {
  fn default() -> Self {
    let mut dir_entries = Vec::new();
    let path = current_dir().unwrap();
    if let Ok(dir) = read_dir(path) {
      for entry in dir {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        let name = entry.file_name().into_string().unwrap();
        let size = metadata.len();
        let dir_entry = DirEntry {
          name,
          path: entry.path(),
          size,
        };
        dir_entries.push(dir_entry);
      }
    }
    Self {
      path_search: current_dir().unwrap().to_str().unwrap().to_owned(),
      rename_bar: "".to_owned(),
      pinned_dirs: Vec::new(),
      current_path: current_dir().unwrap(),
      drive_list: Vec::new(),
      last_path: current_dir().unwrap(),
      dir_entries,
      receiver: mpsc::channel().1,
      dwatcher: crossbeam_channel::unbounded().1,
      filesystem: mft_ntfs::Filesystem::new(),
    }
  }
}

#[derive(Debug)]
enum Error {
  // dont panic
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct DirEntry {
  path: std::path::PathBuf,
  name: String,
  size: u64,
}
impl Default for DirEntry {
  fn default() -> Self {
    Self {
      path: current_dir().unwrap(),
      name: current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .split('/')
        .last()
        .unwrap()
        .to_owned(),
      size: 0,
    }
  }
}

impl epi::App for App {
  fn name(&self) -> &str {
    "project themis"
  }
  /// Called once before the first frame.
  fn setup(
    &mut self,
    _ctx: &egui::Context,
    _frame: &epi::Frame,
    storage: Option<&dyn epi::Storage>,
  ) {
    // Load previous app state (if any).
    // Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    if let Some(storage) = storage {
      *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
    }

    self.drive_list = mft_ntfs::get_drive_list();

    let path = self.current_path.clone();
    let (tx, rx) = crossbeam_channel::unbounded();

    let (sender, receiver): (
      crossbeam_channel::Sender<Event>,
      crossbeam_channel::Receiver<Event>,
    ) = crossbeam_channel::unbounded();

    self.dwatcher = receiver;

    thread::spawn(move || {
      let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
          tx.send(event).unwrap();
        }
        Err(e) => println!("watch error: {:?}", e),
      })
      .unwrap();
      watcher
        .watch(&path.clone(), RecursiveMode::Recursive)
        .unwrap();
      loop {
        match rx.recv() {
          Ok(event) => {
            sender.send(event).unwrap();
          }
          Err(e) => println!("watch error: {:?}", e),
        }
      }
    });

    let (sender, receiver) = mpsc::channel();
    self.receiver = receiver;

    thread::spawn(move || {
      let val = mft_ntfs::main(None);
      let val = match val {
        Ok(val) => val,
        Err(err) => {
          println!("{:?}", err);
          return;
        }
      };
      sender.send(val).unwrap();
    });
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
      path_search,
      rename_bar,
      pinned_dirs,
      current_path,
      drive_list,
      last_path,
      dir_entries,
      filesystem,
      receiver,
      dwatcher,
    } = self;

    if filesystem.files.is_empty() {
      let output = receiver.try_recv();
      if let Ok(output) = output {
        *filesystem = output;
      }
    }

    let recv = dwatcher.try_recv();
    if let Ok(event) = recv {
      // println!("Event {:?}", event);
      if event.kind == notify::EventKind::Create(notify::event::CreateKind::Any)
        || event.kind == notify::EventKind::Remove(notify::event::RemoveKind::Any)
      {
        println!("{:?}", event.paths);
      }
    }

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
      ui.heading("Pinned:");
      for pin in pinned_dirs.clone() {
        if ui.button(pin.to_str().unwrap()).clicked() {
          *current_path = pin;
        }
      }
      ui.heading("Drives:");
      for drive in drive_list.clone() {
        if ui.button(drive.to_str().unwrap()).clicked() {
          *current_path = std::path::PathBuf::from(drive);
        }
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
      // * Breadcrumb navigation
      ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        // TODO: maybe use a PathBuf instead of a String?
        // there are some problems with using a PathBuf
        for dir in path_search.clone().split('\\') {
          ui.label(">");
          let mut path = path_search.clone();
          path.truncate(path.find(dir).unwrap() + dir.len());
          let popup_id = ui.make_persistent_id(path.clone());
          let dir = ui.button(dir);
          if dir.clicked() {
            ui.memory().toggle_popup(popup_id);
          }
          egui::popup::popup_below_widget(ui, popup_id, &dir, |ui| {
            ui.set_min_width(75.0);
            if let Ok(popup_dir) = read_dir(path.clone()) {
              for dir in popup_dir {
                let dir = dir.unwrap();
                let dir_path = dir.path();
                if dir.metadata().unwrap().is_dir()
                  && ui.button(dir.file_name().to_str().unwrap()).clicked()
                {
                  *current_path = dir_path;
                }
              }
            }
          });
        }
      });
      ui.end_row();
      let search = ui.text_edit_singleline(path_search);

      if search.changed() {
        *current_path = std::path::PathBuf::from(path_search.clone());
      }
      ui.end_row();

      ui.horizontal(|ui| {
        if ui.button("Go up").clicked() {
          *current_path = current_path.parent().unwrap().to_path_buf();
        }
        if ui.button("Go back").clicked() {
          *current_path = last_path.to_path_buf();
        }
        if pinned_dirs.contains(current_path) {
          if ui.button("Unpin directory").clicked() {
            pinned_dirs.retain(|x| x != &current_path.clone());
          }
        } else if ui.button("Pin directory").clicked() {
          pinned_dirs.push(current_path.to_path_buf());
        }
        let _new_dir_name = ui.text_edit_singleline(rename_bar);
        if ui.button("New directory").clicked() {
          let new_dir_path = current_path.join(rename_bar);
          std::fs::create_dir(new_dir_path).unwrap();
          // *current_path = new_dir_path;
        }
      });
      ui.end_row();
      egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("central_grid").show(ui, |ui| {
          if search.lost_focus() && ui.input().key_pressed(egui::Key::Enter)
            || current_path != last_path
          {
            let dir_path = std::path::Path::new(&current_path);
            if let Ok(dir) = read_dir(dir_path) {
              set_current_dir(dir_path).unwrap();
              *path_search = dir_path.to_str().unwrap().to_owned();
              *current_path = dir_path.to_path_buf();
              *dir_entries = Vec::new();

              for entry in dir {
                let entrypath = entry.unwrap().path();
                let entrypath2 = entrypath.clone().into_os_string().into_string().unwrap();
                let dir_size = match filesystem.files.get(&entrypath2) {
                  Some(dir_size) => dir_size.real_size,
                  None => 0,
                };

                dir_entries.push(DirEntry {
                  name: entrypath
                    .clone()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                  path: entrypath,
                  size: dir_size,
                });
              }
            }
            *last_path = current_path.clone();
          }

          for entry in dir_entries {
            let name = entry.name.clone();
            let path = entry.path.clone();
            let is_dir = entry.path.is_dir();
            let dir_size = ByteSize(entry.size);
            let label = if is_dir {
              format!("{}/", name)
            } else {
              name.to_owned()
            };

            let entry = ui.button(&label);

            if entry.clicked() {
              if is_dir {
                *current_path = path.to_path_buf()
              } else {
                open::that(path.to_str().unwrap()).unwrap();
              }
            }

            let popup_id = ui.make_persistent_id(path.clone().to_string_lossy() + "popup");
            egui::popup::popup_below_widget(ui, popup_id, &entry, |ui| {
              ui.set_min_width(75.0);
              ui.button("Open").clicked();
              ui.button("Cut").clicked();
              ui.button("Copy").clicked();
              ui.button("Rename").clicked();
            });
            if entry.secondary_clicked() {
              ui.memory().toggle_popup(popup_id);
            }
            ui.label(label);
            ui.label(dir_size.to_string());
            ui.end_row();
          }
        });
      });
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
