// Release derlemelerinde ek konsol penceresi açılmasını engeller.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    wakai_desktop_lib::run();
}
