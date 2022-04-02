#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] 

pub mod app;
pub mod ui;
use app::Themis;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = Themis::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
