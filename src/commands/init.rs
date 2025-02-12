use crate::{api, config, jar, java, modpack};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input, Select};

pub async fn init(
    matches: &ArgMatches,
    override_directory: Option<&str>,
    profile_name: Option<&str>,
) -> i32 {
    let directory = if let Some(override_directory) = override_directory {
        &override_directory.to_string()
    } else {
        matches.get_one::<String>("directory").unwrap()
    };

    if std::path::Path::new(&format!("{}/.mcvcli.json", directory)).exists() {
        println!("{} {}", ".mcvcli.json".cyan(), "already exists!".red());
        println!(
            "{} {} {}",
            "use".red(),
            "mcvcli install".cyan(),
            "instead.".red()
        );

        return 1;
    }

    std::fs::create_dir_all(directory).unwrap();

    let jars = std::fs::read_dir(directory)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                let name = path.file_name().unwrap().to_str().unwrap();

                if name.ends_with(".jar") {
                    return Some(name.to_string());
                }
            }

            None
        })
        .collect::<Vec<String>>();

    let server_jarfile = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Server Jar File")
        .default(0)
        .item("Install New (Jar)")
        .item("Install New (Modrinth Modpack)")
        .items(&jars)
        .interact()
        .unwrap();

    let api = api::mcjars::McjarsApi::new();

    match server_jarfile {
        0 => {
            println!("{}", "Getting server types...".bright_black());

            let types = api.types().await.unwrap();

            println!(
                "{} {}",
                "getting server types...".bright_black(),
                "DONE".green().bold()
            );

            let server_type = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Server Jar File")
                .default(0)
                .items(&types.values().map(|t| &t.name).collect::<Vec<&String>>())
                .max_length(10)
                .interact()
                .unwrap();

            let server_type = types.keys().nth(server_type).unwrap();
            println!(
                "{} {} {}",
                "getting server versions for".bright_black(),
                types.get(server_type).unwrap().name.to_string().cyan(),
                "...".bright_black()
            );

            let versions = api.versions(server_type).await.unwrap();

            println!(
                "{} {} {} {}",
                "getting server versions for".bright_black(),
                types.get(server_type).unwrap().name.to_string().cyan(),
                "...".bright_black(),
                "DONE".green().bold()
            );

            let server_version = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Jar Version")
                .default(0)
                .items(&versions.keys().rev().collect::<Vec<&String>>())
                .max_length(10)
                .interact()
                .unwrap();

            let server_version = versions.keys().rev().nth(server_version).unwrap();
            println!(
                "{} {} {}",
                "getting server builds for".bright_black(),
                server_version.to_string().cyan(),
                "...".bright_black()
            );

            let builds = api.builds(server_type, server_version).await.unwrap();

            println!(
                "{} {} {} {}",
                "getting server builds for".bright_black(),
                server_version.to_string().cyan(),
                "...".bright_black(),
                "DONE".green().bold()
            );

            let server_build = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Jar Build")
                .default(0)
                .items(&builds.iter().map(|b| &b.name).collect::<Vec<&String>>())
                .max_length(10)
                .interact()
                .unwrap();

            let server_build = &builds[server_build];

            println!(
                "{} {} {} {}",
                "installing".bright_black(),
                server_version.cyan(),
                server_build.name.cyan(),
                "...".bright_black()
            );

            jar::install(server_build, directory, 1).await.unwrap();

            println!(
                "{} {} {} {} {}",
                "installing".bright_black(),
                server_version.cyan(),
                server_build.name.cyan(),
                "...".bright_black(),
                "DONE".green().bold()
            );

            let ram_mb = Input::<u16>::with_theme(&ColorfulTheme::default())
                .with_prompt("RAM (MB)")
                .default(2048)
                .interact()
                .unwrap();

            let mut config = config::Config::new(&format!("{}/.mcvcli.json", directory), true);
            config.profile_name = profile_name.unwrap_or("default").to_string();
            config.java_version = versions.get(server_version).unwrap().java;
            config.ram_mb = ram_mb;
            config.save();
        }
        1 => {
            let modrinth_api = api::modrinth::ModrinthApi::new();
            let mut projects = modrinth_api
                .projects(
                    "",
                    "[[\"project_type:modpack\"],[\"server_side != unsupported\"]]",
                )
                .await
                .unwrap();
            let mut _project = 0;

            loop {
                let modpack = FuzzySelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Modpack?")
                    .default(0)
                    .item("Search")
                    .items(
                        &projects
                            .iter()
                            .map(|p| {
                                format!(
                                    "{:17} {}",
                                    format!(
                                        "{} - {}",
                                        p.versions.first().unwrap(),
                                        p.versions.last().unwrap()
                                    ),
                                    p.title
                                )
                            })
                            .collect::<Vec<String>>(),
                    )
                    .max_length(10)
                    .interact()
                    .unwrap();

                _project = modpack;

                if modpack == 0 {
                    let search = Input::<String>::new()
                        .with_prompt("Search")
                        .interact()
                        .unwrap();

                    projects = modrinth_api
                        .projects(
                            &search,
                            "[[\"project_type:modpack\"],[\"server_side != unsupported\"]]",
                        )
                        .await
                        .unwrap();
                } else {
                    break;
                }
            }

            let project = &projects[_project - 1];

            println!();
            println!(
                "{} {} {}",
                "getting versions for".bright_black(),
                project.title.cyan(),
                "...".bright_black()
            );

            let versions = modrinth_api
                .versions(project.project_id.as_ref().unwrap())
                .await
                .unwrap();
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
                project.project_id.as_ref().unwrap().cyan()
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
                    &versions
                        .iter()
                        .map(|v| {
                            format!(
                                "{:8} {}",
                                v.game_versions.first().unwrap(),
                                v.name
                                    .as_ref()
                                    .unwrap_or(v.version_number.as_ref().unwrap())
                            )
                        })
                        .collect::<Vec<String>>(),
                )
                .max_length(5)
                .interact()
                .unwrap();

            let modpack_version = &versions[modpack_version];

            let ram_mb = Input::<u16>::with_theme(&ColorfulTheme::default())
                .with_prompt("RAM (MB)")
                .default(2048)
                .interact()
                .unwrap();

            println!();
            println!(
                "{} {} {}",
                "installing".bright_black(),
                project.title.cyan(),
                "...".bright_black()
            );

            modpack::install(directory, &api, modpack_version).await;

            let mut config = config::Config::new(&format!("{}/.mcvcli.json", directory), true);
            config.profile_name = profile_name.unwrap_or("default").to_string();
            config.ram_mb = ram_mb;
            config.jar_file = "server.jar".to_string();
            config.modpack_slug = Some(project.project_id.clone().unwrap());
            config.modpack_version = Some(modpack_version.id.clone());
            let detected = jar::detect(".".to_string(), &config).await;

            if detected.is_some() {
                let ([build, _], versions, _) = detected.unwrap();

                config.java_version = versions
                    .get(
                        &build
                            .version_id
                            .unwrap_or(build.project_version_id.unwrap_or("unknown".to_string())),
                    )
                    .unwrap_or(versions.last().unwrap().1)
                    .java;
            } else {
                config.java_version = 21;
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
        _ => {
            let jar_file = jars[server_jarfile - 2].clone();
            let java = java::Java::new();

            println!("{}", "getting java versions...".bright_black());

            let java_versions = java.versions().await;

            println!(
                "{} {}",
                "getting java versions...".bright_black(),
                "DONE".green().bold()
            );

            let java_version = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Java Version")
                .default(0)
                .items(
                    &java_versions
                        .iter()
                        .rev()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>(),
                )
                .max_length(10)
                .interact()
                .unwrap();

            let java_version = *java_versions.iter().rev().nth(java_version).unwrap();

            let ram_mb = Input::<u16>::with_theme(&ColorfulTheme::default())
                .with_prompt("RAM (MB)")
                .default(2048)
                .interact()
                .unwrap();

            let mut config = config::Config::new(&format!("{}/.mcvcli.json", directory), true);
            config.profile_name = profile_name.unwrap_or("default").to_string();
            config.jar_file = jar_file;
            config.java_version = java_version;
            config.ram_mb = ram_mb;
            config.save();
        }
    }

    0
}
