use crate::{backups, config, detached};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, theme::ColorfulTheme};

pub async fn restore(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let name = matches.get_one::<String>("name");
    let _config = config::Config::new(".mcvcli.json", false);

    if detached::is_running() {
        println!(
            "{} {}",
            "server is currently running, use".red(),
            "mcvcli stop".cyan()
        );
        return Ok(1);
    }

    let list = backups::list();

    let name = if let Some(name) = name {
        name
    } else {
        if list.is_empty() {
            println!("{}", "no backups to restore".red());
            return Ok(1);
        }

        let name = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select backup to restore")
            .items(list.iter().map(|b| &b.name).collect::<Vec<&String>>())
            .default(0)
            .max_length(5)
            .interact()?;

        &list[name].name
    };

    if !list.iter().any(|b| b.name == *name) {
        println!(
            "{} {} {}",
            "backup".red(),
            name.cyan(),
            "does not exist!".red()
        );
        return Ok(1);
    }

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to restore this backup? (This will wipe the current server data!)")
        .default(false)
        .interact()?;

    if !confirm {
        return Ok(1);
    }

    println!("{}", "wiping server directory...".bright_black());

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
        "wiping server directory...".bright_black(),
        "DONE".green().bold()
    );

    println!(
        "{} {} {}",
        "restoring backup".bright_black(),
        name.cyan(),
        "...".bright_black()
    );

    backups::restore(
        list.iter()
            .find(|b| b.name == *name)
            .ok_or_else(|| anyhow::anyhow!("backup {name} not found"))?,
    )?;

    println!(
        "{} {} {} {}",
        "restoring backup".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    Ok(0)
}
