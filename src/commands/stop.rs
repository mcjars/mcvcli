use crate::detached;

use clap::ArgMatches;
use colored::Colorize;
use tokio::io::AsyncWriteExt;

/// Best-effort kill of the server and supervisor when the control socket is unreachable (a wedged
/// or already-killed daemon, the latter having orphaned the java child).
async fn force_kill() {
    let Some(state) = detached::read_state() else {
        detached::cleanup();
        return;
    };

    let pids = [state.java_pid, state.daemon_pid];

    {
        let sys = sysinfo::System::new_all();
        for pid in pids {
            if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
                process.kill_with(sysinfo::Signal::Kill);
            }
        }
    }

    // Wait for the kills to actually land before clearing state, so a follow-up `start` doesn't
    // race a not-yet-dead server still holding the port.
    for _ in 0..50 {
        let sys = sysinfo::System::new_all();
        let alive = pids
            .iter()
            .any(|pid| sys.process(sysinfo::Pid::from(*pid as usize)).is_some());
        if !alive {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    detached::cleanup();
}

pub async fn stop(matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    let timeout = *matches.get_one::<u64>("timeout").expect("required");

    if !detached::is_running() {
        println!(
            "{} {}",
            "server is not running, use".red(),
            "mcvcli start --detached".cyan()
        );
        return Ok(1);
    }

    let connection = match detached::connect().await {
        Ok(connection) => connection,
        Err(_) => {
            println!(
                "{}",
                "daemon is unreachable, killing the server ...".bright_black()
            );
            force_kill().await;
            println!(
                "{} {}",
                "stopping server ...".bright_black(),
                "DONE".green().bold()
            );
            return Ok(0);
        }
    };

    let (mut reader, mut writer) = tokio::io::split(connection);

    println!(
        "{}",
        format!("stopping server ({timeout}s before being killed) ...").bright_black()
    );

    detached::write_frame(&mut writer, detached::TAG_STOP, &timeout.to_be_bytes()).await?;

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
            Ok((detached::TAG_EXIT, _)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    // Wait for the daemon to finish tearing itself down so a follow-up `start` sees a clean slate.
    for _ in 0..50 {
        if !detached::is_running() {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    println!();
    println!(
        "{} {}",
        "stopping server ...".bright_black(),
        "DONE".green().bold()
    );

    Ok(0)
}
