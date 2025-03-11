use crate::{config, java};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};

pub async fn r#use(matches: &ArgMatches) -> i32 {
    let version = matches
        .get_one::<String>("version")
        .map(|v| v.parse::<u8>().expect("invalid version"));
    let mut config = config::Config::new(".mcvcli.json", false);

    println!("{}", "listing java versions...".bright_black());

    let java = java::Java::new();
    let list: Vec<u8> = java.versions().await.into_iter().rev().collect();

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
            .with_prompt("Select java version to use")
            .items(
                &list
                    .iter()
                    .map(|p| {
                        format!(
                            "java {} {}",
                            p,
                            if *p == config.java_version {
                                "(currently using)"
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

    println!(
        "{} {} {}",
        "using java".bright_black(),
        version.to_string().cyan(),
        "...".bright_black()
    );

    if !java.installed().iter().any(|(v, _)| *v == version) {
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
    }

    config.java_version = version;
    config.save();

    println!(
        "{} {} {} {}",
        "using java".bright_black(),
        version.to_string().cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    0
}
