use crate::app::Themis;
use bytesize::ByteSize;
use eframe::egui;

use crate::ui::update_current_dir;

pub fn file_menu(state: &mut Themis, ui: &mut egui::Ui) {
  ui.vertical(|ui| {
    let dir_entries;
    if state.search == "" {
      dir_entries = state.dir_entries.clone();
    } else {
      dir_entries = state.search_results.clone();
    }
    for entry in dir_entries {
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
        if state.rename.target.clone().unwrap_or_default() != path {
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
          thing.context_menu(|ui| {
            context_menu(state, ui);
          });
        } else {
          let rename_bar = ui.text_edit_singleline(&mut state.rename.value);
          if rename_bar.lost_focus() {
            if state.rename.value != "" {
              let new_name = state.rename.value.clone();
              let new_path = state
                .rename
                .target
                .clone()
                .unwrap()
                .with_file_name(new_name);
              std::fs::rename(state.rename.target.clone().unwrap(), new_path).unwrap();
              update_current_dir(state);
            }
            state.rename.target = None;
            state.rename.value = String::new();
          } else {
            rename_bar.request_focus();
          }
          if rename_bar.gained_focus() {
            state.rename.value = name.to_owned();
          }
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
    context_menu(state, ui);
  });

  fn context_menu(state: &mut Themis, ui: &mut egui::Ui) {
    if ui.button("Print Name").clicked() {
      println!("{:?}", state.selected_path);
    }
    if ui.button("Rename").clicked() {
      state.rename.target = Some(state.selected_path.clone());
      ui.close_menu();
    }
  }
}
