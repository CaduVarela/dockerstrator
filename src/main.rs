use colored::*;
use inquire::{Confirm, MultiSelect, Select};
use std::process::Command;
use std::{fs, path::PathBuf};

#[derive(Clone, Debug)]
struct Service {
    name: String,
    path: PathBuf,
    compose_file: String,
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

    let services = find_services();

    if services.is_empty() {
        println!("{}", "No docker-compose.yml found in this directory structure.".red());
        return;
    }

    println!("\n{}\n", "Docker Services Orchestrator".bold().cyan());
    println!("Services found: {}\n", services.len());

    loop {
        match show_main_menu() {
            Some(choice) => match choice.as_str() {
                "Start" => start_services(&services),
                "Stop" => stop_services(&services),
                "Restart" => restart_services(&services),
                "Status" => show_status(&services),
                "Logs" => show_logs(&services),
                "Cleanup" => cleanup_data(&services),
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
}

fn find_services() -> Vec<Service> {
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
    visited.insert(current_dir.clone());
    scan_directory(&current_dir, &mut services, &mut visited);
    services.sort_by(|a, b| a.name.cmp(&b.name));
    services
}

fn scan_directory(dir: &PathBuf, services: &mut Vec<Service>, visited: &mut std::collections::HashSet<PathBuf>) {
    if visited.contains(dir) {
        return;
    }
    visited.insert(dir.clone());

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // Skip hidden directories and common non-service directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || matches!(name, "target" | "node_modules") {
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
                    scan_directory(&path, services, visited);
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

fn show_main_menu() -> Option<String> {
    let options = vec![
        "Start",
        "Stop",
        "Restart",
        "Status",
        "Logs",
        "Cleanup",
        "Exit",
    ];

    Select::new("What would you like to do?", options)
        .with_formatter(&|i| i.value.to_string())
        .prompt()
        .ok()
        .map(|s| s.to_string())
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

fn start_services(services: &[Service]) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    println!("\n{}\n", "Starting services...".yellow().bold());

    let mut success_count = 0;

    for service in &selected {
        print!("  {} ... ", service.name.cyan());

        if run_docker_compose(service, &["up", "-d"]) {
            println!("{}", "OK".green());
            success_count += 1;
        } else {
            println!("{}", "ERROR".red());
        }
    }

    println!("\n{}\n", format!("{}/{} services started", success_count, selected.len()).green());
}

fn stop_services(services: &[Service]) {
    let selected = select_services(services);

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
            if run_docker_compose(service, &["down"]) {
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

fn restart_services(services: &[Service]) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    println!("\n{}\n", "Restarting services...".yellow());

    let mut success_count = 0;

    for service in &selected {
        print!("  {} ... ", service.name.cyan());
        if run_docker_compose(service, &["restart"]) {
            println!("{}", "OK".green());
            success_count += 1;
        } else {
            println!("{}", "ERROR".red());
        }
    }

    println!("\n{}\n", format!("{}/{} services restarted", success_count, selected.len()).green());
}

fn show_status(services: &[Service]) {
    println!("\n{}\n", "Services Status:".bold().cyan());

    for service in services {
        let status = get_service_status(service);
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

fn get_service_status(service: &Service) -> bool {
    // Try with docker-compose first
    let mut cmd = Command::new("docker-compose");
    cmd.current_dir(&service.path)
        .arg("-f")
        .arg(&service.compose_file)
        .args(&["ps", "-q"]);

    match cmd.output() {
        Ok(output) => !output.stdout.is_empty(),
        Err(_) => {
            // Fallback to docker compose
            let mut cmd = Command::new("docker");
            cmd.current_dir(&service.path)
                .arg("compose")
                .arg("-f")
                .arg(&service.compose_file)
                .args(&["ps", "-q"]);

            match cmd.output() {
                Ok(output) => !output.stdout.is_empty(),
                Err(_) => false,
            }
        }
    }
}

fn show_logs(services: &[Service]) {
    let service_names: Vec<&str> = services
        .iter()
        .map(|s| s.name.as_str())
        .chain(std::iter::once("All"))
        .collect();

    if let Ok(selected) = Select::new("Which service?", service_names).prompt() {
        println!("\n{}\n", "Streaming logs (Ctrl+C to return to menu)...".yellow());

        if selected == "All" {
            for service in services {
                stream_logs(service);
            }
        } else {
            if let Some(service) = services.iter().find(|s| s.name == selected) {
                stream_logs(service);
            }
        }
    }
}

fn stream_logs(service: &Service) {
    let mut cmd = Command::new("docker-compose");
    cmd.current_dir(&service.path)
        .arg("-f")
        .arg(&service.compose_file)
        .args(&["logs", "-f"]);

    // Try spawning docker-compose
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => {
            // Fallback to docker compose
            let mut cmd = Command::new("docker");
            cmd.current_dir(&service.path)
                .arg("compose")
                .arg("-f")
                .arg(&service.compose_file)
                .args(&["logs", "-f"]);

            match cmd.spawn() {
                Ok(c) => c,
                Err(_) => return,
            }
        }
    };

    // Ignore Ctrl+C in parent while child is running
    let _ctrlc_guard = ctrlc::set_handler(|| {})
        .map_err(|_| ())
        .ok();

    // Wait for child to finish
    let _ = child.wait();

    // Ctrl+C handler will be restored when _ctrlc_guard is dropped
}

fn cleanup_data(services: &[Service]) {
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
            if run_docker_compose(service, &["down", "-v"]) {
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

fn run_docker_compose(service: &Service, args: &[&str]) -> bool {
    let mut cmd = Command::new("docker-compose");
    cmd.current_dir(&service.path)
        .arg("-f")
        .arg(&service.compose_file)
        .args(args);

    match cmd.status() {
        Ok(status) => status.success(),
        Err(_) => {
            // Try newer docker compose syntax
            let mut cmd = Command::new("docker");
            cmd.current_dir(&service.path)
                .arg("compose")
                .arg("-f")
                .arg(&service.compose_file)
                .args(args);

            cmd.status().map(|s| s.success()).unwrap_or(false)
        }
    }
}

fn pause() {
    println!("{}", "Press ENTER to continue...".bright_black());
    let _ = std::io::stdin().read_line(&mut String::new());
}
