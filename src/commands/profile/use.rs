use crate::{config, detached, profiles};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};
use std::path::Path;

pub async fn r#use(matches: &ArgMatches) -> i32 {
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

    let list = profiles::list();

    let name = if let Some(name) = name {
        name
    } else {
        if list.is_empty() {
            println!("{}", "no profiles to use".red());
            return 1;
        }

        let name = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select profile to use")
            .items(&list)
            .default(0)
            .max_length(5)
            .interact()
            .unwrap();
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

    println!(
        "{} {} {}",
        "switching to profile".bright_black(),
        name.cyan(),
        "...".bright_black()
    );

    let new_directory = format!(".mcvcli.profiles/{}", config.profile_name);
    let old_directory = format!(".mcvcli.profiles/{}", name);

    if !Path::new(&new_directory).exists() {
        std::fs::create_dir_all(&new_directory).unwrap();
    }

    for entry in std::fs::read_dir(".").unwrap().flatten() {
        let path = entry.path();

        if path.file_name().unwrap() == ".mcvcli.profiles" {
            continue;
        }

        std::fs::rename(
            &path,
            format!(
                "{}/{}",
                new_directory,
                path.file_name().unwrap().to_str().unwrap()
            ),
        )
        .unwrap();
    }

    for entry in std::fs::read_dir(&old_directory).unwrap().flatten() {
        let path = entry.path();

        std::fs::rename(
            &path,
            format!("./{}", path.file_name().unwrap().to_str().unwrap()),
        )
        .unwrap();
    }

    println!(
        "{} {} {} {}",
        "switching to profile".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    0
}
