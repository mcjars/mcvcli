use crate::{backups, config};

use clap::ArgMatches;
use colored::Colorize;
use human_bytes::human_bytes;

pub async fn list(_matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let _config = config::Config::new(".mcvcli.json", false);

    println!("{}", "listing backups...".bright_black());

    let list = backups::list();

    println!(
        "{} {}",
        "listing backups...".bright_black(),
        "DONE".green().bold()
    );

    if list.is_empty() {
        println!();
        println!("{}", "no backups found".red());
        return Ok(1);
    }

    for backup in list {
        println!();
        println!("{}", backup.name.cyan().bold().underline());

        println!("  {} {}", "path:   ".bright_black(), backup.path.cyan());
        println!(
            "  {} {}",
            "format: ".bright_black(),
            backups::extension(&backup.format).cyan()
        );
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
    }

    Ok(0)
}
