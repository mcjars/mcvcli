use crate::{config, jar, profiles};

use clap::ArgMatches;
use colored::Colorize;
use std::path::Path;

pub async fn list(matches: &ArgMatches) -> i32 {
    let include_version = matches.get_one::<bool>("include_version").unwrap();
    let config = config::Config::new(".mcvcli.json", false);

    println!("{}", "listing profiles...".bright_black());

    let mut list = profiles::list();
    list.push(config.profile_name.clone());

    let mut futures = Vec::new();

    if *include_version {
        for profile in &list {
            let directory = if *profile != config.profile_name {
                format!(".mcvcli.profiles/{}", profile)
            } else {
                String::from(".")
            };

            futures.push(async move {
                let profile_config = config::Config::new(
                    Path::new(&directory).join(".mcvcli.json").to_str().unwrap(),
                    false,
                );

                jar::detect(directory, &profile_config).await
            });
        }
    }

    let results = futures::future::join_all(futures).await;

    println!(
        "{} {}",
        "listing profiles...".bright_black(),
        "DONE".green().bold()
    );

    for profile in &list {
        println!();

        let directory = if *profile != config.profile_name {
            format!(".mcvcli.profiles/{}", profile)
        } else {
            String::from(".")
        };

        let profile_config = config::Config::new(
            Path::new(&directory).join(".mcvcli.json").to_str().unwrap(),
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
                .get(list.iter().position(|x| x == profile).unwrap())
                .unwrap()
                .as_ref();

            if detected.is_some() {
                let ([build, latest], versions, modpack) = detected.unwrap();

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
                    if versions.keys().next_back().unwrap_or(&String::new())
                        == build.version_id.as_ref().unwrap_or(
                            build
                                .project_version_id
                                .as_ref()
                                .unwrap_or(&"unknown".to_string())
                        )
                    {
                        "(latest)".green()
                    } else {
                        "(outdated)".red()
                    }
                );
                println!(
                    "    {} {} {}",
                    "build:  ".bright_black(),
                    build.name.cyan(),
                    if build.id == latest.id {
                        "(latest)".green()
                    } else {
                        "(outdated)".red()
                    }
                );

                if modpack.is_some() {
                    let modpack = modpack.as_ref().unwrap();

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
                        modpack.id.as_ref().unwrap().cyan()
                    );
                    println!(
                        "    {} {} {}",
                        "version id: ".bright_black(),
                        config.modpack_version.as_ref().unwrap().cyan(),
                        if modpack.versions.last().unwrap()
                            == config.modpack_version.as_ref().unwrap()
                        {
                            "(latest)".green()
                        } else {
                            "(outdated)".red()
                        }
                    );
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

    0
}
