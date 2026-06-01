#![windows_subsystem = "windows"]

#[path = "../common.rs"]
mod common;

fn main() {
    common::run_monitor_forever();
}
