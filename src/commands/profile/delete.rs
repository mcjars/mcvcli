use crate::{config, profiles};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};

pub async fn delete(matches: &ArgMatches) -> i32 {
    let name = matches.get_one::<String>("name").unwrap();
    let config = config::Config::new(".mcvcli.json", false);

    if config.profile_name == *name {
        println!(
            "{} {} {}",
            "profile".red(),
            name.cyan(),
            "is currently in use!".red()
        );
        return 1;
    }

    if !profiles::list().contains(name) {
        println!(
            "{} {} {}",
            "profile".red(),
            name.cyan(),
            "does not exist!".red()
        );
        return 1;
    }

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to delete this profile?")
        .default(false)
        .interact()
        .unwrap();

    if !confirm {
        return 1;
    }

    println!(
        "{} {} {}",
        "deleting profile".bright_black(),
        name.cyan(),
        "...".bright_black()
    );

    let directory = format!(".mcvcli.profiles/{}", name);
    std::fs::remove_dir_all(directory).unwrap();

    println!(
        "{} {} {} {}",
        "deleting profile".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    0
}
