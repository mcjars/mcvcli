use crate::detached;

use clap::ArgMatches;
use colored::Colorize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn attach(_matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    if !detached::is_running() {
        println!(
            "{} {}",
            "server is not running, use".red(),
            "mcvcli start --detached".cyan()
        );
        return Ok(1);
    }

    println!("{}", "attaching to server ...".bright_black());

    let connection = match detached::connect().await {
        Ok(connection) => connection,
        Err(_) => {
            println!(
                "{} {}",
                "the server is orphaned (its supervisor is gone), use".red(),
                "mcvcli stop".cyan()
            );
            return Ok(1);
        }
    };
    let (mut reader, mut writer) = tokio::io::split(connection);

    detached::write_frame(&mut writer, detached::TAG_ATTACH, &[]).await?;

    println!(
        "{} {}",
        "attaching to server ...".bright_black(),
        "DONE".green().bold()
    );
    println!(
        "{}",
        "(press ctrl-c to detach without stopping)".bright_black()
    );
    println!();

    // Forward terminal stdin to the server.
    let stdin_task = tokio::spawn(async move {
        let mut stdin = tokio::io::stdin();
        let mut buffer = [0u8; 1024];

        loop {
            match stdin.read(&mut buffer).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if detached::write_frame(&mut writer, detached::TAG_STDIN, &buffer[..n])
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    let stream_future = async {
        let mut stdout = tokio::io::stdout();
        let mut stderr = tokio::io::stderr();

        loop {
            match detached::read_frame(&mut reader).await {
                Ok((detached::TAG_STDOUT, data)) => {
                    let _ = stdout.write_all(&data).await;
                    let _ = stdout.flush().await;
                }
                Ok((detached::TAG_STDERR, data)) => {
                    let _ = stderr.write_all(&data).await;
                    let _ = stderr.flush().await;
                }
                Ok((detached::TAG_EXIT, data)) => {
                    let code = data
                        .as_slice()
                        .try_into()
                        .map(i32::from_be_bytes)
                        .unwrap_or(0);
                    return Some(code);
                }
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    };

    let result = tokio::select! {
        code = stream_future => code,
        _ = tokio::signal::ctrl_c() => {
            stdin_task.abort();
            println!();
            println!("{}", "detached (server is still running)".bright_black());
            return Ok(0);
        }
    };

    stdin_task.abort();

    match result {
        Some(code) => {
            println!();
            println!("{} {}", "server has stopped with code".red(), code);
        }
        None => {
            println!();
            println!("{}", "disconnected from server".red());
        }
    }

    Ok(0)
}
