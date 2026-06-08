use interprocess::local_socket::{
    GenericNamespaced, ToNsName,
    tokio::{Stream, prelude::*},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const DIR: &str = ".mcvcli.detached";

pub const TAG_STDIN: u8 = 1;
pub const TAG_STOP: u8 = 2;
pub const TAG_ATTACH: u8 = 3;

pub const TAG_STDOUT: u8 = 10;
pub const TAG_STDERR: u8 = 11;
pub const TAG_EXIT: u8 = 12;

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub identifier: String,
    pub daemon_pid: u32,
    pub java_pid: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Spec {
    pub binary: String,
    pub java_home: String,
    pub jar_file: String,
    pub ram_mb: u32,
    pub extra_flags: Vec<String>,
    pub extra_args: Vec<String>,
    pub stop_command: String,
    pub log_max_bytes: u64,
}

pub fn dir() -> &'static Path {
    Path::new(DIR)
}

pub fn state_path() -> PathBuf {
    dir().join("state.json")
}

pub fn spec_path() -> PathBuf {
    dir().join("daemon.json")
}

pub fn log_path() -> PathBuf {
    dir().join("latest.log")
}

pub fn log_old_path() -> PathBuf {
    dir().join("latest.log.old")
}

pub fn socket_label(identifier: &str) -> String {
    format!("mcvcli-{identifier}.sock")
}

pub fn read_state() -> Option<State> {
    let file = std::fs::File::open(state_path()).ok()?;
    serde_json::from_reader(file).ok()
}

pub fn write_state(state: &State) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(dir())?;
    let file = std::fs::File::create(state_path())?;
    serde_json::to_writer_pretty(file, state)?;

    Ok(())
}

pub fn read_spec() -> Result<Spec, anyhow::Error> {
    let file = std::fs::File::open(spec_path())?;

    Ok(serde_json::from_reader(file)?)
}

pub fn write_spec(spec: &Spec) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(dir())?;
    let file = std::fs::File::create(spec_path())?;
    serde_json::to_writer_pretty(file, spec)?;

    Ok(())
}

pub fn cleanup() {
    let _ = std::fs::remove_file(state_path());
    let _ = std::fs::remove_file(spec_path());
}

pub fn is_running() -> bool {
    let Some(state) = read_state() else {
        return false;
    };

    let sys = sysinfo::System::new_all();

    process_alive(&sys, state.daemon_pid, "mcvcli") || process_alive(&sys, state.java_pid, "java")
}

fn process_alive(sys: &sysinfo::System, pid: u32, name_hint: &str) -> bool {
    match sys.process(sysinfo::Pid::from(pid as usize)) {
        Some(process) => process
            .exe()
            .and_then(|exe| exe.file_stem())
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.contains(name_hint))
            .unwrap_or(true),
        None => false,
    }
}

pub async fn connect() -> Result<Stream, anyhow::Error> {
    let state = read_state().ok_or_else(|| anyhow::anyhow!("server is not running"))?;
    let label = socket_label(&state.identifier);
    let name = label.to_ns_name::<GenericNamespaced>()?;

    Ok(Stream::connect(name).await?)
}

pub fn spawn_daemon() -> Result<u32, anyhow::Error> {
    use std::process::{Command, Stdio};

    let exe = std::env::current_exe()?;
    let mut command = Command::new(exe);
    command
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        unsafe {
            command.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    }

    let child = command.spawn()?;

    Ok(child.id())
}

pub async fn write_frame<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    tag: u8,
    payload: &[u8],
) -> std::io::Result<()> {
    writer.write_u8(tag).await?;
    writer.write_u32(payload.len() as u32).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;

    Ok(())
}

pub async fn read_frame<R: AsyncReadExt + Unpin>(reader: &mut R) -> std::io::Result<(u8, Vec<u8>)> {
    let tag = reader.read_u8().await?;
    let len = reader.read_u32().await? as usize;
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;

    Ok((tag, payload))
}
