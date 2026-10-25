#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod capture_policy;
mod native_popup;
mod shell;

fn main() {
    shell::run();
}
