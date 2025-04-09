use crate::{api, config, jar};

use clap::ArgMatches;
use colored::Colorize;
use std::path::Path;

pub async fn list(_matches: &ArgMatches) -> i32 {
    let config = config::Config::new(".mcvcli.json", false);

    if !Path::new("mods").exists() {
        println!("{}", "no mods folder found.".red());
        return 1;
    }

    println!("{}", "checking installed version ...".bright_black());

    let detected = jar::detect(".", &config).await;

    println!(
        "{} {}",
        "checking installed version ...".bright_black(),
        "DONE".green().bold()
    );

    if detected.is_none() {
        println!("{}", "installed version could not be detected.".red());
        return 1;
    }

    println!("{}", "listing mods...".bright_black());

    let [build, _] = detected.unwrap().0;
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
    .await
    .unwrap();

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
        println!(
            "  {} {} {}",
            "version:    ".bright_black(),
            project
                .installed_version
                .as_ref()
                .unwrap()
                .name
                .as_ref()
                .unwrap_or(
                    project
                        .installed_version
                        .as_ref()
                        .unwrap()
                        .version_number
                        .as_ref()
                        .unwrap()
                )
                .cyan(),
            if project.installed_latest_version.is_none() || project.installed_version.is_none() {
                "(unknown)".yellow()
            } else if project.installed_latest_version.as_ref().unwrap().id
                == project.installed_version.as_ref().unwrap().id
            {
                "(latest)".green()
            } else {
                "(outdated)".red()
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
                project.installed_latest_version.is_some()
                    && project.installed_version.is_some()
                    && project.installed_version.as_ref().unwrap().id
                        != project.installed_latest_version.as_ref().unwrap().id
            })
            .count()
            .to_string()
            .cyan()
    );

    0
}
