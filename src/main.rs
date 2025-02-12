mod api;
mod backups;
mod commands;
mod config;
mod jar;
mod java;
mod modpack;
mod profiles;
mod progress;

use clap::{Arg, Command};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn cli() -> Command {
    Command::new("mcvcli")
        .about("A simple CLI for interacting with Minecraft servers")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .version(VERSION)
        .subcommand(
            Command::new("upgrade")
                .about("Upgrades the CLI to the latest version")
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new("init")
                .about("Initializes a new Minecraft server")
                .arg(
                    Arg::new("directory")
                        .help("The directory to initialize the server in")
                        .num_args(1)
                        .default_value(".")
                        .required(false),
                )
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new("install")
                .about("Install a new version of the Minecraft server")
                .arg(
                    Arg::new("wipe")
                        .long("wipe")
                        .short('w')
                        .help("Wipe the server directory before installing")
                        .num_args(0)
                        .default_value("false")
                        .required(false),
                )
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new("start")
                .about("Starts the Minecraft server")
                .arg(
                    Arg::new("eula")
                        .long("eula")
                        .short('e')
                        .help("Accept the Minecraft EULA automatically")
                        .num_args(0)
                        .default_value("false")
                        .required(false),
                )
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new("lookup")
                .about("Looks up a Player on your server")
                .arg(
                    Arg::new("player")
                        .help("The player to look up")
                        .num_args(1)
                        .required(true),
                )
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("version")
                .about("Gets the installed version of the Minecraft server")
                .arg(
                    Arg::new("profile")
                        .long("profile")
                        .short('p')
                        .help("The profile to get the version of")
                        .num_args(1)
                        .required(false),
                )
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new("update")
                .about("Updates the installed version of the Minecraft server")
                .arg(
                    Arg::new("profile")
                        .long("profile")
                        .short('p')
                        .help("The profile to update")
                        .num_args(1)
                        .required(false),
                )
                .arg_required_else_help(false),
        )
        .subcommand(
            Command::new("profile")
                .about("Manages profiles")
                .subcommand(
                    Command::new("create")
                        .about("Creates a new profile")
                        .arg(
                            Arg::new("name")
                                .help("The name of the profile to create")
                                .num_args(1)
                                .required(true),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("delete")
                        .about("Deletes a profile")
                        .arg(
                            Arg::new("name")
                                .help("The name of the profile to delete")
                                .num_args(1)
                                .required(true),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("use")
                        .about("Switches to a profile")
                        .arg(
                            Arg::new("name")
                                .help("The name of the profile to switch to")
                                .num_args(1)
                                .required(true),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("list")
                        .about("Lists all profiles")
                        .arg(
                            Arg::new("include_version")
                                .long("version")
                                .short('v')
                                .help("Include the version of each profile")
                                .num_args(0)
                                .default_value("false")
                                .required(false),
                        )
                        .arg_required_else_help(false),
                )
                .arg_required_else_help(true)
                .subcommand_required(true),
        )
        .subcommand(
            Command::new("backup")
                .about("Manages backups")
                .subcommand(
                    Command::new("create")
                        .about("Creates a new backup")
                        .arg(
                            Arg::new("name")
                                .help("The name of the backup to create")
                                .num_args(1)
                                .required(true),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("delete")
                        .about("Deletes a backup")
                        .arg(
                            Arg::new("name")
                                .help("The name of the backup to delete")
                                .num_args(1)
                                .required(true),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("restore")
                        .about("Restores a backup")
                        .arg(
                            Arg::new("name")
                                .help("The name of the backup to restore")
                                .num_args(1)
                                .required(true),
                        )
                        .arg_required_else_help(true),
                )
                .subcommand(
                    Command::new("list")
                        .about("Lists all backups")
                        .arg_required_else_help(false),
                )
                .arg_required_else_help(true)
                .subcommand_required(true),
        )
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("upgrade", sub_matches)) => {
            std::process::exit(commands::upgrade::upgrade(sub_matches).await)
        }
        Some(("init", sub_matches)) => {
            std::process::exit(commands::init::init(sub_matches, None, None).await)
        }
        Some(("install", sub_matches)) => {
            std::process::exit(commands::install::install(sub_matches).await)
        }
        Some(("start", sub_matches)) => {
            std::process::exit(commands::start::start(sub_matches).await)
        }
        Some(("lookup", sub_matches)) => {
            std::process::exit(commands::lookup::lookup(sub_matches).await)
        }
        Some(("version", sub_matches)) => {
            std::process::exit(commands::version::version(sub_matches).await)
        }
        Some(("update", sub_matches)) => {
            std::process::exit(commands::update::update(sub_matches).await)
        }
        Some(("profile", sub_matches)) => match sub_matches.subcommand() {
            Some(("create", sub_matches)) => {
                std::process::exit(commands::profile::create::create(sub_matches).await)
            }
            Some(("delete", sub_matches)) => {
                std::process::exit(commands::profile::delete::delete(sub_matches).await)
            }
            Some(("use", sub_matches)) => {
                std::process::exit(commands::profile::r#use::r#use(sub_matches).await)
            }
            Some(("list", sub_matches)) => {
                std::process::exit(commands::profile::list::list(sub_matches).await)
            }
            _ => unreachable!(),
        },
        Some(("backup", sub_matches)) => match sub_matches.subcommand() {
            Some(("create", sub_matches)) => {
                std::process::exit(commands::backups::create::create(sub_matches).await)
            }
            Some(("delete", sub_matches)) => {
                std::process::exit(commands::backups::delete::delete(sub_matches).await)
            }
            Some(("restore", sub_matches)) => {
                std::process::exit(commands::backups::restore::restore(sub_matches).await)
            }
            Some(("list", sub_matches)) => {
                std::process::exit(commands::backups::list::list(sub_matches).await)
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
