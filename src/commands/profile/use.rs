use crate::{config, profiles};

use clap::ArgMatches;
use colored::Colorize;
use std::path::Path;

pub async fn r#use(matches: &ArgMatches) -> i32 {
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

    let entries = std::fs::read_dir(".").unwrap();
    for entry in entries {
        let entry = entry.unwrap();
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

    let entries = std::fs::read_dir(&old_directory).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
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
