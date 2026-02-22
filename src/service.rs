use crate::config::Config;
use inquire::Select;
use std::{fs, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Service {
    pub name: String,
    pub path: PathBuf,
    pub compose_file: String,
}

pub fn find_services(config: &Config) -> Vec<Service> {
    let current_dir = std::env::current_dir().unwrap();
    let mut services = Vec::new();
    let mut visited = std::collections::HashSet::new();

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

    if let Some(max_depth) = config.max_depth {
        if depth >= max_depth {
            return;
        }
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || matches!(name, "target" | "node_modules" | "vendor") {
                    continue;
                }
                if config.excluded_dirs.contains(&name.to_string()) {
                    continue;
                }
            }

            if path.is_dir() {
                let compose_files = get_compose_files(&path);

                if !compose_files.is_empty() {
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

    match Select::new("Multiple compose files found. Which one to use?", files.to_vec()).prompt() {
        Ok(selected) => Some(selected),
        Err(_) => None,
    }
}
