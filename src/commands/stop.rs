use crate::{config, detached};

use clap::ArgMatches;
use colored::Colorize;

pub async fn stop(_matches: &ArgMatches) -> i32 {
    let mut config = config::Config::new(".mcvcli.json", false);

    if !detached::status(config.pid) {
        println!(
            "{} {}",
            "server is not running, use".red(),
            "mcvcli start --detach".cyan()
        );
        return 1;
    }

    println!("{}", "stopping server ...".bright_black());

    let pid = sysinfo::Pid::from(config.pid.unwrap());
    let sys = sysinfo::System::new_all();
    let process = sys.process(pid).unwrap();

    let (mut stdin, mut stdout, mut stderr) = detached::get_pipes(&config.identifier.unwrap());
    let mut threads = Vec::new();

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut std::io::stdin(), &mut stdin).unwrap();
    }));

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut stdout, &mut std::io::stdout()).unwrap();
    }));

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut stderr, &mut std::io::stderr()).unwrap();
    }));

    process.kill_with(sysinfo::Signal::Interrupt);

    tokio::spawn(async move {
        loop {
            if !detached::status(config.pid) {
                println!();
                println!(
                    "{} {}",
                    "stopping server ...".bright_black(),
                    "DONE".green().bold()
                );
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

    config.pid = None;
    config.identifier = None;
    config.save();

    0
}
