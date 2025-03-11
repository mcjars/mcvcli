use crate::{backups, config, detached};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, theme::ColorfulTheme};

pub async fn restore(matches: &ArgMatches) -> i32 {
    let name = matches.get_one::<String>("name");
    let config = config::Config::new(".mcvcli.json", false);

    if detached::status(config.pid) {
        println!(
            "{} {}",
            "server is currently running, use".red(),
            "mcvcli stop".cyan()
        );
        return 1;
    }

    let list = backups::list();

    let name = if let Some(name) = name {
        name
    } else {
        if list.is_empty() {
            println!("{}", "no backups to restore".red());
            return 1;
        }

        let name = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select backup to restore")
            .items(&list.iter().map(|b| &b.name).collect::<Vec<&String>>())
            .default(0)
            .max_length(5)
            .interact()
            .unwrap();

        &list[name].name
    };

    if !list.iter().any(|b| b.name == *name) {
        println!(
            "{} {} {}",
            "backup".red(),
            name.cyan(),
            "does not exist!".red()
        );
        return 1;
    }

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to restore this backup? (This will wipe the current server data!)")
        .default(false)
        .interact()
        .unwrap();

    if !confirm {
        return 1;
    }

    println!("{}", "wiping server directory...".bright_black());

    let entries = std::fs::read_dir(".").unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .starts_with(".mcvcli")
        {
            continue;
        }

        if path.is_dir() {
            std::fs::remove_dir_all(&path).unwrap();
        } else {
            std::fs::remove_file(&path).unwrap();
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

    backups::restore(list.iter().find(|b| b.name == *name).unwrap());

    println!(
        "{} {} {} {}",
        "restoring backup".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    0
}
