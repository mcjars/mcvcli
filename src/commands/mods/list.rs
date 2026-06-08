use crate::{api, config, jar};

use clap::ArgMatches;
use colored::Colorize;
use std::path::Path;

pub async fn list(_matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let config = config::Config::new(".mcvcli.json", false);

    if !Path::new("mods").exists() {
        println!("{}", "no mods folder found.".red());
        return Ok(1);
    }

    println!("{}", "checking installed version ...".bright_black());

    let detected = jar::detect(".", &config).await;

    println!(
        "{} {}",
        "checking installed version ...".bright_black(),
        "DONE".green().bold()
    );

    let Some(detected) = detected else {
        println!("{}", "installed version could not be detected.".red());
        return Ok(1);
    };

    println!("{}", "listing mods...".bright_black());

    let [build, _] = detected.0;
    let list = api::modrinth::lookup(
        "mods",
        Some(&build.r#type.to_lowercase()),
        Some(
            build.version_id.as_ref().unwrap_or(
                build
                    .project_version_id
                    .as_ref()
                    .unwrap_or(&"unknown".to_string()),
            ),
        ),
    )
    .await?;

    println!(
        "{} {}",
        "listing mods...".bright_black(),
        "DONE".green().bold()
    );

    for (path, project) in list.iter() {
        println!();
        println!("{}", project.title.cyan().bold().underline());

        println!(
            "  {} {}",
            "description:".bright_black(),
            project.description.cyan()
        );
        println!(
            "  {} {}",
            "path:       ".bright_black(),
            path.display().to_string().cyan()
        );
        println!(
            "  {} {}",
            "downloads:  ".bright_black(),
            project.downloads.to_string().cyan()
        );
        let version_name = project
            .installed_version
            .as_ref()
            .and_then(|version| version.name.as_ref().or(version.version_number.as_ref()))
            .map(|name| name.as_str())
            .unwrap_or("unknown");

        println!(
            "  {} {} {}",
            "version:    ".bright_black(),
            version_name.cyan(),
            if let Some(installed) = &project.installed_version
                && let Some(latest) = &project.installed_latest_version
            {
                if installed.id == latest.id {
                    "(latest)".green()
                } else {
                    "(outdated)".red()
                }
            } else {
                "(unknown)".yellow()
            }
        );
    }

    println!();
    println!("{}", "summary:".bright_black());
    println!(
        "  {} {}",
        "total mods:".bright_black(),
        list.len().to_string().cyan()
    );
    println!(
        "  {} {}",
        "outdated:  ".bright_black(),
        list.values()
            .filter(|project| {
                match (
                    &project.installed_version,
                    &project.installed_latest_version,
                ) {
                    (Some(installed), Some(latest)) => installed.id != latest.id,
                    _ => false,
                }
            })
            .count()
            .to_string()
            .cyan()
    );

    Ok(0)
}
