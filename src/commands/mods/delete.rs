use crate::{api, config};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{MultiSelect, theme::ColorfulTheme};
use std::path::Path;

pub async fn delete(_matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let _config = config::Config::new(".mcvcli.json", false);

    if !Path::new("mods").exists() {
        println!("{}", "no mods folder found.".red());
        return Ok(1);
    }

    println!("{}", "listing mods...".bright_black());

    let list = api::modrinth::lookup("mods", None, None).await?;

    println!(
        "{} {}",
        "listing mods...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    if list.is_empty() {
        println!("{}", "no mods to delete".red());
        return Ok(1);
    }

    let mods = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select mods to delete")
        .items(
            list.values()
                .map(|p| p.title.clone())
                .collect::<Vec<String>>(),
        )
        .max_length(10)
        .interact()?;
    println!();

    for i in mods {
        let Some((path, project)) = list.get_index(i) else {
            continue;
        };
        println!(
            "{} {} {}",
            "deleting mod".bright_black(),
            project.title.cyan(),
            "...".bright_black(),
        );

        std::fs::remove_file(path)?;

        println!(
            "{} {} {} {}",
            "deleting mod".bright_black(),
            project.title.cyan(),
            "...".bright_black(),
            "DONE".green().bold(),
        );
    }

    Ok(0)
}
