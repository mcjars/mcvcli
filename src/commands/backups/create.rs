use crate::{backups, config};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};

pub async fn create(matches: &ArgMatches) -> i32 {
    let name = matches.get_one::<String>("name").expect("required");
    let format = matches.get_one::<String>("format").expect("required");
    let format: Option<backups::BackupFormat> =
        serde_json::from_str(&format!("\"{}\"", format)).ok();
    let _config = config::Config::new(".mcvcli.json", false);

    if format.is_none() {
        println!(
            "{} {}",
            "invalid format, accepted values:".red(),
            "(zip, tar, tar.gz, tar.xz)".cyan()
        );
        return 1;
    }

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

    backups::create(name, &format.unwrap());

    println!(
        "{} {}",
        "creating backup...".bright_black(),
        "DONE".green().bold()
    );

    0
}
