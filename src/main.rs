mod config;
mod docker;
mod ops;
mod service;
mod ui;

use colored::*;
use config::load_config;
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use service::find_services;
use std::io::stdout;
use std::path::PathBuf;
use ui::clear_screen;

fn main() {
    let target_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    if let Err(e) = std::env::set_current_dir(&target_dir) {
        println!(
            "{}",
            format!("Error: Cannot access directory {:?}: {}", target_dir, e).red()
        );
        return;
    }

    let mut config = load_config();
    let services = find_services(&config);

    if services.is_empty() {
        println!(
            "{}",
            "No docker-compose.yml found in this directory structure.".red()
        );
        return;
    }

    let _ = execute!(stdout(), EnterAlternateScreen);

    loop {
        clear_screen();
        ops::print_header(services.len());
        match ops::show_main_menu() {
            Some(action) => ops::handle_action(&action, &services, &mut config),
            None => {
                println!("{}\n", "Goodbye!".green());
                break;
            }
        }
    }

    let _ = execute!(stdout(), LeaveAlternateScreen);
}
