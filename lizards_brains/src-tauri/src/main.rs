#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--mcp".to_string()) {
        pomodoro_ai::run_mcp_shim();
    } else {
        pomodoro_ai::run_gui();
    }
}
