use clap::ArgMatches;
use colored::Colorize;

pub async fn query(matches: &ArgMatches) -> i32 {
    let mut address = matches
        .get_one::<String>("address")
        .expect("required")
        .to_string();
    let use_query = matches.get_one::<bool>("query").expect("required");

    if !address.contains(':') {
        address.push_str(":25565");
    }

    println!(
        "{} {}{}",
        "querying server".bright_black(),
        address.bright_cyan(),
        "...".bright_black()
    );

    let (host, port) = match address.split_once(':') {
        Some((host, port)) => (host.to_string(), port.parse::<u16>().unwrap_or(25565)),
        None => (address.clone(), 25565),
    };

    let server = msp::Conf::create_with_port(&host, port);

    if *use_query {
        let status = match server.query_full() {
            Ok(status) => status,
            Err(e) => {
                println!("{}: {}", "Error".red(), e);
                return 1;
            }
        };

        println!("{status:?}");
    } else {
        match server.get_server_status() {
            Ok(status) => {
                println!(
                    "{} {}{} {}",
                    "querying server".bright_black(),
                    address.bright_cyan(),
                    "...".bright_black(),
                    "DONE".green().bold()
                );
                println!();

                println!("{}", address.bright_cyan().underline());
                println!(
                    "  {} {}",
                    "version: ".bright_black(),
                    status.version.name.cyan()
                );
                println!(
                    "  {} {}",
                    "protocol:".bright_black(),
                    status.version.protocol.to_string().cyan()
                );

                println!("  {}", "players:".bright_black());
                println!(
                    "    {} {}",
                    "online:".bright_black(),
                    status.players.online.to_string().cyan()
                );
                println!(
                    "    {} {}",
                    "max:   ".bright_black(),
                    status.players.max.to_string().cyan()
                );
                println!("    {}", "sample:".bright_black());
                for player in status.players.sample {
                    println!("      {}", player.name.cyan());
                }

                println!("  {}", "motd:".bright_black());
                for line in status.description.text.lines() {
                    println!("    {}", line.cyan());
                }
            }
            Err(_) => {
                let status = match server.get_netty_server_status() {
                    Ok(status) => status,
                    Err(_) => match server.get_legacy_server_status() {
                        Ok(status) => status,
                        Err(_) => {
                            let status = match server.get_beta_legacy_server_status() {
                                Ok(status) => status,
                                Err(e) => {
                                    println!("{}: {}", "Error".red(), e);
                                    return 1;
                                }
                            };

                            println!(
                                "{} {}{} {}",
                                "querying server".bright_black(),
                                address.bright_cyan(),
                                "...".bright_black(),
                                "DONE".green().bold()
                            );
                            println!();

                            println!("{}", address.bright_cyan().underline());
                            println!("  {}", "players:".bright_black());
                            println!(
                                "    {} {}",
                                "online:".bright_black(),
                                status.online_players.to_string().cyan()
                            );
                            println!(
                                "    {} {}",
                                "max:   ".bright_black(),
                                status.max_players.to_string().cyan()
                            );

                            return 0;
                        }
                    },
                };

                println!(
                    "{} {}{} {}",
                    "querying server".bright_black(),
                    address.bright_cyan(),
                    "...".bright_black(),
                    "DONE".green().bold()
                );
                println!();

                println!("{}", address.bright_cyan().underline());
                println!(
                    "  {} {}",
                    "version: ".bright_black(),
                    status.server_version.cyan()
                );
                println!(
                    "  {} {}",
                    "protocol:".bright_black(),
                    status.protocol_version.to_string().cyan()
                );

                println!("  {}", "players:".bright_black());
                println!(
                    "    {} {}",
                    "online:".bright_black(),
                    status.online_players.to_string().cyan()
                );
                println!(
                    "    {} {}",
                    "max:   ".bright_black(),
                    status.max_players.to_string().cyan()
                );

                println!("  {}", "motd:".bright_black());
                for line in status.motd.lines() {
                    println!("    {}", line.cyan());
                }
            }
        };
    }

    0
}
