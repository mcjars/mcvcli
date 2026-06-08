use crate::{api, config, detached, jar, modpack};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{FuzzySelect, Input, Select, theme::ColorfulTheme};

fn wipe_directory() -> Result<(), anyhow::Error> {
    println!("{}", "Wiping server directory...".bright_black());

    for entry in std::fs::read_dir(".")?.flatten() {
        let path = entry.path();

        if path
            .file_name()
            .map(|name| name.to_string_lossy().starts_with(".mcvcli"))
            .unwrap_or(false)
        {
            continue;
        }

        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
    }

    println!(
        "{} {}",
        "Wiping server directory...".bright_black(),
        "DONE".green().bold()
    );

    Ok(())
}

pub async fn install(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let mut config = config::Config::new(".mcvcli.json", false);
    let wipe = matches.get_one::<bool>("wipe").expect("required");

    if detached::is_running() {
        println!(
            "{} {}",
            "server is currently running, use".red(),
            "mcvcli stop".cyan()
        );
        return Ok(1);
    }

    let server_jarfile = if let Some(file) = matches.get_one::<String>("file") {
        match file.as_str() {
            "install" => 0,
            "modrinth" => 1,
            _ => {
                println!("{} {}", file.cyan(), "not found".red());
                return Ok(1);
            }
        }
    } else {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Server Jar File")
            .default(0)
            .item("Install New (Jar)")
            .item("Install New (Modrinth Modpack)")
            .interact()?
    };

    match server_jarfile {
        0 => {
            if *wipe {
                wipe_directory()?;
            }

            let java = if let Some(Ok(build_id)) =
                matches.get_one::<String>("build").map(|b| b.parse::<u32>())
            {
                println!(
                    "{} {}",
                    "getting server build...".bright_black(),
                    "...".bright_black()
                );

                if let Ok((server_build, versions)) = api::mcjars::lookup_id(build_id).await {
                    println!(
                        "{} {}",
                        "getting server build...".bright_black(),
                        "DONE".green().bold()
                    );

                    let server_version = server_build
                        .version_id
                        .as_deref()
                        .or(server_build.project_version_id.as_deref())
                        .unwrap_or("unknown");

                    println!(
                        "{} {} {} {}",
                        "installing".bright_black(),
                        server_version.cyan(),
                        server_build.name.cyan(),
                        "...".bright_black()
                    );

                    jar::install(&server_build, ".", 1).await?;

                    println!(
                        "{} {} {} {} {}",
                        "installing".bright_black(),
                        server_version.cyan(),
                        server_build.name.cyan(),
                        "...".bright_black(),
                        "DONE".green().bold()
                    );

                    let fallback = versions
                        .last()
                        .ok_or_else(|| anyhow::anyhow!("no versions available"))?
                        .1;
                    versions.get(server_version).unwrap_or(fallback).java
                } else {
                    println!(
                        "{} {} {}",
                        "server build".red(),
                        build_id.to_string().cyan(),
                        "not found!".red()
                    );
                    return Ok(1);
                }
            } else if let Some(Ok(build_uuid)) = matches
                .get_one::<String>("build")
                .map(|b| uuid::Uuid::parse_str(b))
            {
                println!(
                    "{} {}",
                    "getting server build...".bright_black(),
                    "...".bright_black()
                );

                if let Ok((server_build, versions)) = api::mcjars::lookup_uuid(build_uuid).await {
                    println!(
                        "{} {}",
                        "getting server build...".bright_black(),
                        "DONE".green().bold()
                    );

                    let server_version = server_build
                        .version_id
                        .as_deref()
                        .or(server_build.project_version_id.as_deref())
                        .unwrap_or("unknown");

                    println!(
                        "{} {} {} {}",
                        "installing".bright_black(),
                        server_version.cyan(),
                        server_build.name.cyan(),
                        "...".bright_black()
                    );

                    jar::install(&server_build, ".", 1).await?;

                    println!(
                        "{} {} {} {} {}",
                        "installing".bright_black(),
                        server_version.cyan(),
                        server_build.name.cyan(),
                        "...".bright_black(),
                        "DONE".green().bold()
                    );

                    let fallback = versions
                        .last()
                        .ok_or_else(|| anyhow::anyhow!("no versions available"))?
                        .1;
                    versions.get(server_version).unwrap_or(fallback).java
                } else {
                    println!(
                        "{} {} {}",
                        "server build".red(),
                        build_uuid.to_string().cyan(),
                        "not found!".red()
                    );
                    return Ok(1);
                }
            } else {
                println!("{}", "getting server types...".bright_black());

                let types = api::mcjars::types().await?;

                println!(
                    "{} {}",
                    "getting server types...".bright_black(),
                    "DONE".green().bold()
                );

                let server_type = if let Some(r#type) = matches.get_one::<String>("type") {
                    if !types.contains_key(&r#type.to_uppercase()) {
                        println!(
                            "{} {} {}",
                            "server type".red(),
                            r#type.to_string().cyan(),
                            "not found!".red()
                        );
                        return Ok(1);
                    }

                    &r#type.to_uppercase()
                } else {
                    let server_type = FuzzySelect::with_theme(&ColorfulTheme::default())
                        .with_prompt("Server Jar File")
                        .default(0)
                        .items(types.values().map(|t| &t.name).collect::<Vec<&String>>())
                        .max_length(10)
                        .interact()?;

                    types
                        .keys()
                        .nth(server_type)
                        .ok_or_else(|| anyhow::anyhow!("selected server type not found"))?
                };

                let server_type_name = types
                    .get(server_type)
                    .ok_or_else(|| anyhow::anyhow!("server type {server_type} not found"))?
                    .name
                    .clone();

                println!(
                    "{} {} {}",
                    "getting server versions for".bright_black(),
                    server_type_name.clone().cyan(),
                    "...".bright_black()
                );

                let versions = api::mcjars::versions(server_type).await?;

                println!(
                    "{} {} {} {}",
                    "getting server versions for".bright_black(),
                    server_type_name.cyan(),
                    "...".bright_black(),
                    "DONE".green().bold()
                );

                let server_version = if let Some(version) = matches.get_one::<String>("version") {
                    if !versions.contains_key(version) {
                        println!(
                            "{} {} {}",
                            "server version".red(),
                            version.to_string().cyan(),
                            "not found!".red()
                        );
                        return Ok(1);
                    }

                    version
                } else {
                    let server_version = FuzzySelect::with_theme(&ColorfulTheme::default())
                        .with_prompt("Jar Version")
                        .default(0)
                        .items(versions.keys().rev().collect::<Vec<&String>>())
                        .max_length(10)
                        .interact()?;

                    versions
                        .keys()
                        .rev()
                        .nth(server_version)
                        .ok_or_else(|| anyhow::anyhow!("selected version not found"))?
                };

                println!(
                    "{} {} {}",
                    "getting server builds for".bright_black(),
                    server_version.to_string().cyan(),
                    "...".bright_black()
                );

                let builds = api::mcjars::builds(server_type, server_version).await?;

                println!(
                    "{} {} {} {}",
                    "getting server builds for".bright_black(),
                    server_version.to_string().cyan(),
                    "...".bright_black(),
                    "DONE".green().bold()
                );

                let server_build = if let Some(build) = matches.get_one::<String>("build") {
                    if build.as_str() == "latest" {
                        builds
                            .first()
                            .ok_or_else(|| anyhow::anyhow!("no builds available"))?
                    } else if let Some(build) = builds.iter().find(|b| &b.name == build) {
                        build
                    } else {
                        println!(
                            "{} {} {}",
                            "server build".red(),
                            build.to_string().cyan(),
                            "not found!".red()
                        );
                        return Ok(1);
                    }
                } else {
                    let server_build = FuzzySelect::with_theme(&ColorfulTheme::default())
                        .with_prompt("Jar Build")
                        .default(0)
                        .items(builds.iter().map(|b| &b.name).collect::<Vec<&String>>())
                        .max_length(10)
                        .interact()?;

                    &builds[server_build]
                };

                println!(
                    "{} {} {} {}",
                    "installing".bright_black(),
                    server_version.cyan(),
                    server_build.name.cyan(),
                    "...".bright_black()
                );

                jar::install(server_build, ".", 1).await?;

                println!(
                    "{} {} {} {} {}",
                    "installing".bright_black(),
                    server_version.cyan(),
                    server_build.name.cyan(),
                    "...".bright_black(),
                    "DONE".green().bold()
                );

                versions
                    .get(server_version)
                    .ok_or_else(|| anyhow::anyhow!("version {server_version} not found"))?
                    .java
            };

            config.modpack_slug = None;
            config.modpack_version = None;
            config.java_version = java;
            config.save();
        }
        1 => {
            let mut projects = api::modrinth::projects(
                "",
                "[[\"project_type:modpack\"],[\"server_side != unsupported\"]]",
            )
            .await?;
            let mut project;

            loop {
                let modpack = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Modpack?")
                    .default(0)
                    .item("Search")
                    .items(
                        projects
                            .iter()
                            .map(|p| {
                                format!(
                                    "{:17} {}",
                                    format!(
                                        "{} - {}",
                                        p.versions.first().map(|v| v.as_str()).unwrap_or("?"),
                                        p.versions.last().map(|v| v.as_str()).unwrap_or("?")
                                    ),
                                    p.title
                                )
                            })
                            .collect::<Vec<String>>(),
                    )
                    .max_length(10)
                    .interact()?;

                project = modpack;

                if modpack == 0 {
                    let search = Input::<String>::new().with_prompt("Search").interact()?;

                    projects = api::modrinth::projects(
                        &search,
                        "[[\"project_type:modpack\"],[\"server_side != unsupported\"]]",
                    )
                    .await?;
                } else {
                    break;
                }
            }

            let project = &projects[project - 1];

            println!();
            println!(
                "{} {} {}",
                "getting versions for".bright_black(),
                project.title.cyan(),
                "...".bright_black()
            );

            let project_id = project
                .project_id
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("project has no id"))?;
            let versions = api::modrinth::versions(project_id).await?;
            let versions = versions
                .iter()
                .filter(|v| !v.files.is_empty())
                .filter(|v| v.name.is_some() || v.version_number.is_some())
                .collect::<Vec<&api::modrinth::Version>>();

            println!(
                "{} {} {} {}",
                "getting versions for".bright_black(),
                project.title.cyan(),
                "...".bright_black(),
                "DONE".green().bold()
            );

            println!();
            println!("{}", project.title.cyan().bold());
            println!(
                "  {} {}",
                "description:".bright_black(),
                project.description.cyan()
            );
            println!(
                "  {} {}",
                "project id: ".bright_black(),
                project.project_id.as_deref().unwrap_or("unknown").cyan()
            );
            println!(
                "  {} {}",
                "downloads:  ".bright_black(),
                project.downloads.to_string().cyan()
            );
            println!();

            let modpack_version = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Modpack Version?")
                .default(0)
                .items(
                    versions
                        .iter()
                        .map(|v| {
                            format!(
                                "{:8} {}",
                                v.game_versions.first().map(|g| g.as_str()).unwrap_or("?"),
                                v.name
                                    .as_deref()
                                    .or(v.version_number.as_deref())
                                    .unwrap_or("unknown")
                            )
                        })
                        .collect::<Vec<String>>(),
                )
                .max_length(5)
                .interact()?;

            let modpack_version = &versions[modpack_version];

            println!();

            if *wipe {
                wipe_directory()?;
            }

            println!(
                "{} {} {}",
                "installing".bright_black(),
                project.title.cyan(),
                "...".bright_black()
            );

            modpack::install(".", modpack_version).await?;

            config.jar_file = "server.jar".to_string();
            config.modpack_slug = Some(project_id.clone());
            config.modpack_version = Some(modpack_version.id.clone());
            let detected = jar::detect(".", &config).await;

            if let Some(([build, _], versions, _)) = detected
                && let Some(fallback) = versions.last()
            {
                let key = build
                    .version_id
                    .unwrap_or(build.project_version_id.unwrap_or("unknown".to_string()));
                config.java_version = versions.get(&key).unwrap_or(fallback.1).java;
            }

            config.save();

            println!(
                "{} {} {} {}",
                "installing".bright_black(),
                project.title.cyan(),
                "...".bright_black(),
                "DONE".green().bold()
            );
        }
        _ => unreachable!(),
    }

    Ok(0)
}
