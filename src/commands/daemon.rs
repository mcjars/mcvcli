use crate::detached::{self, State};

use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName, tokio::prelude::*};
use rand::{RngExt, distr::Alphanumeric};
use std::{collections::VecDeque, fs::File, io::Write, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{Mutex, mpsc},
};

struct LogWriter {
    file: Option<File>,
    written: u64,
    cap: u64,
}

impl LogWriter {
    fn new(cap: u64) -> Self {
        let _ = std::fs::create_dir_all(detached::dir());
        let _ = std::fs::rename(detached::log_path(), detached::log_old_path());

        LogWriter {
            file: File::create(detached::log_path()).ok(),
            written: 0,
            cap,
        }
    }

    fn write(&mut self, data: &[u8]) {
        if self.cap > 0 && self.written.saturating_add(data.len() as u64) > self.cap {
            self.rotate();
        }

        if let Some(file) = self.file.as_mut()
            && file.write_all(data).is_ok()
        {
            self.written += data.len() as u64;
        }
    }

    fn rotate(&mut self) {
        self.file = None;
        let _ = std::fs::rename(detached::log_path(), detached::log_old_path());
        self.file = File::create(detached::log_path()).ok();
        self.written = 0;
    }
}

struct Shared {
    ring: VecDeque<(u8, Vec<u8>)>,
    ring_bytes: usize,
    ring_cap: usize,
    clients: Vec<mpsc::UnboundedSender<(u8, Vec<u8>)>>,
    log: LogWriter,
}

impl Shared {
    fn new(log_cap: u64) -> Self {
        Shared {
            ring: VecDeque::new(),
            ring_bytes: 0,
            ring_cap: 256 * 1024,
            clients: Vec::new(),
            log: LogWriter::new(log_cap),
        }
    }

    fn push(&mut self, tag: u8, data: Vec<u8>) {
        self.log.write(&data);

        self.clients
            .retain(|client| client.send((tag, data.clone())).is_ok());

        self.ring_bytes += data.len();
        self.ring.push_back((tag, data));
        while self.ring_bytes > self.ring_cap {
            match self.ring.pop_front() {
                Some((_, dropped)) => self.ring_bytes -= dropped.len(),
                None => break,
            }
        }
    }

    fn register(&mut self, client: mpsc::UnboundedSender<(u8, Vec<u8>)>) {
        for (tag, data) in &self.ring {
            let _ = client.send((*tag, data.clone()));
        }

        self.clients.push(client);
    }

    fn broadcast_exit(&mut self, code: i32) {
        let payload = code.to_be_bytes().to_vec();
        self.clients
            .retain(|client| client.send((detached::TAG_EXIT, payload.clone())).is_ok());
    }
}

pub async fn run() -> Result<i32, anyhow::Error> {
    let spec = detached::read_spec()?;

    let identifier: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();

    let mut child = tokio::process::Command::new(&spec.binary)
        .args(&spec.extra_flags)
        .arg(format!("-Xmx{}M", spec.ram_mb))
        .arg("-jar")
        .arg(&spec.jar_file)
        .arg("nogui")
        .args(&spec.extra_args)
        .env("JAVA_HOME", &spec.java_home)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let java_pid = child.id().unwrap_or(0);

    detached::write_state(&State {
        identifier: identifier.clone(),
        daemon_pid: std::process::id(),
        java_pid,
    })?;

    let shared = Arc::new(Mutex::new(Shared::new(spec.log_max_bytes)));

    let (stdin_tx, mut stdin_rx) = mpsc::unbounded_channel::<Vec<u8>>();
    if let Some(mut child_stdin) = child.stdin.take() {
        tokio::spawn(async move {
            while let Some(bytes) = stdin_rx.recv().await {
                if child_stdin.write_all(&bytes).await.is_err() {
                    break;
                }
                let _ = child_stdin.flush().await;
            }
        });
    }

    if let Some(mut stdout) = child.stdout.take() {
        let shared = Arc::clone(&shared);
        tokio::spawn(async move {
            let mut buffer = [0u8; 8192];
            loop {
                match stdout.read(&mut buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => shared
                        .lock()
                        .await
                        .push(detached::TAG_STDOUT, buffer[..n].to_vec()),
                }
            }
        });
    }
    if let Some(mut stderr) = child.stderr.take() {
        let shared = Arc::clone(&shared);
        tokio::spawn(async move {
            let mut buffer = [0u8; 8192];
            loop {
                match stderr.read(&mut buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => shared
                        .lock()
                        .await
                        .push(detached::TAG_STDERR, buffer[..n].to_vec()),
                }
            }
        });
    }

    let (control_tx, mut control_rx) = mpsc::unbounded_channel::<u64>();
    let name = detached::socket_label(&identifier).to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new().name(name).create_tokio()?;
    {
        let shared = Arc::clone(&shared);
        let stdin_tx = stdin_tx.clone();
        let control_tx = control_tx.clone();
        tokio::spawn(async move {
            loop {
                let connection = match listener.accept().await {
                    Ok(connection) => connection,
                    Err(_) => break,
                };

                let shared = Arc::clone(&shared);
                let stdin_tx = stdin_tx.clone();
                let control_tx = control_tx.clone();
                tokio::spawn(async move {
                    let (mut reader, mut writer) = tokio::io::split(connection);
                    let (tx, mut rx) = mpsc::unbounded_channel::<(u8, Vec<u8>)>();

                    shared.lock().await.register(tx);

                    let writer_task = tokio::spawn(async move {
                        while let Some((tag, payload)) = rx.recv().await {
                            if detached::write_frame(&mut writer, tag, &payload)
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    });

                    loop {
                        match detached::read_frame(&mut reader).await {
                            Ok((detached::TAG_STDIN, payload)) => {
                                let _ = stdin_tx.send(payload);
                            }
                            Ok((detached::TAG_STOP, payload)) => {
                                let timeout = payload
                                    .as_slice()
                                    .try_into()
                                    .map(u64::from_be_bytes)
                                    .unwrap_or(20);
                                let _ = control_tx.send(timeout);
                            }
                            Ok(_) => {}
                            Err(_) => break,
                        }
                    }

                    writer_task.abort();
                });
            }
        });
    }

    let exit_code = tokio::select! {
        status = child.wait() => status.ok().and_then(|status| status.code()).unwrap_or(0),
        Some(timeout) = control_rx.recv() => {
            let _ = stdin_tx.send(format!("{}\n", spec.stop_command).into_bytes());

            tokio::select! {
                status = child.wait() => status.ok().and_then(|status| status.code()).unwrap_or(0),
                _ = tokio::time::sleep(Duration::from_secs(timeout)) => {
                    let _ = child.kill().await;
                    child.wait().await.ok().and_then(|status| status.code()).unwrap_or(0)
                }
            }
        }
    };

    {
        let mut shared = shared.lock().await;
        shared
            .log
            .write(format!("\n[mcvcli] server exited with code {exit_code}\n").as_bytes());
        shared.broadcast_exit(exit_code);
    }

    tokio::time::sleep(Duration::from_millis(250)).await;

    detached::cleanup();

    Ok(exit_code)
}
