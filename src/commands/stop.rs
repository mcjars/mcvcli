use crate::{config, detached};

use clap::ArgMatches;
use colored::Colorize;

pub async fn stop(matches: &ArgMatches) -> i32 {
    let timeout = matches.get_one::<String>("timeout").expect("required");
    let timeout = timeout
        .parse::<u64>()
        .expect("timeout must be a non-negative integer");
    let mut config = config::Config::new(".mcvcli.json", false);

    if !detached::status(config.pid) {
        println!(
            "{} {}",
            "server is not running, use".red(),
            "mcvcli start --detached".cyan()
        );
        return 1;
    }

    println!(
        "{}",
        format!("stopping server ({}s before being killed) ...", timeout).bright_black()
    );

    let pid = sysinfo::Pid::from(config.pid.unwrap());
    let sys = sysinfo::System::new_all();
    let process = sys.process(pid).unwrap();

    let [mut stdin, mut stdout, mut stderr] = detached::get_pipes(&config.identifier.unwrap());
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

    let kill = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;

        if detached::status(config.pid) {
            println!(
                " {}",
                "server is taking too long to stop, killing it ..."
                    .bright_black()
                    .italic()
            );

            let pid = sysinfo::Pid::from(config.pid.unwrap());
            let sys = sysinfo::System::new_all();
            let process = sys.process(pid).unwrap();

            process.kill_with(sysinfo::Signal::Kill);

            println!(
                " {} {}",
                "server is taking too long to stop, killing it ..."
                    .bright_black()
                    .italic(),
                "DONE".green().bold().italic()
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
