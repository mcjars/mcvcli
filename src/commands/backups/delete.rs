use crate::{backups, config};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, theme::ColorfulTheme};

pub async fn delete(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let name = matches.get_one::<String>("name");
    let _config = config::Config::new(".mcvcli.json", false);

    let list = backups::list();

    let name = if let Some(name) = name {
        name
    } else {
        if list.is_empty() {
            println!("{}", "no backups to delete".red());
            return Ok(1);
        }

        let name = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select backup to delete")
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
        .with_prompt("Are you sure you want to delete this backup?")
        .default(false)
        .interact()?;

    if !confirm {
        return Ok(1);
    }

    println!(
        "{} {} {}",
        "deleting backup".bright_black(),
        name.cyan(),
        "...".bright_black()
    );

    let backup = list
        .iter()
        .find(|b| b.name == *name)
        .ok_or_else(|| anyhow::anyhow!("backup {name} not found"))?;
    std::fs::remove_file(&backup.path)?;

    println!(
        "{} {} {} {}",
        "deleting backup".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    Ok(0)
}
