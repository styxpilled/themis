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

pub fn update_search(state: &mut Themis) {
  if state.search == "" {}
  let dir_path = std::path::Path::new(&state.current_path);
  if let Ok(dir) = read_dir(dir_path) {
    state.search_results = Vec::new();
    let mut reg: String = "".to_owned();
    for part in state.current_path.into_iter() {
      if part != "\\" {
        reg.push_str(part.to_str().unwrap());
        reg.push_str("\\\\");
      }
    }
    reg.push_str(&state.search.clone());

    if state.settings.search_strict {
      reg.push('$');
    }
    let matcher = reg;
    println!("{}", matcher);

    let glob = Pattern::new(&matcher).unwrap_or(Pattern::new("!*").unwrap());
    let re = Regex::new(&matcher).unwrap_or(Regex::new("$-").unwrap());

    if state.settings.search_recursive && !state.filesystem.files.is_empty() {
      for (path, _entry) in state.filesystem.files.iter() {
        let search;
        if !state.settings.search_sensitive {
          search = path.clone().to_lowercase();
        } else {
          search = path.clone();
        };
        if state.settings.search_mode == SearchMode::Glob && glob.matches(&search)
          || state.settings.search_mode == SearchMode::Regex && re.is_match(&search)
          || re.is_match(&search)
        {
          state.search_results.push(update2(
            state,
            PathBuf::from(path.clone())
              .file_name()
              .unwrap()
              .to_str()
              .unwrap_or("")
              .to_string(),
            PathBuf::from(path.clone()),
          ));
        }
      }
    } else {
      for entry in dir {
        let path = entry.unwrap().path();
        let name = path
          .clone()
          .file_name()
          .unwrap()
          .to_str()
          .unwrap()
          .to_owned();

        let search = if state.settings.search_sensitive {
          path.clone().to_str().unwrap().to_string()
        } else {
          path.clone().to_str().unwrap().to_lowercase()
        };

        if state.search == ""
          || state.settings.search_mode == SearchMode::Glob && glob.matches(&search)
          || state.settings.search_mode == SearchMode::Regex && re.is_match(&search)
          || state.settings.search_mode == SearchMode::Contains && search.contains(&matcher)
        {
          state.search_results.push(update2(state, name, path));
        }
      }
    }
  }
}

pub fn update_current_dir(state: &mut Themis) {
  let dir_path = std::path::Path::new(&state.current_path);
  if let Ok(dir) = read_dir(dir_path) {
    set_current_dir(dir_path).unwrap();
    state.navigation = dir_path.to_str().unwrap().to_owned();
    state.current_path = dir_path.to_path_buf();
    state.dir_entries = Vec::new();

    for entry in dir {
      let path = entry.unwrap().path();
      let name = path
        .clone()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
      state.dir_entries.push(update2(state, name, path));
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

fn update2(state: &Themis, name: String, path: PathBuf) -> DirEntry {
  let size = match state
    .filesystem
    .files
    .get(&path.clone().into_os_string().into_string().unwrap())
  {
    Some(size) => size.real_size,
    None => 0,
  };
  DirEntry {
    name,
    path: path.clone(),
    size,
    is_dir: path.clone().is_dir(),
    is_empty: path.is_dir() && path.read_dir().unwrap().count() == 0,
  }
}
