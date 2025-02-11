use crate::{backups, config};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};

pub async fn delete(matches: &ArgMatches) -> i32 {
    let name = matches.get_one::<String>("name").unwrap();
    let _config = config::Config::new(".mcvcli.json", false);

    let backups = backups::list();
    if !backups.iter().any(|b| b.name == *name) {
        println!(
            "{} {} {}",
            "backup".red(),
            name.cyan(),
            "does not exist!".red()
        );
        return 1;
    }

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to delete this backup?")
        .default(false)
        .interact()
        .unwrap();

    if !confirm {
        return 1;
    }

    println!(
        "{} {} {}",
        "deleting backup".bright_black(),
        name.cyan(),
        "...".bright_black()
    );

    let backup = backups.iter().find(|b| b.name == *name).unwrap();
    std::fs::remove_file(&backup.path).unwrap();

    println!(
        "{} {} {} {}",
        "deleting backup".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    return 0;
}
