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
            let name = path.file_name().unwrap().to_str().unwrap();

            if !path.is_file() {
                continue;
            }

            if name.ends_with("zip") {
                backups.push(Backup {
                    name: name.to_string().replacen(".zip", "", 1),
                    path: path.to_str().unwrap().to_string(),
                    size: path.metadata().unwrap().len(),
                    format: BackupFormat::Zip,
                    created: DateTime::from(path.metadata().unwrap().created().unwrap()),
                });
            } else if name.ends_with("tar") {
                backups.push(Backup {
                    name: name.to_string().replacen(".tar", "", 1),
                    path: path.to_str().unwrap().to_string(),
                    size: path.metadata().unwrap().len(),
                    format: BackupFormat::Tar,
                    created: DateTime::from(path.metadata().unwrap().created().unwrap()),
                });
            } else if name.ends_with("tar.gz") {
                backups.push(Backup {
                    name: name.to_string().replacen(".tar.gz", "", 1),
                    path: path.to_str().unwrap().to_string(),
                    size: path.metadata().unwrap().len(),
                    format: BackupFormat::TarGz,
                    created: DateTime::from(path.metadata().unwrap().created().unwrap()),
                });
            } else if path.is_file() && name.ends_with("tar.xz") {
                backups.push(Backup {
                    name: name.to_string().replacen(".tar.xz", "", 1),
                    path: path.to_str().unwrap().to_string(),
                    size: path.metadata().unwrap().len(),
                    format: BackupFormat::TarXz,
                    created: DateTime::from(path.metadata().unwrap().created().unwrap()),
                });
            }
        }
    }

    backups.sort_by(|a, b| b.created.cmp(&a.created));

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

pub fn create(name: &str, format: &BackupFormat) {
    if !Path::new(".mcvcli.backups").exists() {
        std::fs::create_dir_all(".mcvcli.backups").unwrap();
    }

    match format {
        BackupFormat::Zip => zip::create(name),
        BackupFormat::Tar => tar::create(name, tar::TarEncoder::Tar, "tar"),
        BackupFormat::TarGz => tar::create(name, tar::TarEncoder::Gz, "tar.gz"),
        BackupFormat::TarXz => tar::create(name, tar::TarEncoder::Xz, "tar.xz"),
    }
}

pub fn restore(backup: &Backup) {
    let path = &backup.path;

    match backup.format {
        BackupFormat::Zip => zip::restore(path),
        BackupFormat::Tar => tar::restore(path, tar::TarEncoder::Tar),
        BackupFormat::TarGz => tar::restore(path, tar::TarEncoder::Gz),
        BackupFormat::TarXz => tar::restore(path, tar::TarEncoder::Xz),
    }
}
