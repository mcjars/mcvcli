use crate::{config, jar, profiles};

use clap::ArgMatches;
use colored::Colorize;
use std::path::Path;

pub async fn list(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let include_version = matches
        .get_one::<bool>("include_version")
        .expect("has default value");
    let config = config::Config::new(".mcvcli.json", false);

    println!("{}", "listing profiles...".bright_black());

    let mut list = profiles::list();
    list.push(config.profile_name.clone());

    let mut futures = Vec::new();

    if *include_version {
        for profile in list.iter() {
            let directory = if *profile != config.profile_name {
                format!(".mcvcli.profiles/{profile}")
            } else {
                String::from(".")
            };

            futures.push(async move {
                let profile_config = config::Config::new(
                    &Path::new(&directory).join(".mcvcli.json").to_string_lossy(),
                    false,
                );

                jar::detect(&directory, &profile_config).await
            });
        }
    }

    let results = futures::future::join_all(futures).await;

    println!(
        "{} {}",
        "listing profiles...".bright_black(),
        "DONE".green().bold()
    );

    for (i, profile) in list.into_iter().enumerate() {
        println!();

        let directory = if *profile != config.profile_name {
            format!(".mcvcli.profiles/{profile}")
        } else {
            String::from(".")
        };

        let profile_config = config::Config::new(
            &Path::new(&directory).join(".mcvcli.json").to_string_lossy(),
            false,
        );

        println!(
            "{} {}",
            profile.cyan().bold().underline(),
            if *profile == config.profile_name {
                "(current)".green()
            } else {
                String::new().green()
            }
        );

        println!(
            "  {} {}",
            "jar file:    ".bright_black(),
            profile_config.jar_file.cyan()
        );
        println!(
            "  {} {}",
            "java version:".bright_black(),
            profile_config.java_version.to_string().cyan()
        );
        println!(
            "  {} {}",
            "ram (mb):    ".bright_black(),
            profile_config.ram_mb.to_string().cyan()
        );

        if *include_version {
            let detected = results
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("missing profile detection result"))?
                .as_ref();

            if let Some(([build, latest], versions, modpack)) = detected {
                println!("  {}", "version:".bright_black());
                println!("    {} {}", "type:   ".bright_black(), build.r#type.cyan());
                println!(
                    "    {} {} {}",
                    "version:".bright_black(),
                    build
                        .version_id
                        .as_ref()
                        .unwrap_or(
                            build
                                .project_version_id
                                .as_ref()
                                .unwrap_or(&"unknown".to_string())
                        )
                        .cyan(),
                    if jar::is_latest_version(build, versions) {
                        "(latest)".green()
                    } else {
                        "(outdated)".red()
                    }
                );
                println!(
                    "    {} {} {}",
                    "build:  ".bright_black(),
                    build.name.cyan(),
                    if build.uuid == latest.uuid {
                        "(latest)".green()
                    } else {
                        "(outdated)".red()
                    }
                );

                if let Some(modpack) = modpack {
                    println!("  {}", "installed modpack:".bright_black());
                    println!(
                        "    {} {}",
                        "name:       ".bright_black(),
                        modpack.title.cyan()
                    );
                    println!(
                        "    {} {}",
                        "description:".bright_black(),
                        modpack.description.cyan()
                    );
                    println!(
                        "    {} {}",
                        "project id: ".bright_black(),
                        modpack.id.as_deref().unwrap_or("unknown").cyan()
                    );
                    if let Some(version) = config.modpack_version.as_ref() {
                        println!(
                            "    {} {} {}",
                            "version id: ".bright_black(),
                            version.cyan(),
                            if modpack.versions.last() == Some(version) {
                                "(latest)".green()
                            } else {
                                "(outdated)".red()
                            }
                        );
                    }
                    println!(
                        "    {} {}",
                        "downloads:  ".bright_black(),
                        modpack.downloads.to_string().cyan()
                    );
                    println!();
                }
            } else {
                println!("  {} {}", "version:     ".bright_black(), "unknown".cyan());
            }
        }
    }

    Ok(0)
}
