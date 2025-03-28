use crate::{config, detached};

use chrono::DateTime;
use clap::ArgMatches;
use colored::Colorize;
use human_bytes::human_bytes;

pub async fn status(_matches: &ArgMatches) -> i32 {
    let config = config::Config::new(".mcvcli.json", false);

    if !detached::status(config.pid) {
        println!(
            "{} {}",
            "server is not running, use".red(),
            "mcvcli start --detached".cyan()
        );
        return 1;
    }

    println!("{}", "getting server status ...".bright_black());

    let pid = sysinfo::Pid::from(config.pid.unwrap());
    let sys = sysinfo::System::new_all();

    let process = sys.process(pid).unwrap();

    println!(
        "{} {}",
        "getting server status ...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    println!(
        "{} (pid {})",
        config.profile_name.cyan().cyan().bold().underline(),
        pid.as_u32()
    );
    println!(
        "  {} {}",
        "memory usage:".bright_black(),
        human_bytes(process.memory() as f64).cyan()
    );

    let uptime = chrono::Utc::now().timestamp() - process.start_time() as i64;
    println!(
        "  {} {} ({}h {}m {}s)",
        "start time:  ".bright_black(),
        DateTime::from_timestamp(process.start_time() as i64, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .cyan(),
        (uptime / 3600).to_string().cyan(),
        ((uptime % 3600) / 60).to_string().cyan(),
        (uptime % 60).to_string().cyan()
    );

    0
}
