use crate::api::{self, Progress};
use crate::{config, detached, java};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use human_bytes::human_bytes;
use std::{fs::File, io::Write, path::Path};
use tokio::io::AsyncReadExt;
use tokio::{io::AsyncWriteExt, process::Command, sync::Mutex};

pub async fn start(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let config = config::Config::new(".mcvcli.json", false);
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
                .interact()?;

            if !accept_eula {
                return Ok(1);
            }
        }

        std::fs::write("eula.txt", "eula=true\n")?;
    }

    if detached::is_running() {
        println!(
            "{} {}",
            "server is already running, use".red(),
            "mcvcli attach".cyan()
        );
        return Ok(1);
    }

    let [binary, java_home] = java::binary(config.java_version).await?;
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
                .await?;
            let mut file = File::create(&config.jar_file)?;

            let mut progress = Progress::new(req.content_length().unwrap_or(0) as usize);
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

            while let Some(chunk) = req.chunk().await? {
                file.write_all(&chunk)?;
                progress.incr(chunk.len());
            }

            file.sync_all()?;
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
                .await?;
            let mut file = File::create(&config.jar_file)?;

            let mut progress = Progress::new(req.content_length().unwrap_or(0) as usize);
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
                file.write_all(&chunk)?;
                progress.incr(chunk.len());
            }

            file.sync_all()?;

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
            return Ok(1);
        }
    }

    println!();
    println!("{}", "starting the minecraft server...".yellow());
    println!("{command}");

    if !detached {
        let mut child = {
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

            // Process group 0 to make sure child processes are also killed
            #[cfg(unix)]
            command.process_group(0);

            command.spawn()?
        };

        let child_kill_notifier = tokio::sync::Notify::new();
        let child_stdin = Mutex::new(
            child
                .stdin
                .take()
                .ok_or_else(|| anyhow::anyhow!("failed to capture server stdin"))?,
        );

        let stop_future = async {
            let _ = tokio::signal::ctrl_c().await;

            println!();
            println!();
            println!(
                "{}",
                format!("stopping server ({timeout}s before being killed) ...").bright_black()
            );
            println!();

            let _ = child_stdin
                .lock()
                .await
                .write_all((config.stop_command + "\n").as_bytes())
                .await;

            tokio::time::sleep(tokio::time::Duration::from_secs(timeout)).await;

            println!(
                "{}",
                "server is taking too long to stop, killing it ...".bright_black()
            );

            child_kill_notifier.notify_waiters();

            println!(
                "{} {}",
                "server is taking too long to stop, killing it ...".bright_black(),
                "DONE".green().bold()
            );
        };

        let stdin_future = async {
            let mut buffer = [0; 1024];
            let mut stdin = tokio::io::stdin();

            loop {
                match stdin.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let _ = child_stdin.lock().await.write_all(&buffer[..n]).await;
                    }
                    Err(_) => break,
                }
            }
        };

        let code_future = async {
            tokio::select! {
                _ = child_kill_notifier.notified() => {
                    let _ = child.kill().await;
                    0
                },
                status = child.wait() => {
                    status.ok().and_then(|status| status.code()).unwrap_or(0)
                }
            }
        };

        let code = tokio::select! {
            _ = stop_future => {
                child.wait().await.ok().and_then(|status| status.code()).unwrap_or(0)
            },
            _ = stdin_future => {
                child.wait().await.ok().and_then(|status| status.code()).unwrap_or(0)
            },
            code = code_future => {
                code
            }
        };

        println!();
        println!(
            "{} {}",
            "stopping server ...".bright_black(),
            "DONE".green().bold()
        );

        println!("{} {}", "server has stopped with code".red(), code);
    } else {
        // Hand the server off to a detached `mcvcli daemon` supervisor. The daemon owns java,
        // continuously drains its output (so the pipe never fills) and exposes a control socket
        // that `attach`/`stop` connect to.
        detached::write_spec(&detached::Spec {
            binary,
            java_home,
            jar_file: config.jar_file.clone(),
            ram_mb: config.ram_mb,
            extra_flags: config.extra_flags.clone(),
            extra_args: config.extra_args.clone(),
            stop_command: config.stop_command.clone(),
            log_max_bytes: config.detached_log_max_mb.saturating_mul(1024 * 1024),
        })?;

        detached::spawn_daemon()?;

        // Wait for the daemon to come up and report its state.
        let mut started = false;
        for _ in 0..50 {
            if detached::is_running() {
                started = true;
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        if !started {
            println!(
                "{}",
                "failed to start detached server, check .mcvcli.detached/latest.log".red()
            );
            return Ok(1);
        }

        println!(
            "{} {}",
            "server has started in detached mode, use".green(),
            "mcvcli attach".cyan()
        );
    }

    Ok(0)
}
