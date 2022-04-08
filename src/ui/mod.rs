use crate::app::{DirEntry, DirWatcherEvent, Themis};
use eframe::egui;
use std::env::set_current_dir;
use std::fs::read_dir;
use regex::Regex;
use glob::Pattern;

mod file_menu;
use file_menu::file_menu;

pub fn main(ctx: &egui::Context, state: &mut Themis) {
  if state.filesystem.files.is_empty() {
    let output = state.fs_receiver.try_recv();
    if let Ok(output) = output {
      state.filesystem = output;
    }
  }

  let recv = state.dir_watcher.dir_watcher.try_recv();
  if let Ok(event) = recv {
    if event.kind == notify::EventKind::Create(notify::event::CreateKind::Any)
      || event.kind == notify::EventKind::Remove(notify::event::RemoveKind::Any)
    {
      // println!("{:?}", event.paths);
      update_current_dir(state);
    }
  }

  egui::SidePanel::left("side_panel").show(ctx, |ui| {
    ui.heading("( ._.)");
    ui.heading("Pinned:");
    for pin in state.pinned_dirs.clone() {
      if ui.button(pin.to_str().unwrap()).clicked() {
        state.current_path = pin;
      }
    }
    ui.heading("Drives:");
    for drive in state.drive_list.clone() {
      if ui.button(drive.to_str().unwrap()).clicked() {
        state.current_path = std::path::PathBuf::from(drive);
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
      ui.label("🥺");
      ui.spacing_mut().item_spacing.x = 1.5;
      let test = std::path::PathBuf::from(state.navigation.clone());
      let mut searchable_path = std::path::PathBuf::default();
      for (index, path) in test.iter().enumerate() {
        if index != 1 || path.to_str().unwrap() != "\\" {
          searchable_path.push(path);
          if index == 0 {
            searchable_path.push("\\");
          }
          ui.label("▶");
          let dir =
            ui.add(egui::Label::new(path.to_str().unwrap_or_default()).sense(egui::Sense::click()));
          let popup_id = ui.make_persistent_id(searchable_path.clone());
          if dir.clicked() {
            ui.memory().toggle_popup(popup_id);
          }
          egui::popup::popup_below_widget(ui, popup_id, &dir, |ui| {
            ui.set_width(150.0);
            if let Ok(popup_dir) = read_dir(searchable_path.clone()) {
              for dir in popup_dir {
                let dir = dir.unwrap();
                let dir_path = dir.path();
                if dir.metadata().unwrap().is_dir()
                  && ui.button(dir.file_name().to_str().unwrap()).clicked()
                {
                  state.current_path = dir_path;
                }
              }
            }
          });
        }
      }
    });
    ui.end_row();
    ui.horizontal(|ui| {
      // * Navigation bar
      let navigation = ui.text_edit_singleline(&mut state.navigation);

      if navigation.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
        state.current_path = std::path::PathBuf::from(state.navigation.clone());
        update_current_dir(state);
        // * Very important piece of logic that needs to be moved
      } else if state.current_path != state.last_path {
        update_current_dir(state);
      }

      // * Search bar
      let search = ui.text_edit_singleline(&mut state.search);
      if search.changed() {
        update_current_dir(state);
      }
    });

    ui.end_row();

    ui.horizontal(|ui| {
      if ui.button("Go up").clicked() {
        state.current_path = state.current_path.parent().unwrap().to_path_buf();
      }
      if ui.button("Go back").clicked() {
        state.current_path = state.last_path.to_path_buf();
      }
      if state.pinned_dirs.contains(&state.current_path) {
        if ui.button("Unpin directory").clicked() {
          state
            .pinned_dirs
            .retain(|x| x != &state.current_path.clone());
        }
      } else if ui.button("Pin directory").clicked() {
        state.pinned_dirs.push(state.current_path.to_path_buf());
      }
      // if ui.button("New directory").clicked() {
      //   let new_dir_path = state.current_path.join(state.rename_bar.clone());
      //   std::fs::create_dir(new_dir_path).unwrap();
      // }
      // if ui.button("New file").clicked() {
      //   let new_file_path = state.current_path.join(state.rename_bar.clone());
      //   std::fs::File::create(new_file_path).unwrap();
      // }
    });
    ui.end_row();
    egui::ScrollArea::vertical().show(ui, |ui| {
      egui::Grid::new("central_grid").show(ui, |ui| {
        ui.end_row();
        ui.spacing_mut().item_spacing.y = 1.5;
        // * Current directory file menu
        file_menu(state, ui);
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
