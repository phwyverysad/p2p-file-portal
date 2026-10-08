// Prevents console window on Windows (never flash black cmd window)
#![windows_subsystem = "windows"]

fn main() {
    local_file_sharer_lib::run();
}
