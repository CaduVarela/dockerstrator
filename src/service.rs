use crate::config::Config;
use std::{fs, path::PathBuf};

const BASE_NAMES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
];

const OVERRIDE_NAMES: &[&str] = &[
    "docker-compose.override.yml",
    "docker-compose.override.yaml",
    "compose.override.yml",
    "compose.override.yaml",
];

#[derive(Clone, Debug)]
pub struct Service {
    pub name: String,
    pub path: PathBuf,
    pub compose_files: Vec<String>, // empty = use docker defaults (base + override auto-merge)
    pub variant: Option<String>,    // e.g. Some("prod") for display purposes
}

impl Service {
    /// Clean label for results/status display, e.g. "rabbitmq" or "rabbitmq (prod)"
    pub fn label(&self) -> String {
        match &self.variant {
            None => self.name.clone(),
            Some(v) => format!("{} ({})", self.name, v),
        }
    }
}

pub fn find_services(config: &Config) -> Vec<Service> {
    let current_dir = std::env::current_dir().unwrap();
    let mut services = Vec::new();
    let mut visited = std::collections::HashSet::new();

    let compose_files = get_compose_files(&current_dir);
    if !compose_files.is_empty() {
        services.extend(group_compose_files(&compose_files, "root", &current_dir));
    }

    scan_directory(&current_dir, &mut services, &mut visited, config, 0);
    services.sort_by(|a, b| a.name.cmp(&b.name).then(a.variant.cmp(&b.variant)));
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
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        services.extend(group_compose_files(&compose_files, dir_name, &path));
                    }
                } else {
                    scan_directory(&path, services, visited, config, depth + 1);
                }
            }
        }
    }
}

/// Groups compose files in a directory into Service entries.
///
/// - docker-compose.yml (+ optional override) → one entry, compose_files=[] (docker handles merge)
/// - docker-compose.prod.yml → separate entry with variant="prod"
/// - Single file total (any type) → one entry, no variant label
fn group_compose_files(files: &[String], dir_name: &str, path: &PathBuf) -> Vec<Service> {
    // Single file: no need to differentiate, no variant label
    if files.len() == 1 {
        return vec![Service {
            name: dir_name.to_string(),
            path: path.clone(),
            compose_files: vec![files[0].clone()],
            variant: None,
        }];
    }

    let has_base = files.iter().any(|f| BASE_NAMES.contains(&f.as_str()));
    let variants: Vec<&String> = files
        .iter()
        .filter(|f| !BASE_NAMES.contains(&f.as_str()) && !OVERRIDE_NAMES.contains(&f.as_str()))
        .collect();

    let mut services = Vec::new();

    // Default entry: let docker handle the base + override merge automatically
    if has_base {
        services.push(Service {
            name: dir_name.to_string(),
            path: path.clone(),
            compose_files: vec![],
            variant: None,
        });
    }

    // Each non-base, non-override file becomes its own entry
    for file in &variants {
        services.push(Service {
            name: dir_name.to_string(),
            path: path.clone(),
            compose_files: vec![file.to_string()],
            variant: Some(extract_variant(file)),
        });
    }

    // Edge case: only override files with no base — treat each as explicit entry
    if !has_base && variants.is_empty() {
        for file in files {
            services.push(Service {
                name: dir_name.to_string(),
                path: path.clone(),
                compose_files: vec![file.clone()],
                variant: Some(extract_variant(file)),
            });
        }
    }

    services
}

/// Extracts the variant name from a compose filename.
/// e.g. "docker-compose.prod.yml" → "prod", "compose.staging.yaml" → "staging"
fn extract_variant(filename: &str) -> String {
    let stem = filename
        .strip_suffix(".yaml")
        .or_else(|| filename.strip_suffix(".yml"))
        .unwrap_or(filename);

    stem.strip_prefix("docker-compose.")
        .or_else(|| stem.strip_prefix("compose."))
        .unwrap_or(stem)
        .to_string()
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
