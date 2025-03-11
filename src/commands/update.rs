use crate::{api, config, detached, jar, modpack, profiles};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{FuzzySelect, Select, theme::ColorfulTheme};
use std::collections::HashMap;

pub async fn update(matches: &ArgMatches) -> i32 {
    let profile = matches.get_one::<String>("profile");
    let config = config::Config::new(".mcvcli.json", false);

    if detached::status(config.pid) {
        println!(
            "{} {}",
            "server is currently running, use".red(),
            "mcvcli stop".cyan()
        );
        return 1;
    }

    if profile.is_some() && config.profile_name == *profile.unwrap() {
        println!(
            "{} {} {}",
            "profile".red(),
            profile.unwrap().cyan(),
            "is currently in use!".red()
        );
        return 1;
    }

    if profile.is_some() && !profiles::list().contains(profile.unwrap()) {
        println!(
            "{} {} {}",
            "profile".red(),
            profile.unwrap().cyan(),
            "does not exist!".red()
        );
        return 1;
    }

    let directory = if let Some(profile) = profile {
        format!(".mcvcli.profiles/{}", profile)
    } else {
        ".".to_string()
    };

    println!("{}", "checking installed version ...".bright_black());

    let mut config = config::Config::new(&format!("{}/.mcvcli.json", directory), false);
    let detected = jar::detect(directory.clone(), &config).await;

    if detected.is_none() {
        println!(
            "{} {}",
            "checking installed version ...".bright_black(),
            "FAILED".red().bold()
        );
        return 1;
    }

    println!(
        "{} {}",
        "checking installed version ...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    let ([build, latest], versions, modpack) = detected.unwrap();
    let mut items: Vec<&str> = Vec::new();

    if versions.keys().next_back().unwrap_or(&String::new())
        != build.version_id.as_ref().unwrap_or(
            build
                .project_version_id
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
        )
        && config.modpack_slug.is_none()
    {
        items.push("Update Version");
    }

    if build.id != latest.id {
        items.push("Update Build");
    }

    let mut modpack_versions = Vec::new();
    if modpack.is_some() {
        let modrinth_api = api::modrinth::ModrinthApi::new();
        modpack_versions = modrinth_api
            .versions(config.modpack_slug.as_ref().unwrap())
            .await
            .unwrap();

        if &modpack_versions[0].id != config.modpack_version.as_ref().unwrap() {
            items.push("Update Modpack");
        }
    }

    if items.is_empty() {
        println!("{}", "everything is up to date!".green());
        return 0;
    }

    let update = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Update?")
        .default(0)
        .items(&items)
        .interact()
        .unwrap();

    let update = items[update];
    let api = api::mcjars::McjarsApi::new();

    if update == "Update Version" {
        let version_index = versions
            .keys()
            .position(|v| {
                v.as_str()
                    == build
                        .version_id
                        .as_ref()
                        .unwrap_or(build.project_version_id.as_ref().unwrap_or(&String::new()))
                        .as_str()
            })
            .unwrap();
        let versions_java: HashMap<&String, u8> =
            versions.iter().map(|(k, v)| (k, v.java)).collect();
        let versions: Vec<&String> = versions.keys().skip(version_index + 1).rev().collect();

        let server_version = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Jar Version")
            .default(0)
            .items(&versions)
            .interact()
            .unwrap();
        let server_version = &versions[server_version];

        println!(
            "{} {} {}",
            "getting server builds for".bright_black(),
            server_version.to_string().cyan(),
            "...".bright_black()
        );

        let builds = api.builds(&build.r#type, server_version).await.unwrap();

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

        jar::install(server_build, &directory, 1).await.unwrap();

        config.java_version = *versions_java.get(*server_version).unwrap();
        config.save();

        println!(
            "{} {} {} {} {}",
            "installing".bright_black(),
            server_version.cyan(),
            server_build.name.cyan(),
            "...".bright_black(),
            "DONE".green().bold()
        );
    } else if update == "Update Build" {
        let server_version = build
            .version_id
            .unwrap_or(build.project_version_id.unwrap_or("unknown".to_string()));
        println!(
            "{} {} {}",
            "getting server builds for".bright_black(),
            server_version.to_string().cyan(),
            "...".bright_black()
        );

        let builds = api.builds(&build.r#type, &server_version).await.unwrap();
        let builds = builds.iter().rev().collect::<Vec<&api::mcjars::Build>>();

        println!(
            "{} {} {} {}",
            "getting server builds for".bright_black(),
            server_version.to_string().cyan(),
            "...".bright_black(),
            "DONE".green().bold()
        );

        let build_index = builds.iter().position(|b| b.id == build.id).unwrap_or(0);
        let builds: Vec<&&api::mcjars::Build> = builds.iter().skip(build_index + 1).rev().collect();

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

        jar::install(server_build, &directory, 1).await.unwrap();

        println!(
            "{} {} {} {} {}",
            "installing".bright_black(),
            server_version.cyan(),
            server_build.name.cyan(),
            "...".bright_black(),
            "DONE".green().bold()
        );
    } else if update == "Update Modpack" {
        let modpack = modpack.unwrap();
        let modpack_versions = modpack_versions
            .iter()
            .rev()
            .collect::<Vec<&api::modrinth::Version>>();

        let version_index = modpack_versions
            .iter()
            .position(|v| &v.id == config.modpack_version.as_ref().unwrap())
            .unwrap();
        let versions: Vec<&&api::modrinth::Version> = modpack_versions
            .iter()
            .skip(version_index + 1)
            .rev()
            .collect();

        let modpack_version = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Modpack Version?")
            .default(0)
            .items(
                &versions
                    .iter()
                    .map(|v| {
                        v.name
                            .as_ref()
                            .unwrap_or(v.version_number.as_ref().unwrap())
                    })
                    .collect::<Vec<&String>>(),
            )
            .max_length(5)
            .interact()
            .unwrap();
        let modpack_version = &versions[modpack_version];

        println!(
            "{} {} {}",
            "updating".bright_black(),
            modpack.title.cyan(),
            "...".bright_black()
        );

        modpack::install(&directory, &api, modpack_version).await;

        config.modpack_version = Some(modpack_version.id.clone());
        config.save();

        println!(
            "{} {} {} {}",
            "updating to".bright_black(),
            modpack_version
                .name
                .as_ref()
                .unwrap_or(modpack_version.version_number.as_ref().unwrap())
                .cyan(),
            "...".bright_black(),
            "DONE".green().bold()
        );
    }

    0
}
