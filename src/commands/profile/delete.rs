use crate::{config, profiles};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, FuzzySelect, theme::ColorfulTheme};

pub async fn delete(matches: &ArgMatches) -> i32 {
    let name = matches.get_one::<String>("name");
    let config = config::Config::new(".mcvcli.json", false);

    let list = profiles::list();

    let name = if let Some(name) = name {
        name
    } else {
        if list.is_empty() {
            println!("{}", "no profiles to delete".red());
            return 1;
        }

        let name = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select profile to delete")
            .items(&list)
            .default(0)
            .max_length(5)
            .interact()
            .unwrap();

        &list[name]
    };

    if config.profile_name == *name {
        println!(
            "{} {} {}",
            "profile".red(),
            name.cyan(),
            "is currently in use!".red()
        );
        return 1;
    }

    if !list.contains(name) {
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

    let directory = format!(".mcvcli.profiles/{name}");
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
