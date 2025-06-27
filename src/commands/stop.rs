use crate::{config, detached};

use clap::ArgMatches;
use colored::Colorize;
use std::io::Write;

pub async fn stop(matches: &ArgMatches) -> i32 {
    let timeout = *matches.get_one::<u64>("timeout").expect("required");
    let mut config = config::Config::new(".mcvcli.json", false);

    if !detached::status(config.pid) {
        println!(
            "{} {}",
            "server is not running, use".red(),
            "mcvcli start --detached".cyan()
        );
        return 1;
    }

    let [mut stdin, mut stdout, mut stderr] = detached::get_pipes(&config.identifier.unwrap());
    let mut threads = Vec::new();

    println!(
        "{}",
        format!("stopping server ({timeout}s before being killed) ...").bright_black()
    );

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut stdout, &mut std::io::stdout()).unwrap();
    }));

    threads.push(tokio::spawn(async move {
        std::io::copy(&mut stderr, &mut std::io::stderr()).unwrap();
    }));

    threads.push(tokio::spawn({
        let stop_command = config.stop_command.clone();

        async move {
            stdin.write_all((stop_command + "\n").as_bytes()).unwrap();

            std::io::copy(&mut std::io::stdin(), &mut stdin).unwrap();
        }
    }));

    let kill = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;

        if detached::status(config.pid) {
            println!(
                "{}",
                "server is taking too long to stop, killing it ...".bright_black()
            );

            let pid = sysinfo::Pid::from(config.pid.unwrap());
            let sys = sysinfo::System::new_all();
            let process = sys.process(pid).unwrap();

            process.kill_with(sysinfo::Signal::Kill);

            println!(
                "{} {}",
                "server is taking too long to stop, killing it ...".bright_black(),
                "DONE".green().bold()
            );
        }
    });

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

    kill.abort();

    for thread in threads {
        thread.abort();
    }

    config.pid = None;
    config.identifier = None;
    config.save();

    0
}
