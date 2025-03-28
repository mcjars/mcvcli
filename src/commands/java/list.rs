use crate::{config, java};

use clap::ArgMatches;
use colored::Colorize;
use human_bytes::human_bytes;

fn recursive_size(path: &str) -> u64 {
    let mut size = 0;

    for file in std::fs::read_dir(path).unwrap() {
        let file = file.unwrap();
        let metadata = file.metadata().unwrap();

        if metadata.is_dir() {
            size += recursive_size(file.path().to_str().unwrap());
        } else {
            size += metadata.len();
        }
    }

    size
}

pub async fn list(_matches: &ArgMatches) -> i32 {
    let config = config::Config::new_optional(".mcvcli.json");

    println!("{}", "listing java versions...".bright_black());

    let java = java::Java::new();
    let local = java.find_local();
    let mut list = java.installed();
    list.sort();

    let mut versions: Vec<(String, u64)> = Vec::with_capacity(list.len());
    for (_, path) in &list {
        let version = std::process::Command::new(format!("{}/bin/java", path))
            .arg("-version")
            .output()
            .map(|output| {
                String::from_utf8(output.stderr)
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap()
                    .to_string()
            })
            .unwrap_or("unknown".to_string());

        versions.push((version, recursive_size(path)));
    }

    println!(
        "{} {}",
        "listing java versions...".bright_black(),
        "DONE".green().bold()
    );

    for (version, path) in list {
        println!();

        println!(
            "{} {}",
            format!("java {}", version).cyan().bold().underline(),
            if version
                == config
                    .as_ref()
                    .map(|config| config.java_version)
                    .unwrap_or(0)
            {
                "(current)".green()
            } else {
                String::new().green()
            }
        );

        let (version, size) = versions.remove(0);

        println!("  {} {}", "path:   ".bright_black(), path.cyan());
        println!("  {} {}", "version:".bright_black(), version.cyan());
        println!(
            "  {} {}",
            "size:   ".bright_black(),
            human_bytes(size as f64).cyan()
        );
    }

    if let Some((version, path, root)) = local {
        println!();

        println!(
            "{} {}",
            format!("java {}", version).cyan().bold().underline(),
            "(local)".green()
        );

        let version = std::process::Command::new(&path)
            .arg("-version")
            .output()
            .map(|output| {
                String::from_utf8(output.stderr)
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap()
                    .to_string()
            })
            .unwrap_or("unknown".to_string());

        println!("  {} {}", "path:   ".bright_black(), path.cyan());
        println!("  {} {}", "version:".bright_black(), version.cyan());
        println!(
            "  {} {}",
            "size:   ".bright_black(),
            if !root.is_empty() {
                human_bytes(recursive_size(&root) as f64).cyan()
            } else {
                "unknown".cyan()
            }
        );
    }

    0
}
