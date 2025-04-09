use crate::{config, jar, profiles};

use clap::ArgMatches;
use colored::Colorize;

pub async fn version(matches: &ArgMatches) -> i32 {
    let profile = matches.get_one::<String>("profile");

    if profile.is_some() && !profiles::list().contains(profile.unwrap()) {
        println!(
            "{} {} {}",
            "profile".red(),
            profile.unwrap().cyan(),
            "does not exist!".red()
        );
        return 1;
    }

    let directory = if let Some(profile) = profile {
        format!(".mcvcli.profiles/{}", profile)
    } else {
        ".".to_string()
    };

    println!("{}", "checking installed version ...".bright_black());

    let config = config::Config::new(&format!("{}/.mcvcli.json", directory), false);

    println!(
        "{} {}",
        "checking installed version ...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    println!(
        "{} {}",
        "installed jar location:".bright_black(),
        config.jar_file.cyan()
    );
    println!("{}", "installed jar version:".bright_black());

    if let Some(([build, latest], versions, modpack)) = jar::detect(&directory, &config).await {
        println!("  {} {}", "type:   ".bright_black(), build.r#type.cyan());
        println!(
            "  {} {} {}",
            "version:".bright_black(),
            build
                .version_id
                .as_ref()
                .unwrap_or(
                    build
                        .project_version_id
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                )
                .cyan(),
            if jar::is_latest_version(&build, &versions) {
                "(latest)".green()
            } else {
                "(outdated)".red()
            }
        );
        println!(
            "  {} {} {}",
            "build:  ".bright_black(),
            build.name.cyan(),
            if build.id == latest.id {
                "(latest)".green()
            } else {
                "(outdated)".red()
            }
        );

        if let Some(modpack) = modpack {
            println!("{}", "installed modpack:".bright_black());
            println!(
                "  {} {}",
                "name:       ".bright_black(),
                modpack.title.cyan(),
            );
            println!(
                "  {} {}",
                "description:".bright_black(),
                modpack.description.cyan()
            );
            println!(
                "  {} {}",
                "project id: ".bright_black(),
                modpack.id.unwrap().cyan()
            );
            println!(
                "  {} {} {}",
                "version id: ".bright_black(),
                config.modpack_version.as_ref().unwrap().cyan(),
                if modpack.versions.last().unwrap() == &config.modpack_version.unwrap() {
                    "(latest)".green()
                } else {
                    "(outdated)".red()
                }
            );
            println!(
                "  {} {}",
                "downloads:  ".bright_black(),
                modpack.downloads.to_string().cyan()
            );
        }
    } else {
        println!("  {} {}", "type:   ".bright_black(), "unknown".cyan());
        println!("  {} {}", "version:".bright_black(), "unknown".cyan());
        println!("  {} {}", "build:  ".bright_black(), "unknown".cyan());
    }

    println!(
        "{} {}",
        "installed java version:".bright_black(),
        config.java_version.to_string().cyan()
    );

    0
}
