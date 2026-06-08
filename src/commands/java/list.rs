use crate::{config, java};

use clap::ArgMatches;
use colored::Colorize;
use human_bytes::human_bytes;

fn recursive_size(path: &str) -> u64 {
    let mut size = 0;

    let Ok(entries) = std::fs::read_dir(path) else {
        return size;
    };

    for file in entries.flatten() {
        let Ok(metadata) = file.metadata() else {
            continue;
        };

        if metadata.is_dir() {
            if let Some(path) = file.path().to_str() {
                size += recursive_size(path);
            }
        } else {
            size += metadata.len();
        }
    }

    size
}

pub async fn list(_matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let config = config::Config::new_optional(".mcvcli.json");

    println!("{}", "listing java versions...".bright_black());

    let local = java::find_local();
    let mut list = java::installed();
    list.sort();

    let mut versions: Vec<(String, u64)> = Vec::with_capacity(list.len());
    for (_, path) in list.iter() {
        let version = std::process::Command::new(format!("{path}/bin/java"))
            .arg("-version")
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .next()
                    .unwrap_or("unknown")
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

    for (i, (version, path)) in list.into_iter().enumerate() {
        println!();

        println!(
            "{} {}",
            format!("java {version}").cyan().bold().underline(),
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

        let (version, size) = versions
            .get(i)
            .ok_or_else(|| anyhow::anyhow!("missing version info"))?;

        println!("  {} {}", "path:   ".bright_black(), path.cyan());
        println!("  {} {}", "version:".bright_black(), version.cyan());
        println!(
            "  {} {}",
            "size:   ".bright_black(),
            human_bytes(*size as f64).cyan()
        );
    }

    if let Some((version, path, root)) = local {
        println!();

        println!(
            "{} {}",
            format!("java {version}").cyan().bold().underline(),
            "(local)".green()
        );

        let version = std::process::Command::new(&path)
            .arg("-version")
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stderr)
                    .lines()
                    .next()
                    .unwrap_or("unknown")
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

    Ok(0)
}
