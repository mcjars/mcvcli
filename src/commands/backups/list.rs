use crate::{backups, config};

use clap::ArgMatches;
use colored::Colorize;
use human_bytes::human_bytes;

pub async fn list(_matches: &ArgMatches) -> i32 {
    let _config = config::Config::new(".mcvcli.json", false);

    println!("{}", "listing backups...".bright_black());

    let list = backups::list();

    println!(
        "{} {}",
        "listing backups...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    for backup in list.clone() {
        println!("{}", backup.name.cyan().bold());

        println!("  {} {}", "path:   ".bright_black(), backup.path.cyan());
        println!(
            "  {} {}",
            "size:   ".bright_black(),
            human_bytes(backup.size as f64).cyan()
        );
        println!(
            "  {} {}",
            "created:".bright_black(),
            backup
                .created
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
                .cyan()
        );

        if backup.name != list.last().unwrap().name {
            println!();
        }
    }

    if list.is_empty() {
        println!("{}", "no backups found".red());
    }

    return 0;
}
