use crate::{config, detached, profiles};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use std::path::Path;

pub async fn r#use(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let name = matches.get_one::<String>("name");
    let config = config::Config::new(".mcvcli.json", false);

    if detached::is_running() {
        println!(
            "{} {}",
            "server is currently running, use".red(),
            "mcvcli stop".cyan()
        );
        return Ok(1);
    }

    let list = profiles::list();

    let name = if let Some(name) = name {
        name
    } else {
        if list.is_empty() {
            println!("{}", "no profiles to use".red());
            return Ok(1);
        }

        let name = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select profile to use")
            .items(&list)
            .default(0)
            .max_length(5)
            .interact()?;
        println!();

        &list[name]
    };

    if config.profile_name == *name {
        println!(
            "{} {} {}",
            "profile".red(),
            name.cyan(),
            "is currently in use!".red()
        );
        return Ok(1);
    }

    if !list.contains(name) {
        println!(
            "{} {} {}",
            "profile".red(),
            name.cyan(),
            "does not exist!".red()
        );
        return Ok(1);
    }

    println!(
        "{} {} {}",
        "switching to profile".bright_black(),
        name.cyan(),
        "...".bright_black()
    );

    let new_directory = format!(".mcvcli.profiles/{}", config.profile_name);
    let old_directory = format!(".mcvcli.profiles/{name}");

    if !Path::new(&new_directory).exists() {
        std::fs::create_dir_all(&new_directory)?;
    }

    for entry in std::fs::read_dir(".")?.flatten() {
        let path = entry.path();

        let Some(file_name) = path.file_name() else {
            continue;
        };

        if file_name == ".mcvcli.profiles" {
            continue;
        }

        std::fs::rename(
            &path,
            format!("{}/{}", new_directory, file_name.to_string_lossy()),
        )?;
    }

    for entry in std::fs::read_dir(&old_directory)?.flatten() {
        let path = entry.path();

        let Some(file_name) = path.file_name() else {
            continue;
        };

        std::fs::rename(&path, format!("./{}", file_name.to_string_lossy()))?;
    }

    println!(
        "{} {} {} {}",
        "switching to profile".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    Ok(0)
}
