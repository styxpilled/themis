use eframe::{egui, epi};
use notify::{Event, RecursiveMode, Watcher};
use std::env::current_dir;
use std::ffi::OsString;
use std::fs::read_dir;
use std::thread;

use crate::ui;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct Themis {
  pub path_search: String,
  pub rename_bar: String,
  pub current_path: std::path::PathBuf,
  pub pinned_dirs: Vec<std::path::PathBuf>,
  pub last_path: std::path::PathBuf,
  pub drive_list: Vec<OsString>,
  #[cfg_attr(feature = "persistence", serde(skip))]
  pub filesystem: mft_ntfs::Filesystem,
  #[cfg_attr(feature = "persistence", serde(skip))]
  pub fs_receiver: crossbeam_channel::Receiver<mft_ntfs::Filesystem>,
  #[cfg_attr(feature = "persistence", serde(skip))]
  pub dir_watcher: crossbeam_channel::Receiver<Event>,
  pub dir_entries: Vec<DirEntry>,
}

impl Default for Themis {
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
      fs_receiver: crossbeam_channel::unbounded().1,
      dir_watcher: crossbeam_channel::unbounded().1,
      filesystem: mft_ntfs::Filesystem::new(),
    }
  }
}

#[derive(Debug)]
enum Error {
  // dont panic
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct DirEntry {
  pub path: std::path::PathBuf,
  pub name: String,
  pub size: u64,
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

impl epi::App for Themis {
  fn name(&self) -> &str {
    "themis"
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

    let (sender, receiver) = crossbeam_channel::unbounded();

    self.dir_watcher = receiver;

    thread::spawn(move || {
      let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
          tx.send(event).unwrap();
        }
        Err(e) => println!("watch error: {:?}", e),
      })
      .unwrap();
      watcher
        .watch(&path, RecursiveMode::Recursive)
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

    let (sender, receiver) = crossbeam_channel::unbounded();
    self.fs_receiver = receiver;

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
  fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
    ui::main(ctx, self);
  }
}