use colored::*;
use crossterm::cursor::{MoveTo};
use crossterm::event::{read, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use inquire::{Confirm, MultiSelect, Select};
use std::io::{stdout, Write};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::{fs, path::PathBuf};

#[derive(Clone, Debug)]
struct Service {
    name: String,
    path: PathBuf,
    compose_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    max_depth: Option<usize>,
    excluded_dirs: Vec<String>,
    #[serde(default)]
    legacy_compose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_depth: Some(7),
            excluded_dirs: Vec::new(),
            legacy_compose: false,
        }
    }
}

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("dockerstrator")
        .join("config.toml")
}

fn load_config() -> Config {
    let config_path = get_config_path();

    if config_path.exists() {
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }
    }

    Config::default()
}

fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let contents = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, contents).map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    // Parse optional directory argument
    let target_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    // Change to target directory
    if let Err(e) = std::env::set_current_dir(&target_dir) {
        println!("{}", format!("Error: Cannot access directory {:?}: {}", target_dir, e).red());
        return;
    }

    let mut config = load_config();
    let services = find_services(&config);

    if services.is_empty() {
        println!("{}", "No docker-compose.yml found in this directory structure.".red());
        return;
    }

    // Enter alternate screen to avoid polluting terminal history
    let _ = execute!(stdout(), EnterAlternateScreen);

    loop {
        clear_screen();
        println!("\n{}\n", "Docker Services Orchestrator".bold().cyan());
        println!("Services found: {}\n", services.len());
        match show_main_menu() {
            Some(choice) => match choice.as_str() {
                "Start" => start_services(&services, &config),
                "Stop" => stop_services(&services, &config),
                "Restart" => restart_services(&services, &config),
                "Status" => show_status(&services, &config),
                "Logs" => show_logs(&services, &config),
                "Cleanup" => cleanup_data(&services, &config),
                "Settings" => show_settings(&mut config),
                "Exit" => {
                    println!("{}\n", "Goodbye!".green());
                    break;
                }
                _ => {}
            },
            None => {
                println!("{}\n", "Goodbye!".green());
                break;
            }
        }
    }

    // Exit alternate screen and restore terminal
    let _ = execute!(stdout(), LeaveAlternateScreen);
}

fn find_services(config: &Config) -> Vec<Service> {
    let current_dir = std::env::current_dir().unwrap();
    let mut services = Vec::new();
    let mut visited = std::collections::HashSet::new();

    // Check root directory first
    let compose_files = get_compose_files(&current_dir);
    if !compose_files.is_empty() {
        if let Some(compose_file) = select_compose_file(&compose_files) {
            services.push(Service {
                name: "root".to_string(),
                path: current_dir.clone(),
                compose_file,
            });
        }
    }

    // Then scan subdirectories
    scan_directory(&current_dir, &mut services, &mut visited, config, 0);
    services.sort_by(|a, b| a.name.cmp(&b.name));
    services
}

fn scan_directory(
    dir: &PathBuf,
    services: &mut Vec<Service>,
    visited: &mut std::collections::HashSet<PathBuf>,
    config: &Config,
    depth: usize,
) {
    if visited.contains(dir) {
        return;
    }
    visited.insert(dir.clone());

    // Check max depth
    if let Some(max_depth) = config.max_depth {
        if depth >= max_depth {
            return;
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Skip hidden directories and common non-service directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Check hardcoded exclusions
                if name.starts_with('.') || matches!(name, "target" | "node_modules" | "vendor") {
                    continue;
                }

                // Check user-configured exclusions
                if config.excluded_dirs.contains(&name.to_string()) {
                    continue;
                }
            }

            if path.is_dir() {
                let compose_files = get_compose_files(&path);

                if !compose_files.is_empty() {
                    // Has compose files in this directory
                    if let Some(compose_file) = select_compose_file(&compose_files) {
                        if let Some(service_name) = path.file_name().and_then(|n| n.to_str()) {
                            services.push(Service {
                                name: service_name.to_string(),
                                path: path.clone(),
                                compose_file,
                            });
                        }
                    }
                } else {
                    // No compose files, recurse deeper
                    scan_directory(&path, services, visited, config, depth + 1);
                }
            }
        }
    }
}

fn get_compose_files(dir: &PathBuf) -> Vec<String> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Match: docker-compose.yml, docker-compose.prod.yml, compose.yaml, etc.
                    if (filename.starts_with("docker-compose") || filename.starts_with("compose"))
                        && (filename.ends_with(".yml") || filename.ends_with(".yaml"))
                    {
                        files.push(filename.to_string());
                    }
                }
            }
        }
    }

    files.sort();
    files
}

fn select_compose_file(files: &[String]) -> Option<String> {
    if files.is_empty() {
        return None;
    }

    if files.len() == 1 {
        return Some(files[0].clone());
    }

    // Multiple files: ask user to choose
    match Select::new("Multiple compose files found. Which one to use?", files.to_vec()).prompt() {
        Ok(selected) => Some(selected),
        Err(_) => None,
    }
}

fn clear_screen() {
    let _ = execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0));
}

fn interactive_menu(title: &str, items: &[(&str, &str)]) -> Option<char> {
    let n = items.len();

    let draw = |cur: usize| {
        // Clear only the menu area (skip header)
        let _ = execute!(stdout(), MoveTo(0, 5), Clear(ClearType::FromCursorDown));
        println!("{}", title.bold().cyan());
        for (i, (key, label)) in items.iter().enumerate() {
            if i == cur {
                println!("  {} {}", format!("[{}]", key).cyan(), label.bold());
            } else {
                println!("  {} {}", format!("[{}]", key).cyan(), label);
            }
        }
        println!();
        let _ = stdout().flush();
    };

    let mut cursor = 0usize;
    draw(cursor);

    let _ = enable_raw_mode();
    let result = loop {
        if let Ok(Event::Key(key)) = read() {
            match key.code {
                KeyCode::Up => {
                    cursor = if cursor == 0 { n - 1 } else { cursor - 1 };
                    let _ = disable_raw_mode();
                    draw(cursor);
                    let _ = enable_raw_mode();
                }
                KeyCode::Down => {
                    cursor = (cursor + 1) % n;
                    let _ = disable_raw_mode();
                    draw(cursor);
                    let _ = enable_raw_mode();
                }
                KeyCode::Enter => {
                    let _ = disable_raw_mode();
                    break Some(items[cursor].0.chars().next().unwrap());
                }
                KeyCode::Char(c) => {
                    let c = c.to_ascii_lowercase();
                    if items.iter().any(|(k, _)| k.chars().next() == Some(c)) {
                        let _ = disable_raw_mode();
                        break Some(c);
                    }
                }
                KeyCode::Esc => {
                    let _ = disable_raw_mode();
                    break None;
                }
                _ => {}
            }
        }
    };
    result
}

fn show_main_menu() -> Option<String> {
    match interactive_menu("Select an action:", &[
        ("s", "Start services"),
        ("p", "Stop services"),
        ("r", "Restart services"),
        ("t", "Show status"),
        ("l", "Stream logs"),
        ("c", "Cleanup volumes"),
        ("g", "Settings"),
        ("q", "Exit"),
    ]) {
        Some('s') => Some("Start".to_string()),
        Some('p') => Some("Stop".to_string()),
        Some('r') => Some("Restart".to_string()),
        Some('t') => Some("Status".to_string()),
        Some('l') => Some("Logs".to_string()),
        Some('c') => Some("Cleanup".to_string()),
        Some('g') => Some("Settings".to_string()),
        _ => None,
    }
}

fn select_services(services: &[Service]) -> Vec<Service> {
    let service_names: Vec<&str> = services.iter().map(|s| s.name.as_str()).collect();

    match MultiSelect::new("Select services:", service_names)
        .with_help_message("↑↓ navigate  SPACE select  ENTER confirm  ESC cancel")
        .with_formatter(&|items| {
            let selected: Vec<&str> = items.iter().map(|s| *s.value).collect();
            if selected.is_empty() {
                "None selected".to_string()
            } else {
                format!("{} selected", selected.len())
            }
        })
        .prompt()
    {
        Ok(selected) => services
            .iter()
            .filter(|s| selected.contains(&s.name.as_str()))
            .cloned()
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn start_services(services: &[Service], config: &Config) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    println!("\n{}\n", "Starting services...".yellow().bold());

    let mut success_count = 0;

    for service in &selected {
        print!("  {} ... ", service.name.cyan());

        if run_docker_compose(service, &["up", "-d"], config.legacy_compose) {
            println!("{}", "OK".green());
            success_count += 1;
        } else {
            println!("{}", "ERROR".red());
        }
    }

    println!("\n{}\n", format!("{}/{} services started", success_count, selected.len()).green());
}

fn stop_services(services: &[Service], config: &Config) {
    print!("{}", "Checking service status...".bright_black());
    let _ = stdout().flush();
    let running: Vec<Service> = services.iter()
        .filter(|s| get_service_status(s, config.legacy_compose))
        .cloned()
        .collect();
    println!();

    if running.is_empty() {
        println!("{}", "\nNo services are currently running.".yellow());
        pause();
        return;
    }

    let selected = select_services(&running);

    if selected.is_empty() {
        return;
    }

    if Confirm::new(&format!("Stop {} service(s)?", selected.len()))
        .with_default(false)
        .prompt()
        .unwrap_or(false)
    {
        println!("\n{}\n", "Stopping services...".yellow());

        let mut success_count = 0;

        for service in &selected {
            print!("  {} ... ", service.name.cyan());
            if run_docker_compose(service, &["down"], config.legacy_compose) {
                println!("{}", "OK".green());
                success_count += 1;
            } else {
                println!("{}", "ERROR".red());
            }
        }

        println!("\n{}\n", format!("{}/{} services stopped", success_count, selected.len()).green());
        pause();
    }
}

fn restart_services(services: &[Service], config: &Config) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    println!("\n{}\n", "Restarting services...".yellow());

    let mut success_count = 0;

    for service in &selected {
        print!("  {} ... ", service.name.cyan());
        if run_docker_compose(service, &["restart"], config.legacy_compose) {
            println!("{}", "OK".green());
            success_count += 1;
        } else {
            println!("{}", "ERROR".red());
        }
    }

    println!("\n{}\n", format!("{}/{} services restarted", success_count, selected.len()).green());
}

fn show_status(services: &[Service], config: &Config) {
    clear_screen();
    println!("\n{}\n", "Services Status:".bold().cyan());

    for service in services {
        let status = get_service_status(service, config.legacy_compose);
        let status_text = if status {
            "UP".green()
        } else {
            "DOWN".red()
        };
        println!("  {}: {}", service.name.cyan(), status_text);
    }

    println!();
    pause();
}

fn build_compose_cmd(service: &Service, legacy: bool) -> Command {
    let mut cmd = if legacy {
        Command::new("docker-compose")
    } else {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        cmd
    };
    cmd.current_dir(&service.path)
        .arg("-f")
        .arg(&service.compose_file);
    cmd
}

fn get_service_status(service: &Service, legacy: bool) -> bool {
    let output = build_compose_cmd(service, legacy)
        .args(&["ps", "-q"])
        .output();
    match output {
        Ok(out) => !out.stdout.is_empty(),
        Err(_) => false,
    }
}

fn show_logs(services: &[Service], config: &Config) {
    let service_names: Vec<&str> = services
        .iter()
        .map(|s| s.name.as_str())
        .chain(std::iter::once("All"))
        .collect();

    if let Ok(selected) = Select::new("Which service?", service_names).prompt() {
        println!("\n{}\n", "Streaming logs (Ctrl+C to return to menu)...".yellow());

        if selected == "All" {
            for service in services {
                stream_logs(service, config.legacy_compose);
            }
        } else {
            if let Some(service) = services.iter().find(|s| s.name == selected) {
                stream_logs(service, config.legacy_compose);
            }
        }
    }
}

fn stream_logs(service: &Service, legacy: bool) {
    let mut child = match build_compose_cmd(service, legacy)
        .args(&["logs", "-f"])
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    // Ignore Ctrl+C in parent while child is running
    let _ctrlc_guard = ctrlc::set_handler(|| {})
        .map_err(|_| ())
        .ok();

    // Wait for child to finish
    let _ = child.wait();

    // Ctrl+C handler will be restored when _ctrlc_guard is dropped
}

fn cleanup_data(services: &[Service], config: &Config) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    if Confirm::new(&format!(
        "Remove volumes from {} service(s)?",
        selected.len()
    ))
    .with_default(false)
    .prompt()
    .unwrap_or(false)
    {
        println!("\n{}", "Removing volumes...".red());

        let mut success_count = 0;

        for service in &selected {
            print!("  {} ... ", service.name.cyan());
            if run_docker_compose(service, &["down", "-v"], config.legacy_compose) {
                println!("{}", "OK".green());
                success_count += 1;
            } else {
                println!("{}", "ERROR".red());
            }
        }

        println!("\n{}\n", format!("{}/{} volumes removed", success_count, selected.len()).green());
    }

    pause();
}

fn run_docker_compose(service: &Service, args: &[&str], legacy: bool) -> bool {
    build_compose_cmd(service, legacy)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn show_settings(config: &mut Config) {
    loop {
        clear_screen();
        println!("\n{}", "Current configuration:".bold());
        println!("  Max search depth: {}",
            config.max_depth
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Unlimited".to_string())
                .yellow()
        );
        println!("  Excluded dirs: {}",
            if config.excluded_dirs.is_empty() {
                "None".to_string()
            } else {
                config.excluded_dirs.join(", ")
            }.yellow()
        );
        println!("  Docker command: {}",
            if config.legacy_compose { "docker-compose" } else { "docker compose" }.yellow()
        );

        match interactive_menu("Settings", &[
            ("d", "Set max search depth"),
            ("e", "Manage excluded directories"),
            ("c", "Toggle docker command (docker compose / docker-compose)"),
            ("r", "Reset to defaults"),
            ("b", "Back"),
        ]) {
            Some('d') => {
                println!();
                if let Ok(input) = inquire::Text::new("Max search depth (leave empty for unlimited):").prompt() {
                    if input.trim().is_empty() {
                        config.max_depth = None;
                        println!("{}", "Max depth set to unlimited".green());
                    } else if let Ok(depth) = input.parse::<usize>() {
                        config.max_depth = Some(depth);
                        println!("{}", format!("Max depth set to {}", depth).green());
                    } else {
                        println!("{}", "Invalid number!".red());
                    }
                }
                if save_config(config).is_ok() {
                    println!("{}", "Configuration saved!".green());
                }
                pause();
            }
            Some('e') => {
                manage_excluded_dirs(config);
            }
            Some('c') => {
                config.legacy_compose = !config.legacy_compose;
                let cmd = if config.legacy_compose { "docker-compose" } else { "docker compose" };
                if save_config(config).is_ok() {
                    println!("{}", format!("Docker command set to: {}", cmd).green());
                }
                pause();
            }
            Some('r') => {
                if Confirm::new("Reset all settings to defaults?")
                    .with_default(false)
                    .prompt()
                    .unwrap_or(false)
                {
                    *config = Config::default();
                    if save_config(config).is_ok() {
                        println!("{}", "Settings reset to defaults!".green());
                    }
                    pause();
                }
            }
            _ => break,
        }
    }
}

fn manage_excluded_dirs(config: &mut Config) {
    loop {
        clear_screen();
        println!("\n{}", "Currently excluded:".bold());
        if config.excluded_dirs.is_empty() {
            println!("  {}", "None".yellow());
        } else {
            for dir in &config.excluded_dirs {
                println!("  - {}", dir.yellow());
            }
        }
        println!("{}", "(always excluded: hidden dirs, target, node_modules, vendor)".bright_black());

        match interactive_menu("Manage Excluded Directories", &[
            ("a", "Add directory"),
            ("r", "Remove directory"),
            ("c", "Clear all"),
            ("b", "Back"),
        ]) {
            Some('a') => {
                println!();
                if let Ok(dir_name) = inquire::Text::new("Directory name to exclude:").prompt() {
                    let dir_name = dir_name.trim().to_string();
                    if !dir_name.is_empty() && !config.excluded_dirs.contains(&dir_name) {
                        config.excluded_dirs.push(dir_name.clone());
                        if save_config(config).is_ok() {
                            println!("{}", format!("'{}' added to exclusions!", dir_name).green());
                        }
                    } else if config.excluded_dirs.contains(&dir_name) {
                        println!("{}", "Directory already excluded!".yellow());
                    }
                    pause();
                }
            }
            Some('r') => {
                if config.excluded_dirs.is_empty() {
                    println!("\n{}", "No excluded directories to remove!".yellow());
                    pause();
                    continue;
                }

                println!();
                if let Ok(selected) = Select::new("Select directory to remove:", config.excluded_dirs.clone()).prompt() {
                    config.excluded_dirs.retain(|d| d != &selected);
                    if save_config(config).is_ok() {
                        println!("{}", format!("'{}' removed from exclusions!", selected).green());
                    }
                    pause();
                }
            }
            Some('c') => {
                if config.excluded_dirs.is_empty() {
                    println!("\n{}", "No excluded directories to clear!".yellow());
                    pause();
                    continue;
                }

                if Confirm::new("Clear all excluded directories?")
                    .with_default(false)
                    .prompt()
                    .unwrap_or(false)
                {
                    config.excluded_dirs.clear();
                    if save_config(config).is_ok() {
                        println!("{}", "All exclusions cleared!".green());
                    }
                    pause();
                }
            }
            _ => break,
        }
    }
}

fn pause() {
    println!("{}", "Press ENTER to continue...".bright_black());
    let _ = std::io::stdin().read_line(&mut String::new());
}
