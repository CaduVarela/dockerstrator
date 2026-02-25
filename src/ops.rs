use crate::config::{save_config, Config};
use crate::docker::{build_compose_cmd, check_all_statuses, run_parallel};
use crate::service::Service;
use crate::ui::{clear_screen, interactive_menu, pause};
use colored::*;
use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use inquire::{MultiSelect, Select};
use std::io::{BufRead, BufReader, Write};
use std::thread;

pub fn print_header(service_count: usize) {
    println!("\n{}\n", "Docker Services Orchestrator".bold().cyan());
    println!("Services found: {}\n", service_count);
}

pub fn show_main_menu() -> Option<String> {
    match interactive_menu(
        "Select an action:",
        &[
            ("s", "Start services"),
            ("p", "Stop services"),
            ("r", "Restart services"),
            ("t", "Show status"),
            ("l", "Stream logs"),
            ("c", "Cleanup volumes"),
            ("g", "Settings"),
            ("q", "Exit"),
        ],
    ) {
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

pub fn handle_action(action: &str, services: &[Service], config: &mut Config) {
    match action {
        "Start" => start_services(services, config),
        "Stop" => stop_services(services, config),
        "Restart" => restart_services(services, config),
        "Status" => show_status(services, config),
        "Logs" => show_logs(services, config),
        "Cleanup" => cleanup_data(services, config),
        "Settings" => show_settings(config),
        _ => {}
    }
}

fn print_result(name: &str, ok: bool) {
    if ok {
        println!("  {} ... {}", name.cyan(), "OK".green());
    } else {
        println!("  {} ... {}", name.cyan(), "ERROR".red());
    }
}

fn confirm(question: &str) -> bool {
    matches!(
        interactive_menu(question, &[("y", "Yes"), ("n", "No")]),
        Some('y')
    )
}

fn save_with_feedback(config: &Config) {
    match save_config(config) {
        Ok(_) => println!("{}", "Saved.".green()),
        Err(e) => println!("{}", format!("Error saving config: {}", e).red()),
    }
}

fn select_services(services: &[Service]) -> Vec<Service> {
    let max_name = services.iter().map(|s| s.name.len()).max().unwrap_or(0);

    let labels: Vec<String> = services
        .iter()
        .map(|s| match &s.variant {
            None => s.name.clone(),
            Some(v) => format!("{:<width$}  {}", s.name, v.bright_black(), width = max_name),
        })
        .collect();

    let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

    match MultiSelect::new("Select services:", label_refs.clone())
        .with_help_message("↑↓ navigate  SPACE select  ENTER confirm  ESC cancel")
        .with_formatter(&|items| {
            if items.is_empty() {
                "None selected".to_string()
            } else {
                format!("{} selected", items.len())
            }
        })
        .prompt()
    {
        Ok(selected) => {
            let selected_set: std::collections::HashSet<&str> =
                selected.iter().copied().collect();
            labels
                .iter()
                .enumerate()
                .filter(|(_, label)| selected_set.contains(label.as_str()))
                .map(|(i, _)| services[i].clone())
                .collect()
        }
        Err(_) => Vec::new(),
    }
}

fn start_services(services: &[Service], config: &Config) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    println!("\n{}\n", "Starting services...".yellow().bold());
    let results = run_parallel(selected, &["up", "-d"], config.legacy_compose);
    for (name, ok) in &results {
        print_result(name, *ok);
    }
    let ok_count = results.iter().filter(|(_, ok)| *ok).count();
    println!("\n{}\n", format!("{}/{} services started", ok_count, results.len()).green());
    pause();
}

fn stop_services(services: &[Service], config: &Config) {
    print!("{}", "Checking service status...".bright_black());
    let _ = std::io::stdout().flush();
    let statuses = check_all_statuses(services, config.legacy_compose);
    let running: Vec<Service> = services
        .iter()
        .filter(|s| statuses.iter().any(|(name, up)| name == &s.label() && *up))
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

    let services_list = selected
        .iter()
        .map(|s| format!("  - {}", s.name))
        .collect::<Vec<_>>()
        .join("\n");
    let question = format!(
        "Services to stop:\n{}\n\nStop {} service(s)?",
        services_list,
        selected.len()
    );

    if confirm(&question) {
        println!("\n{}\n", "Stopping services...".yellow());
        let results = run_parallel(selected, &["down"], config.legacy_compose);
        for (name, ok) in &results {
            print_result(name, *ok);
        }
        let ok_count = results.iter().filter(|(_, ok)| *ok).count();
        println!("\n{}\n", format!("{}/{} services stopped", ok_count, results.len()).green());
        pause();
    }
}

fn restart_services(services: &[Service], config: &Config) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    println!("\n{}\n", "Restarting services...".yellow().bold());
    let results = run_parallel(selected, &["restart"], config.legacy_compose);
    for (name, ok) in &results {
        print_result(name, *ok);
    }
    let ok_count = results.iter().filter(|(_, ok)| *ok).count();
    println!("\n{}\n", format!("{}/{} services restarted", ok_count, results.len()).green());
}

fn show_status(services: &[Service], config: &Config) {
    clear_screen();
    println!("\n{}\n", "Services Status:".bold().cyan());

    for (name, up) in check_all_statuses(services, config.legacy_compose) {
        let status_text = if up { "UP".green() } else { "DOWN".red() };
        println!("  {}: {}", name.cyan(), status_text);
    }

    println!();
    pause();
}

fn show_logs(services: &[Service], config: &Config) {
    let max_name = services.iter().map(|s| s.name.len()).max().unwrap_or(0);

    let mut labels: Vec<String> = services
        .iter()
        .map(|s| match &s.variant {
            None => s.name.clone(),
            Some(v) => format!("{:<width$}  {}", s.name, v.bright_black(), width = max_name),
        })
        .collect();
    labels.push("All".to_string());

    let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

    if let Ok(selected) = Select::new("Which service?", label_refs).prompt() {
        println!("\n{}\n", "Streaming logs (Ctrl+C to return to menu)...".yellow());

        if selected == "All" {
            for service in services {
                stream_logs(service, config.legacy_compose);
            }
        } else if let Some(idx) = labels.iter().position(|l| l.as_str() == selected) {
            if let Some(service) = services.get(idx) {
                stream_logs(service, config.legacy_compose);
            }
        }
    }
}

fn stream_logs(service: &Service, legacy: bool) {
    let mut child = match build_compose_cmd(service, legacy)
        .args(&["logs", "-f"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };

    let child_stdout = child.stdout.take().unwrap();
    let child_stderr = child.stderr.take().unwrap();

    // Relay stdout — use \r\n so lines display correctly when raw mode is active
    let t1 = thread::spawn(move || {
        let reader = BufReader::new(child_stdout);
        let stdout = std::io::stdout();
        for line in reader.lines().flatten() {
            let mut out = stdout.lock();
            let _ = write!(out, "{}\r\n", line);
        }
    });

    let t2 = thread::spawn(move || {
        let reader = BufReader::new(child_stderr);
        let stderr = std::io::stderr();
        for line in reader.lines().flatten() {
            let mut err = stderr.lock();
            let _ = write!(err, "{}\r\n", line);
        }
    });

    // Watch for Ctrl+C / Esc in raw mode while child runs
    let _ = enable_raw_mode();
    loop {
        if let Ok(true) = poll(std::time::Duration::from_millis(50)) {
            if let Ok(Event::Key(key)) = read() {
                let ctrl_c = key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL);
                if ctrl_c || key.code == KeyCode::Esc {
                    child.kill().ok();
                    break;
                }
            }
        }
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {}
            Err(_) => break,
        }
    }
    let _ = disable_raw_mode();
    let _ = child.wait();
    let _ = t1.join();
    let _ = t2.join();
}

fn cleanup_data(services: &[Service], config: &Config) {
    let selected = select_services(services);

    if selected.is_empty() {
        return;
    }

    if confirm(&format!("Remove volumes from {} service(s)?", selected.len())) {
        println!("\n{}", "Removing volumes...".red());
        let results = run_parallel(selected, &["down", "-v"], config.legacy_compose);
        for (name, ok) in &results {
            print_result(name, *ok);
        }
        let ok_count = results.iter().filter(|(_, ok)| *ok).count();
        println!("\n{}\n", format!("{}/{} volumes removed", ok_count, results.len()).green());
    }

    pause();
}

pub fn show_settings(config: &mut Config) {
    loop {
        clear_screen();
        println!("\n{}", "Current configuration:".bold());
        println!(
            "  Max search depth: {}",
            config
                .max_depth
                .map(|d| d.to_string())
                .unwrap_or_else(|| "Unlimited".to_string())
                .yellow()
        );
        println!(
            "  Excluded dirs: {}",
            if config.excluded_dirs.is_empty() {
                "None".to_string()
            } else {
                config.excluded_dirs.join(", ")
            }
            .yellow()
        );
        println!(
            "  Docker command: {}",
            if config.legacy_compose {
                "docker-compose"
            } else {
                "docker compose"
            }
            .yellow()
        );

        match interactive_menu(
            "Settings",
            &[
                ("d", "Set max search depth"),
                ("e", "Manage excluded directories"),
                ("c", "Toggle docker command (docker compose / docker-compose)"),
                ("r", "Reset to defaults"),
                ("q", "Back"),
            ],
        ) {
            Some('d') => {
                println!();
                if let Ok(input) =
                    inquire::Text::new("Max search depth (leave empty for unlimited):").prompt()
                {
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
                save_with_feedback(config);
                pause();
            }
            Some('e') => {
                manage_excluded_dirs(config);
            }
            Some('c') => {
                config.legacy_compose = !config.legacy_compose;
                let cmd = if config.legacy_compose {
                    "docker-compose"
                } else {
                    "docker compose"
                };
                save_with_feedback(config);
                println!("{}", format!("Docker command set to: {}", cmd).green());
                pause();
            }
            Some('r') => {
                if confirm("Reset all settings to defaults?") {
                    *config = Config::default();
                    save_with_feedback(config);
                    println!("{}", "Settings reset to defaults!".green());
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
        println!(
            "{}",
            "(always excluded: hidden dirs, target, node_modules, vendor)".bright_black()
        );

        match interactive_menu(
            "Manage Excluded Directories",
            &[
                ("a", "Add directory"),
                ("r", "Remove directory"),
                ("c", "Clear all"),
                ("q", "Back"),
            ],
        ) {
            Some('a') => {
                println!();
                if let Ok(dir_name) =
                    inquire::Text::new("Directory name to exclude:").prompt()
                {
                    let dir_name = dir_name.trim().to_string();
                    if !dir_name.is_empty() && !config.excluded_dirs.contains(&dir_name) {
                        config.excluded_dirs.push(dir_name.clone());
                        save_with_feedback(config);
                        println!("{}", format!("'{}' added to exclusions!", dir_name).green());
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
                if let Ok(selected) =
                    Select::new("Select directory to remove:", config.excluded_dirs.clone())
                        .prompt()
                {
                    config.excluded_dirs.retain(|d| d != &selected);
                    save_with_feedback(config);
                    println!(
                        "{}",
                        format!("'{}' removed from exclusions!", selected).green()
                    );
                    pause();
                }
            }
            Some('c') => {
                if config.excluded_dirs.is_empty() {
                    println!("\n{}", "No excluded directories to clear!".yellow());
                    pause();
                    continue;
                }

                if confirm("Clear all excluded directories?") {
                    config.excluded_dirs.clear();
                    save_with_feedback(config);
                    println!("{}", "All exclusions cleared!".green());
                    pause();
                }
            }
            _ => break,
        }
    }
}
