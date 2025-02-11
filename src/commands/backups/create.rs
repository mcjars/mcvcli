use crate::{backups, config};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};

pub async fn create(matches: &ArgMatches) -> i32 {
    let name = matches.get_one::<String>("name").expect("required");
    let _config = config::Config::new(".mcvcli.json", false);

    if backups::list().iter().any(|backup| backup.name == *name) {
        println!(
            "{} {} {}",
            "backup".red(),
            name.cyan(),
            "already exists!".red()
        );
        return 1;
    }

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to create a backup?")
        .default(false)
        .interact()
        .unwrap();

    if !confirm {
        return 1;
    }

    println!("{}", "creating backup...".bright_black());

    backups::create(name);

    println!(
        "{} {}",
        "creating backup...".bright_black(),
        "DONE".green().bold()
    );

    0
}
