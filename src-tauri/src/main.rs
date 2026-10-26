#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod capture_policy;
mod latest_slot;
mod model_pack;
mod sentence_pipeline;
mod translation;
mod native_popup;
mod native_accessibility;
mod shell;

fn main() {
    shell::run();
}
