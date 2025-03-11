use crate::java;

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};

pub async fn install(matches: &ArgMatches) -> i32 {
    let version = matches
        .get_one::<String>("version")
        .map(|v| v.parse::<u8>().expect("invalid version"));

    println!("{}", "listing java versions...".bright_black());

    let java = java::Java::new();
    let list: Vec<u8> = java.versions().await.into_iter().rev().collect();
    let installed = java.installed();

    println!(
        "{} {}",
        "listing java versions...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    let version = if let Some(version) = version {
        version
    } else {
        let version = FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select java version to install")
            .items(
                &list
                    .iter()
                    .map(|p| {
                        format!(
                            "java {} {}",
                            p,
                            if installed.iter().any(|(version, _)| version == p) {
                                "(reinstall)"
                            } else {
                                ""
                            }
                        )
                    })
                    .collect::<Vec<String>>(),
            )
            .default(0)
            .max_length(5)
            .interact()
            .unwrap();
        println!();

        list[version]
    };

    if installed.iter().any(|(v, _)| *v == version) {
        println!(
            "{} {} {}",
            "java".bright_black(),
            version.to_string().cyan(),
            "already installed, removing...".bright_black()
        );

        java.remove(version);

        println!(
            "{} {} {} {}",
            "java".bright_black(),
            version.to_string().cyan(),
            "already installed, removing...".bright_black(),
            "DONE".green().bold()
        );
    }

    println!(
        "{} {} {}",
        "installing java".bright_black(),
        version.to_string().cyan(),
        "...".bright_black()
    );

    java.install(version).await;

    println!(
        "{} {} {} {}",
        "installing java".bright_black(),
        version.to_string().cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    0
}
