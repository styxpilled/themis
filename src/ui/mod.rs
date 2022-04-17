use crate::app::{DirEntry, DirWatcherEvent, PanelOpen, Themis};
use eframe::egui;
use glob::Pattern;
use regex::Regex;
use std::env::set_current_dir;
use std::fs::read_dir;
use std::path::PathBuf;

mod file_menu;
mod main;
pub mod settings;
use file_menu::file_menu;

use self::settings::SearchMode;

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

pub fn update_current_dir(state: &mut Themis) {
  let dir_path = std::path::Path::new(&state.current_path);
  if let Ok(dir) = read_dir(dir_path) {
    set_current_dir(dir_path).unwrap();
    state.navigation = dir_path.to_str().unwrap().to_owned();
    state.current_path = dir_path.to_path_buf();
    state.dir_entries = Vec::new();

    let glob = Pattern::new(&state.search).unwrap_or(Pattern::new("").unwrap());
    let re = Regex::new(&state.search).unwrap_or(Regex::new("").unwrap());

    for entry in dir {
      let path = entry.unwrap().path();
      let name = path
        .clone()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

      if state.search == "" {
        update(state, name, path);
      } else if state.settings.search_mode == SearchMode::Glob && glob.matches(&name) {
        update(state, name, path);
      } else if state.settings.search_mode == SearchMode::Regex && re.is_match(&name) {
        update(state, name, path);
      } else if state.settings.search_mode == SearchMode::Contains && name.contains(&state.search) {
        update(state, name, path);
      }

      fn update(state: &mut Themis, name: String, path: PathBuf) {
        let size = match state
          .filesystem
          .files
          .get(&path.clone().into_os_string().into_string().unwrap())
        {
          Some(size) => size.real_size,
          None => 0,
        };
        state.dir_entries.push(DirEntry {
          name,
          path: path.clone(),
          size,
          is_dir: path.clone().is_dir(),
          is_empty: path.is_dir() && path.read_dir().unwrap().count() == 0,
        });
      }
    }
  }
  if state.last_path != state.current_path {
    state
      .dir_watcher
      .watcher_updater
      .send((DirWatcherEvent::Remove, state.last_path.clone()))
      .unwrap();
    state
      .dir_watcher
      .watcher_updater
      .send((DirWatcherEvent::Add, state.current_path.clone()))
      .unwrap();
    state.last_path = state.current_path.clone();
  }
}
