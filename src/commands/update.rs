use crate::{api, config, detached, jar, modpack, profiles};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{FuzzySelect, Select, theme::ColorfulTheme};
use std::collections::HashMap;

pub async fn update(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let profile = matches.get_one::<String>("profile");
    let config = config::Config::new(".mcvcli.json", false);

    if detached::is_running() {
        println!(
            "{} {}",
            "server is currently running, use".red(),
            "mcvcli stop".cyan()
        );
        return Ok(1);
    }

    if let Some(profile) = profile
        && config.profile_name == *profile
    {
        println!(
            "{} {} {}",
            "profile".red(),
            profile.cyan(),
            "is currently in use!".red()
        );
        return Ok(1);
    }

    if let Some(profile) = profile
        && !profiles::list().contains(profile)
    {
        println!(
            "{} {} {}",
            "profile".red(),
            profile.cyan(),
            "does not exist!".red()
        );
        return Ok(1);
    }

    let directory = if let Some(profile) = profile {
        format!(".mcvcli.profiles/{profile}")
    } else {
        ".".to_string()
    };

    println!("{}", "checking installed version ...".bright_black());

    let mut config = config::Config::new(&format!("{directory}/.mcvcli.json"), false);
    let detected = jar::detect(&directory.clone(), &config).await;

    let Some(([build, latest], versions, modpack)) = detected else {
        println!(
            "{} {}",
            "checking installed version ...".bright_black(),
            "FAILED".red().bold()
        );
        return Ok(1);
    };

    println!(
        "{} {}",
        "checking installed version ...".bright_black(),
        "DONE".green().bold()
    );
    println!();

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

    if build.uuid != latest.uuid {
        items.push("Update Build");
    }

    let mut modpack_versions = Vec::new();
    if modpack.is_some() {
        let slug = config
            .modpack_slug
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("modpack has no slug"))?;
        modpack_versions = api::modrinth::versions(slug).await?;

        if modpack_versions.first().map(|v| &v.id) != config.modpack_version.as_ref() {
            items.push("Update Modpack");
        }
    }

    if items.is_empty() {
        println!("{}", "everything is up to date!".green());
        return Ok(0);
    }

    let update = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Update?")
        .default(0)
        .items(&items)
        .interact()?;

    let update = items[update];
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
            .ok_or_else(|| anyhow::anyhow!("current version not found in version list"))?;
        let versions_java: HashMap<&String, u8> =
            versions.iter().map(|(k, v)| (k, v.java)).collect();
        let versions: Vec<&String> = versions.keys().skip(version_index + 1).rev().collect();

        let server_version = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Jar Version")
            .default(0)
            .items(&versions)
            .max_length(10)
            .interact()?;
        let server_version = &versions[server_version];

        println!(
            "{} {} {}",
            "getting server builds for".bright_black(),
            server_version.to_string().cyan(),
            "...".bright_black()
        );

        let builds = api::mcjars::builds(&build.r#type, server_version).await?;

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
            .items(builds.iter().map(|b| &b.name).collect::<Vec<&String>>())
            .max_length(10)
            .interact()?;

        let server_build = &builds[server_build];

        println!(
            "{} {} {} {}",
            "installing".bright_black(),
            server_version.cyan(),
            server_build.name.cyan(),
            "...".bright_black()
        );

        jar::install(server_build, &directory, 1).await?;

        config.java_version = *versions_java
            .get(*server_version)
            .ok_or_else(|| anyhow::anyhow!("no java version for {server_version}"))?;
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

        let builds = api::mcjars::builds(&build.r#type, &server_version).await?;
        let builds = builds.iter().rev().collect::<Vec<&api::mcjars::Build>>();

        println!(
            "{} {} {} {}",
            "getting server builds for".bright_black(),
            server_version.to_string().cyan(),
            "...".bright_black(),
            "DONE".green().bold()
        );

        let build_index = builds
            .iter()
            .position(|b| b.uuid == build.uuid)
            .unwrap_or(0);
        let builds: Vec<&&api::mcjars::Build> = builds.iter().skip(build_index + 1).rev().collect();

        let server_build = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Jar Build")
            .default(0)
            .items(builds.iter().map(|b| &b.name).collect::<Vec<&String>>())
            .max_length(10)
            .interact()?;

        let server_build = &builds[server_build];

        println!(
            "{} {} {} {}",
            "installing".bright_black(),
            server_version.cyan(),
            server_build.name.cyan(),
            "...".bright_black()
        );

        jar::install(server_build, &directory, 1).await?;

        println!(
            "{} {} {} {} {}",
            "installing".bright_black(),
            server_version.cyan(),
            server_build.name.cyan(),
            "...".bright_black(),
            "DONE".green().bold()
        );
    } else if update == "Update Modpack" {
        let modpack = modpack.ok_or_else(|| anyhow::anyhow!("no modpack installed"))?;
        let modpack_versions = modpack_versions
            .iter()
            .rev()
            .collect::<Vec<&api::modrinth::Version>>();

        let current_version = config
            .modpack_version
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no modpack version set"))?;
        let version_index = modpack_versions
            .iter()
            .position(|v| &v.id == current_version)
            .ok_or_else(|| anyhow::anyhow!("current modpack version not found"))?;
        let versions: Vec<&&api::modrinth::Version> = modpack_versions
            .iter()
            .skip(version_index + 1)
            .rev()
            .collect();

        let modpack_version = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Modpack Version?")
            .default(0)
            .items(
                versions
                    .iter()
                    .map(|v| {
                        v.name
                            .clone()
                            .or_else(|| v.version_number.clone())
                            .unwrap_or_else(|| "unknown".to_string())
                    })
                    .collect::<Vec<String>>(),
            )
            .max_length(10)
            .interact()?;
        let modpack_version = &versions[modpack_version];

        println!(
            "{} {} {}",
            "updating".bright_black(),
            modpack.title.cyan(),
            "...".bright_black()
        );

        modpack::install(&directory, modpack_version).await?;

        config.modpack_version = Some(modpack_version.id.clone());
        config.save();

        println!(
            "{} {} {} {}",
            "updating to".bright_black(),
            modpack_version
                .name
                .as_deref()
                .or(modpack_version.version_number.as_deref())
                .unwrap_or("unknown")
                .cyan(),
            "...".bright_black(),
            "DONE".green().bold()
        );
    }

    Ok(0)
}
