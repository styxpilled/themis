use crate::app::{DirEntry, DirWatcherEvent, Themis};
use bytesize::ByteSize;
use eframe::egui;
use std::env::set_current_dir;
use std::fs::read_dir;

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
      ui.spacing_mut().item_spacing.x = 0.0;
      // TODO: maybe use a PathBuf instead of a String?
      // there are some problems with using a PathBuf
      for dir in state.path_search.clone().split('\\') {
        ui.label(">");
        let mut path = state.path_search.clone();
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
                state.current_path = dir_path;
              }
            }
          }
        });
      }
    });
    ui.end_row();
    let search = ui.text_edit_singleline(&mut state.path_search);

    if search.changed() {
      state.current_path = std::path::PathBuf::from(state.path_search.clone());
    }
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
      if ui.button("New directory").clicked() {
        let new_dir_path = state.current_path.join(state.rename_bar.clone());
        std::fs::create_dir(new_dir_path).unwrap();
      }
      if ui.button("New file").clicked() {
        let new_file_path = state.current_path.join(state.rename_bar.clone());
        std::fs::File::create(new_file_path).unwrap();
      }
    });
    ui.end_row();
    ui.text_edit_singleline(&mut state.rename_bar);
    ui.end_row();
    egui::ScrollArea::vertical().show(ui, |ui| {
      egui::Grid::new("central_grid").show(ui, |ui| {
        if search.lost_focus() && ui.input().key_pressed(egui::Key::Enter)
          || state.current_path != state.last_path
        {
          update_current_dir(state);
        }
        ui.end_row();
        ui.spacing_mut().item_spacing.y = 1.5;
        ui.vertical(|ui| {
          for entry in state.dir_entries.clone() {
            ui.horizontal(|ui| {
              let name = entry.name.clone();
              let path = entry.path.clone();
              let is_dir = entry.path.is_dir();
              let dir_size = ByteSize(entry.size);
              let label = if is_dir {
                format!("{}/", name)
              } else {
                name.to_owned()
              };
              let formatted = if is_dir {
                if entry.is_empty {
                  format!("🗁 {} ({})", label, dir_size)
                } else {
                  format!("🗀 {} ({})", label, dir_size)
                }
              } else {
                format!("🗋 {} ({})", label, dir_size)
              };
              let thing = ui.add(egui::Label::new(formatted).sense(egui::Sense::click()));
              if thing.double_clicked() {
                if is_dir {
                  state.current_path = path.to_path_buf()
                } else {
                  open::that(path.to_str().unwrap()).unwrap()
                }
              }
              if thing.hovered() {
                state.selected_path = path.to_path_buf();
              }
            });
            ui.end_row();
            ui.add(egui::Separator::spacing(
              egui::Separator::horizontal(egui::Separator::default()),
              0.0,
            ));
            ui.end_row();
          }
        })
        .response
        .context_menu(|ui| {
          if ui.button("Print Name").clicked() {
            println!("{:?}", state.selected_path);
          }
        });
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
    state.path_search = dir_path.to_str().unwrap().to_owned();
    state.current_path = dir_path.to_path_buf();
    state.dir_entries = Vec::new();

    for entry in dir {
      let entrypath = entry.unwrap().path();
      let dir_size = match state
        .filesystem
        .files
        .get(&entrypath.clone().into_os_string().into_string().unwrap())
      {
        Some(dir_size) => dir_size.real_size,
        None => 0,
      };

      state.dir_entries.push(DirEntry {
        name: entrypath
          .clone()
          .file_name()
          .unwrap()
          .to_str()
          .unwrap()
          .to_owned(),
        path: entrypath.clone(),
        size: dir_size,
        is_dir: entrypath.clone().is_dir(),
        is_empty: entrypath.is_dir() && entrypath.read_dir().unwrap().count() == 0,
      });
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
