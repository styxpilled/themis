use crate::app::Themis;
use eframe::egui;
use std::fs::read_dir;

use super::file_menu;
use crate::misc::search::{update_current_dir, update_search};

pub fn main(ctx: &egui::Context, state: &mut Themis) {
  if state.filesystem.files.is_empty() {
    let output = state.fs_receiver.try_recv();
    if let Ok(output) = output {
      state.filesystem = output;
      update_current_dir(state);
    }
  }

  let recv = state.dir_watcher.dir_watcher.try_recv();
  if let Ok(event) = recv {
    if event.kind == notify::EventKind::Create(notify::event::CreateKind::Any)
      || event.kind == notify::EventKind::Remove(notify::event::RemoveKind::Any)
    {
      // println!("{:?}", event.paths);
      println!("updating because file changed");
      update_current_dir(state);
    }
  }

  egui::SidePanel::left("side_panel").show(ctx, |ui| {
    if state.settings.show_francis {
      ui.heading("( ._.)");
    }
    
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
        println!("updating because path changed");
        update_current_dir(state);
      }

      // * Search bar
      let search = ui.text_edit_singleline(&mut state.search);
      if search.changed() && state.search != "" {
        println!("updating because of search");
        update_search(state);
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
