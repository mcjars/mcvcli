use chrono::{DateTime, Local};
use serde::Deserialize;
use std::path::Path;

mod counting_reader;
mod tar;
mod zip;

#[derive(Debug, Deserialize)]
pub enum BackupFormat {
    #[serde(rename = "zip")]
    Zip,
    #[serde(rename = "tar")]
    Tar,
    #[serde(rename = "tar.gz")]
    TarGz,
    #[serde(rename = "tar.xz")]
    TarXz,
}

#[derive(Debug)]
pub struct Backup {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub format: BackupFormat,

    pub created: DateTime<Local>,
}

pub fn list() -> Vec<Backup> {
    let mut backups = Vec::new();

    if let Ok(entries) = std::fs::read_dir(".mcvcli.backups") {
        for entry in entries.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Ok(metadata) = path.metadata() else {
                continue;
            };

            let (stripped, format) = if name.ends_with("tar.gz") {
                (name.replacen(".tar.gz", "", 1), BackupFormat::TarGz)
            } else if name.ends_with("tar.xz") {
                (name.replacen(".tar.xz", "", 1), BackupFormat::TarXz)
            } else if name.ends_with("tar") {
                (name.replacen(".tar", "", 1), BackupFormat::Tar)
            } else if name.ends_with("zip") {
                (name.replacen(".zip", "", 1), BackupFormat::Zip)
            } else {
                continue;
            };

            let created = metadata
                .created()
                .map(DateTime::from)
                .unwrap_or_else(|_| Local::now());

            backups.push(Backup {
                name: stripped,
                path: path.to_string_lossy().to_string(),
                size: metadata.len(),
                format,
                created,
            });
        }
    }

    backups.sort_by_key(|b| std::cmp::Reverse(b.created));

    backups
}

pub fn extension(format: &BackupFormat) -> String {
    match format {
        BackupFormat::Zip => String::from("zip"),
        BackupFormat::Tar => String::from("tar"),
        BackupFormat::TarGz => String::from("tar.gz"),
        BackupFormat::TarXz => String::from("tar.xz"),
    }
}

pub fn create(name: &str, format: &BackupFormat) -> Result<(), anyhow::Error> {
    if !Path::new(".mcvcli.backups").exists() {
        std::fs::create_dir_all(".mcvcli.backups")?;
    }

    match format {
        BackupFormat::Zip => zip::create(name),
        BackupFormat::Tar => tar::create(name, tar::TarEncoder::Tar, "tar"),
        BackupFormat::TarGz => tar::create(name, tar::TarEncoder::Gz, "tar.gz"),
        BackupFormat::TarXz => tar::create(name, tar::TarEncoder::Xz, "tar.xz"),
    }
}

pub fn restore(backup: &Backup) -> Result<(), anyhow::Error> {
    let path = &backup.path;

    match backup.format {
        BackupFormat::Zip => zip::restore(path),
        BackupFormat::Tar => tar::restore(path, tar::TarEncoder::Tar),
        BackupFormat::TarGz => tar::restore(path, tar::TarEncoder::Gz),
        BackupFormat::TarXz => tar::restore(path, tar::TarEncoder::Xz),
    }
}
