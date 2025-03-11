use crate::api::Progress;
use crate::{config, detached, java};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use human_bytes::human_bytes;
use rand::{Rng, distr::Alphanumeric};
use std::path::Path;
use std::sync::Arc;
use std::{fs::File, io::Write};
use tokio::sync::Mutex;
use tokio::{process::Command, signal::ctrl_c};

pub async fn start(matches: &ArgMatches) -> i32 {
    let mut config = config::Config::new(".mcvcli.json", false);
    let auto_agree_eula = matches.get_one::<bool>("eula").expect("required");
    let detached = matches.get_one::<bool>("detached").expect("required");

    let eula_accepted = std::fs::read_to_string("eula.txt")
        .unwrap_or_default()
        .contains("eula=true");

    if !eula_accepted {
        if !auto_agree_eula {
            let accept_eula = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you accept the Minecraft EULA? (https://minecraft.net/eula)")
                .default(false)
                .interact()
                .unwrap();

            if !accept_eula {
                return 1;
            }
        }

        std::fs::write("eula.txt", "eula=true\n").unwrap();
    }

    if detached::status(config.pid) {
        println!(
            "{} {}",
            "server is already running, use".red(),
            "mcvcli attach".cyan()
        );
        return 1;
    }

    let java = java::Java::new();
    let [binary, java_home] = java.binary(config.java_version).await;
    let command = format!(
        "{} {} -Xmx{}M -jar {} nogui {}",
        binary,
        config.extra_flags.join(" "),
        config.ram_mb,
        config.jar_file,
        config.extra_args.join(" ")
    );

    if !Path::new(&config.jar_file).exists() {
        if Path::new("libraries/net/minecraftforge/forge").exists() {
            println!("{}", "downloading forge wrapper jar...".bright_black());

            let mut req = reqwest::get("https://s3.mcjars.app/forge/ForgeServerJAR.jar")
                .await
                .unwrap();
            let mut file = File::create(&config.jar_file).unwrap();

            let mut progress = Progress::new(req.content_length().unwrap() as usize);
            progress.spinner(|progress, spinner| {
                format!(
                    "\r {} {} {}/{} ({}%)      ",
                    "downloading forge wrapper jar...".bright_black().italic(),
                    spinner.cyan(),
                    human_bytes(progress.progress() as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    human_bytes(progress.total as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    progress.percent().round().to_string().cyan().italic()
                )
            });

            while let Some(chunk) = req.chunk().await.unwrap() {
                file.write_all(&chunk).unwrap();
                progress.incr(chunk.len());
            }

            file.sync_all().unwrap();
            progress.finish();
            println!();

            println!(
                "{} {}",
                "downloading forge wrapper jar...".bright_black().italic(),
                "DONE".green().bold().italic()
            );
        } else if Path::new("libraries/net/neoforged/neoforge").exists() {
            println!("{}", "downloading neoforge wrapper jar...".bright_black());

            let mut req = reqwest::get("https://s3.mcjars.app/neoforge/NeoForgeServerJAR.jar")
                .await
                .unwrap();
            let mut file = File::create(&config.jar_file).unwrap();

            let mut progress = Progress::new(req.content_length().unwrap() as usize);
            progress.spinner(|progress, spinner| {
                format!(
                    "\r {} {} {}/{} ({}%)      ",
                    "downloading neoforge wrapper jar..."
                        .bright_black()
                        .italic(),
                    spinner.cyan(),
                    human_bytes(progress.progress() as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    human_bytes(progress.total as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    progress.percent().round().to_string().cyan().italic()
                )
            });

            while let Some(chunk) = req.chunk().await.unwrap() {
                file.write_all(&chunk).unwrap();
                progress.incr(chunk.len());
            }

            file.sync_all().unwrap();

            progress.finish();
            println!();

            println!(
                "{} {}",
                "downloading neoforge wrapper jar..."
                    .bright_black()
                    .italic(),
                "DONE".green().bold().italic()
            );
        } else {
            println!("{}", "no server jar found".red());
            return 1;
        }
    }

    println!();
    println!("{}", "starting the minecraft server...".yellow());
    println!("{}", command);

    if !*detached {
        let child = Arc::new(Mutex::new(
            Command::new(binary)
                .args(config.extra_flags)
                .arg(format!("-Xmx{}M", config.ram_mb))
                .arg("-jar")
                .arg(config.jar_file)
                .arg("nogui")
                .args(config.extra_args)
                .env("JAVA_HOME", java_home)
                .spawn()
                .unwrap(),
        ));

        let child_clone = Arc::clone(&child);
        tokio::spawn(async move {
            ctrl_c().await.unwrap();
            let mut child = child_clone.lock().await;

            child.start_kill().unwrap_or(());
        });

        let code = child.lock().await.wait().await.unwrap();

        println!();
        println!(
            "{} {}",
            "server has stopped with code".red(),
            code.code().unwrap_or(0)
        );
    } else {
        if std::env::consts::OS == "windows" {
            println!(
                "{}",
                "detached mode is currently not supported on windows".red()
            );
            return 1;
        }

        config.identifier = Some(
            rand::rng()
                .sample_iter(&Alphanumeric)
                .take(7)
                .map(char::from)
                .collect(),
        );

        let [stdin, stdout, stderr] = detached::get_pipes(config.identifier.as_ref().unwrap());

        #[allow(clippy::zombie_processes)]
        let child = std::process::Command::new(binary)
            .args(&config.extra_flags)
            .arg(format!("-Xmx{}M", config.ram_mb))
            .arg("-jar")
            .arg(&config.jar_file)
            .arg("nogui")
            .args(&config.extra_args)
            .env("JAVA_HOME", java_home)
            .stdin(File::open(stdin.path()).unwrap())
            .stdout(File::create(stdout.path()).unwrap())
            .stderr(File::create(stderr.path()).unwrap())
            .spawn()
            .unwrap();

        config.pid = Some(child.id() as usize);
        config.save();

        println!("{}", "server has started in detached mode".green());
    }

    0
}
