use crate::api::{self, Progress};
use crate::{config, detached, java};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use human_bytes::human_bytes;
use rand::{Rng, distr::Alphanumeric};
use std::{fs::File, io::Read, io::Write, path::Path, sync::Arc};
use tokio::{io::AsyncWriteExt, process::Command, sync::Mutex};

pub async fn start(matches: &ArgMatches) -> i32 {
    let mut config = config::Config::new(".mcvcli.json", false);
    let auto_agree_eula = *matches.get_one::<bool>("eula").expect("required");
    let detached = *matches.get_one::<bool>("detached").expect("required");
    let timeout = *matches.get_one::<u64>("timeout").expect("required");

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

    let [binary, java_home] = java::binary(config.java_version).await;
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

            let mut req = api::CLIENT
                .get("https://s3.mcjars.app/forge/ForgeServerJAR.jar")
                .send()
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

            let mut req = api::CLIENT
                .get("https://s3.mcjars.app/neoforge/NeoForgeServerJAR.jar")
                .send()
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

            while let Ok(Some(chunk)) = req.chunk().await {
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

    if !detached {
        let child = Arc::new(Mutex::new({
            let mut command = Command::new(binary);

            command.args(config.extra_flags);
            command.arg(format!("-Xmx{}M", config.ram_mb));
            command.arg("-jar");
            command.arg(config.jar_file);
            command.arg("nogui");
            command.args(config.extra_args);
            command.env("JAVA_HOME", java_home);
            command.stdin(std::process::Stdio::piped());
            command.stdout(std::process::Stdio::inherit());
            command.stderr(std::process::Stdio::inherit());
            command.kill_on_drop(true);

            #[cfg(unix)]
            command.process_group(0);

            command.spawn().unwrap()
        }));

        let kill = Arc::new(Mutex::new(None));
        tokio::spawn({
            let child = Arc::clone(&child);
            let kill = Arc::clone(&kill);
            let stop_command = config.stop_command.clone();

            async move {
                tokio::signal::ctrl_c().await.unwrap();

                println!();
                println!();
                println!(
                    "{}",
                    format!("stopping server ({}s before being killed) ...", timeout)
                        .bright_black()
                );
                println!();

                if let Some(stdin) = child.lock().await.stdin.as_mut() {
                    stdin
                        .write_all((stop_command + "\n").as_bytes())
                        .await
                        .unwrap();
                }

                let child = Arc::clone(&child);
                *kill.lock().await = Some(tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;

                    println!(
                        "{}",
                        "server is taking too long to stop, killing it ...".bright_black()
                    );

                    child.lock().await.kill().await.unwrap();

                    println!(
                        "{} {}",
                        "server is taking too long to stop, killing it ...".bright_black(),
                        "DONE".green().bold()
                    );
                }));
            }
        });

        tokio::spawn({
            let child = Arc::clone(&child);
            let mut stdin = std::io::stdin();

            async move {
                let mut buffer = [0; 1024];

                loop {
                    match stdin.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let mut child = child.lock().await;
                            if let Some(stdin) = child.stdin.as_mut() {
                                stdin.write_all(&buffer[..n]).await.unwrap();
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
        });

        let code = {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                let mut child = child.lock().await;
                match child.try_wait() {
                    Ok(Some(code)) => break code,
                    Ok(None) => continue,
                    Err(_) => break Default::default(),
                }
            }
        };

        if let Some(kill) = kill.lock().await.take() {
            kill.abort();
        }

        println!();
        println!(
            "{} {}",
            "stopping server ...".bright_black(),
            "DONE".green().bold()
        );

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
