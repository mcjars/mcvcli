use crate::{config, profiles};

use clap::ArgMatches;
use colored::Colorize;

pub async fn config(matches: &ArgMatches) -> i32 {
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

    let mut config = if let Some(profile) = profile {
        config::Config::new(&format!(".mcvcli.profiles/{}/.mcvcli.json", profile), false)
    } else {
        config::Config::new(".mcvcli.json", false)
    };

    let ram = matches.get_one::<u16>("ram");
    let flags = matches.get_one::<String>("flags");
    let args = matches.get_one::<String>("args");

    if ram.is_none() && flags.is_none() && args.is_none() {
        println!(
            "{} {}",
            "no changes made, use".bright_black(),
            "mcvcli config --help".cyan(),
        );
        return 1;
    }

    println!("{}", "updating config ...".bright_black());

    if let Some(ram) = ram {
        config.ram_mb = *ram;
    }
    if let Some(flags) = flags {
        config.extra_flags = vec![flags.to_string()];
    }
    if let Some(args) = args {
        config.extra_args = vec![args.to_string()];
    }

    config.save();

    println!(
        "{} {}",
        "updating config ...".bright_black(),
        "DONE".green().bold()
    );

    0
}
