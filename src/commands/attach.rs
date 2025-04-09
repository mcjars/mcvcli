use crate::{config, detached};

use clap::ArgMatches;
use colored::Colorize;

pub async fn attach(_matches: &ArgMatches) -> i32 {
    let mut config = config::Config::new(".mcvcli.json", false);

    if !detached::status(config.pid) {
        println!(
            "{} {}",
            "server is not running, use".red(),
            "mcvcli start --detach".cyan()
        );
        return 1;
    }

    println!("{}", "attaching to server ...".bright_black());

    let [mut stdin, mut stdout, mut stderr] =
        detached::get_pipes(&config.identifier.clone().unwrap());
    let mut threads = Vec::with_capacity(3);

    println!(
        "{} {}",
        "attaching to server ...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut std::io::stdin(), &mut stdin).unwrap();
    }));

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut stdout, &mut std::io::stdout().lock()).unwrap();
    }));

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut stderr, &mut std::io::stderr().lock()).unwrap();
    }));

    tokio::spawn(async move {
        loop {
            if !detached::status(config.pid) {
                println!();
                println!("{}", "server has stopped".red());

                config.pid = None;
                config.identifier = None;
                config.save();

                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    })
    .await
    .unwrap();

    for thread in threads {
        thread.abort();
    }

    0
}
