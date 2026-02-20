use colored::*;
use inquire::{Confirm, MultiSelect, Select};
use std::process::Command;
use std::{fs, path::PathBuf};

#[derive(Clone, Debug)]
struct Service {
    name: String,
    path: PathBuf,
}

fn main() {
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
                "Stop" => stop_all_services(&services),
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

    if let Ok(entries) = fs::read_dir(&current_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let compose_path = path.join("docker-compose.yml");
                if compose_path.exists() {
                    if let Some(service_name) = path.file_name().and_then(|n| n.to_str()) {
                        services.push(Service {
                            name: service_name.to_string(),
                            path,
                        });
                    }
                }
            }
        }
    }

    services.sort_by(|a, b| a.name.cmp(&b.name));
    services
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
        println!("{}\n", "No services selected".red());
        pause();
        return;
    }

    println!("\n{}\n", "Starting services...".yellow().bold());

    let mut success_count = 0;

    for service in &selected {
        print!("  {} ... ", service.name.cyan());

        if run_docker_compose(&service.path, &["up", "-d"]) {
            println!("{}", "OK".green());
            success_count += 1;
        } else {
            println!("{}", "ERROR".red());
        }
    }

    println!("\n{}\n", format!("{}/{} services started", success_count, selected.len()).green());
    pause();
}

fn stop_all_services(services: &[Service]) {
    if Confirm::new("Stop all services?")
        .with_default(false)
        .prompt()
        .unwrap_or(false)
    {
        println!("\n{}\n", "Stopping services...".yellow());

        let mut success_count = 0;

        for service in services {
            print!("  {} ... ", service.name.cyan());
            if run_docker_compose(&service.path, &["down"]) {
                println!("{}", "OK".green());
                success_count += 1;
            } else {
                println!("{}", "ERROR".red());
            }
        }

        println!("\n{}\n", format!("{}/{} services stopped", success_count, services.len()).green());
    }

    pause();
}

fn restart_services(services: &[Service]) {
    let selected = select_services(services);

    if selected.is_empty() {
        println!("{}\n", "No services selected".red());
        pause();
        return;
    }

    println!("\n{}\n", "Restarting services...".yellow());

    let mut success_count = 0;

    for service in &selected {
        print!("  {} ... ", service.name.cyan());
        if run_docker_compose(&service.path, &["restart"]) {
            println!("{}", "OK".green());
            success_count += 1;
        } else {
            println!("{}", "ERROR".red());
        }
    }

    println!("\n{}\n", format!("{}/{} services restarted", success_count, selected.len()).green());
    pause();
}

fn show_status(services: &[Service]) {
    println!("\n{}\n", "Services Status:".bold().cyan());

    for service in services {
        println!("{}", format!("{}", service.name).cyan().bold());
        run_docker_compose(&service.path, &["ps"]);
        println!();
    }

    pause();
}

fn show_logs(services: &[Service]) {
    let service_names: Vec<&str> = services
        .iter()
        .map(|s| s.name.as_str())
        .chain(std::iter::once("All"))
        .collect();

    if let Ok(selected) = Select::new("Which service?", service_names).prompt() {
        println!("\n{}\n", "Streaming logs (Ctrl+C to stop)...".yellow());

        if selected == "All" {
            for service in services {
                run_docker_compose(&service.path, &["logs", "-f"]);
            }
        } else {
            if let Some(service) = services.iter().find(|s| s.name == selected) {
                run_docker_compose(&service.path, &["logs", "-f"]);
            }
        }
    }

    println!();
    pause();
}

fn cleanup_data(services: &[Service]) {
    let selected = select_services(services);

    if selected.is_empty() {
        println!("{}\n", "No services selected".red());
        pause();
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
            if run_docker_compose(&service.path, &["down", "-v"]) {
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

fn run_docker_compose(path: &PathBuf, args: &[&str]) -> bool {
    let mut cmd = Command::new("docker-compose");
    cmd.current_dir(path).args(args);

    match cmd.status() {
        Ok(status) => status.success(),
        Err(_) => {
            // Try newer docker compose syntax
            let mut cmd = Command::new("docker");
            cmd.current_dir(path)
                .arg("compose")
                .args(args);

            cmd.status().map(|s| s.success()).unwrap_or(false)
        }
    }
}

fn pause() {
    println!("{}", "Press ENTER to continue...".bright_black());
    let _ = std::io::stdin().read_line(&mut String::new());
}
