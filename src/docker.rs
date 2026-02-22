use crate::service::Service;
use std::process::Command;
use std::thread;

pub fn build_compose_cmd(service: &Service, legacy: bool) -> Command {
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

pub fn run_docker_compose(service: &Service, args: &[&str], legacy: bool) -> bool {
    build_compose_cmd(service, legacy)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn get_service_status(service: &Service, legacy: bool) -> bool {
    let output = build_compose_cmd(service, legacy)
        .args(&["ps", "-q"])
        .output();
    match output {
        Ok(out) => !out.stdout.is_empty(),
        Err(_) => false,
    }
}

pub fn check_all_statuses(services: &[Service], legacy: bool) -> Vec<(String, bool)> {
    let handles: Vec<_> = services
        .iter()
        .map(|service| {
            let service = service.clone();
            thread::spawn(move || {
                let up = get_service_status(&service, legacy);
                (service.name, up)
            })
        })
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().unwrap_or_else(|_| ("?".to_string(), false)))
        .collect()
}

pub fn run_parallel(services: Vec<Service>, args: &[&str], legacy: bool) -> Vec<(String, bool)> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    let handles: Vec<_> = services
        .into_iter()
        .map(|service| {
            let args = args.clone();
            thread::spawn(move || {
                let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let ok = run_docker_compose(&service, &args_ref, legacy);
                (service.name, ok)
            })
        })
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().unwrap_or_else(|_| ("?".to_string(), false)))
        .collect()
}
