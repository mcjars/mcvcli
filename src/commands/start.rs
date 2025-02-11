use crate::api::Progress;
use crate::{config, java};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use human_bytes::human_bytes;
use std::path::Path;
use std::sync::Arc;
use std::{
    fs::File,
    io::{Read, Write},
};
use tokio::sync::Mutex;
use tokio::{process::Command, signal::ctrl_c};

pub async fn start(matches: &ArgMatches) -> i32 {
    let config = config::Config::new(".mcvcli.json", false);

    let auto_agree_eula = matches.get_one::<bool>("eula").expect("required");
    let mut eula_file: Option<File> = File::open("eula.txt").ok();
    let mut eula_accepted = false;

    if eula_file.is_none() {
        eula_accepted = false;
    } else {
        let mut eula_contents = String::new();
        eula_file
            .as_mut()
            .unwrap()
            .read_to_string(&mut eula_contents)
            .unwrap();

        if eula_contents.contains("eula=true") {
            eula_accepted = true;
        }
    }

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

        eula_file = File::create("eula.txt").ok();
        eula_file
            .as_mut()
            .unwrap()
            .write_all("eula=true".as_bytes())
            .unwrap();
        eula_file.as_mut().unwrap().sync_all().unwrap();
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

            let mut progress = Progress {
                file_count: req.content_length().unwrap(),
                file_current: 0,
            };

            while let Some(chunk) = req.chunk().await.unwrap() {
                file.write_all(&chunk).unwrap();

                progress.file_current += chunk.len() as u64;
                eprint!(
                    "\r {} {}/{} ({}%)      ",
                    "downloading forge wrapper jar...".bright_black().italic(),
                    human_bytes(progress.file_current as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    human_bytes(progress.file_count as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    ((progress.file_current as f64 / progress.file_count as f64) * 100.0)
                        .round()
                        .to_string()
                        .cyan()
                        .italic()
                );
            }

            file.sync_all().unwrap();

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

            let mut progress = Progress {
                file_count: req.content_length().unwrap(),
                file_current: 0,
            };

            while let Some(chunk) = req.chunk().await.unwrap() {
                file.write_all(&chunk).unwrap();

                progress.file_current += chunk.len() as u64;
                eprint!(
                    "\r {} {}/{} ({}%)      ",
                    "downloading neoforge wrapper jar..."
                        .bright_black()
                        .italic(),
                    human_bytes(progress.file_current as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    human_bytes(progress.file_count as f64)
                        .to_string()
                        .cyan()
                        .italic(),
                    ((progress.file_current as f64 / progress.file_count as f64) * 100.0)
                        .round()
                        .to_string()
                        .cyan()
                        .italic()
                );
            }

            file.sync_all().unwrap();

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

    let child = Arc::new(Mutex::new(
        Command::new(binary)
            .args(config.extra_flags)
            .arg(format!("-Xmx{}M", config.ram_mb))
            .arg("-jar")
            .arg(config.jar_file)
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

    0
}
