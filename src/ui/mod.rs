use crate::app::{DirEntry, DirWatcherEvent, PanelOpen, Themis};
use eframe::egui;
use std::env::set_current_dir;
use std::fs::read_dir;
// use regex::Regex;
use glob::Pattern;

mod file_menu;
mod main;
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
  }
  else if state.panel_open == PanelOpen::Settings {
  }
}

pub fn update_current_dir(state: &mut Themis) {
  let dir_path = std::path::Path::new(&state.current_path);
  if let Ok(dir) = read_dir(dir_path) {
    set_current_dir(dir_path).unwrap();
    state.navigation = dir_path.to_str().unwrap().to_owned();
    state.current_path = dir_path.to_path_buf();
    state.dir_entries = Vec::new();

    // let re = Regex::new(&state.search).unwrap_or(Regex::new("").unwrap());
    let matcher = Pattern::new(&state.search).unwrap_or(Pattern::new("").unwrap());
    for entry in dir {
      let entrypath = entry.unwrap().path();
      let name = entrypath
        .clone()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      if state.search == "" || matcher.matches(&name) {
        let dir_size = match state
          .filesystem
          .files
          .get(&entrypath.clone().into_os_string().into_string().unwrap())
        {
          Some(dir_size) => dir_size.real_size,
          None => 0,
        };

        state.dir_entries.push(DirEntry {
          name,
          path: entrypath.clone(),
          size: dir_size,
          is_dir: entrypath.clone().is_dir(),
          is_empty: entrypath.is_dir() && entrypath.read_dir().unwrap().count() == 0,
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
