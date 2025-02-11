use crate::{commands, config, profiles};

use clap::ArgMatches;
use colored::Colorize;

pub async fn create(matches: &ArgMatches) -> i32 {
    let name = matches.get_one::<String>("name").unwrap();
    let config = config::Config::new(".mcvcli.json", false);

    if profiles::list().contains(name) {
        println!(
            "{} {} {}",
            "profile".red(),
            name.cyan(),
            "already exists!".red()
        );
        return 1;
    }

    if config.profile_name == *name {
        println!(
            "{} {} {}",
            "profile".red(),
            name.cyan(),
            "is currently in use!".red()
        );
        return 1;
    }

    println!(
        "{} {} {}",
        "creating profile".bright_black(),
        name.cyan(),
        "...".bright_black()
    );

    let directory = format!(".mcvcli.profiles/{}", name);
    commands::init::init(matches, Some(&directory), Some(name)).await;

    println!(
        "{} {} {} {}",
        "creating profile".bright_black(),
        name.cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    0
}
