use crate::api::{self, Progress};

use colored::Colorize;
use dirs::home_dir;
use flate2::read::GzDecoder;
use human_bytes::human_bytes;
use serde::Deserialize;
use std::{fs::File, io::Write, path::Path, sync::LazyLock};
use tar::Archive as TarArchive;
use zip::ZipArchive;

#[derive(Deserialize)]
struct Binary {
    image_type: String,
    package: Package,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    link: String,
}

static LOCATION: LazyLock<String> =
    LazyLock::new(|| format!("{}/.mcvcli/java", home_dir().unwrap().to_str().unwrap()));

fn parse_version_stderr(stderr: &str) -> Option<u8> {
    let line = stderr.lines().next()?;
    let version = line
        .split_whitespace()
        .find(|s| s.chars().any(|c| c.is_numeric()))?
        .trim_matches(|c: char| !c.is_numeric())
        .replace("1.8", "8");

    atoi::atoi(version.as_bytes())
}

pub fn installed() -> Vec<(u8, String)> {
    let mut installed: Vec<(u8, String)> = Vec::new();

    if !Path::new(LOCATION.as_str()).exists() {
        return installed;
    }

    let entries = match std::fs::read_dir(LOCATION.as_str()) {
        Ok(entries) => entries,
        Err(_) => return installed,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() && std::fs::exists(path.join("bin")).unwrap_or_default() {
            let version = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .parse()
                .unwrap_or_default();

            if version != 0 {
                installed.push((version, path.to_str().unwrap().to_string()));
            }
        }
    }

    installed.sort();

    installed.into_iter().rev().collect()
}

pub fn remove(version: u8) {
    let installed = installed();

    if let Some((_, path)) = installed.iter().find(|(v, _)| *v == version) {
        std::fs::remove_dir_all(path).unwrap();
    }
}

pub fn find_local() -> Option<(u8, String, String)> {
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let binary = format!("{java_home}/bin/java");
        let version = std::process::Command::new(&binary)
            .arg("-version")
            .output()
            .ok()?
            .stderr;

        if let Ok(version) = String::from_utf8(version) {
            let version = parse_version_stderr(&version)?;

            return Some((version, binary, java_home));
        }
    } else if let Ok(path) = std::env::var("PATH") {
        for path in path.split(':') {
            let binary = format!("{path}/java");
            if !Path::new(&binary).exists() {
                continue;
            }

            let version = std::process::Command::new(&binary)
                .arg("-version")
                .output()
                .ok()?
                .stderr;

            if let Ok(version) = String::from_utf8(version) {
                let version = parse_version_stderr(&version)?;

                return Some((version, binary, "".to_string()));
            }
        }
    }

    None
}

pub async fn binary(version: u8) -> [String; 2] {
    println!(
        "{} {} {}",
        "checking for java".bright_black(),
        version.to_string().cyan(),
        "...".bright_black()
    );

    let installed = installed();
    let local = find_local();

    if let Some((v, path, root)) = local {
        if v == version {
            println!(
                "{} {} {} {}",
                "checking for java".bright_black(),
                version.to_string().cyan(),
                "...".bright_black(),
                "DONE".green().bold()
            );

            return [path, root];
        }
    }

    if !installed.iter().any(|(v, _)| *v == version) {
        println!(
            "{} {} {}",
            "java".bright_black(),
            version.to_string().cyan(),
            "not found, installing...".bright_black()
        );

        install(version).await.unwrap();

        println!(
            "{} {} {} {}",
            "java".bright_black(),
            version.to_string().cyan(),
            "not found, installing...".bright_black(),
            "DONE".green().bold()
        );
    }

    println!(
        "{} {} {} {}",
        "checking for java".bright_black(),
        version.to_string().cyan(),
        "...".bright_black(),
        "DONE".green().bold()
    );

    [
        format!("{}/{}/bin/java", *LOCATION, version),
        format!("{}/{}", *LOCATION, version),
    ]
}

pub async fn install(version: u8) -> Result<(), anyhow::Error> {
    let query_arch = std::env::consts::ARCH;
    let query_os = match std::env::consts::OS {
        "macos" => "mac",
        "windows" => "windows",
        _ => "linux",
    };

    println!(
        " {} {} {}",
        "downloading java".bright_black().italic(),
        version.to_string().cyan().italic(),
        "...".bright_black().italic()
    );

    let data: Vec<ApiResponse> = api::CLIENT
        .get(format!(
            "https://api.adoptium.net/v3/assets/latest/{version}/hotspot?os={query_os}&architecture={query_arch}"
        ))
        .send()
        .await?.json().await?;

    #[derive(Deserialize)]
    struct ApiResponse {
        binary: Binary,
    }

    let binary = data.into_iter().find(|binary| {
        binary.binary.image_type == "jdk"
            && (binary.binary.package.name.ends_with("tar.gz")
                || binary.binary.package.name.ends_with("zip"))
    });

    let binary = match binary {
        Some(binary) => binary,
        None => return Err(anyhow::anyhow!("no binary found for Java {version}")),
    };

    let destination = format!("{}/{}/java.archive", *LOCATION, version);

    std::fs::create_dir_all(format!("{}/{}", *LOCATION, version))?;

    let mut res = api::CLIENT.get(&binary.binary.package.link).send().await?;
    let mut file = File::create(&destination)?;

    let mut progress = Progress::new(res.content_length().unwrap_or_default() as usize);
    progress.spinner(|progress, spinner| {
        format!(
            "\r  {} {} {}/{} ({}%)      ",
            "downloading...".bright_black().italic(),
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

    while let Ok(Some(chunk)) = res.chunk().await {
        file.write_all(&chunk)?;
        progress.incr(chunk.len());
    }

    file.sync_all()?;
    progress.finish();
    println!();

    println!(
        " {} {} {} {}",
        "downloading java".bright_black().italic(),
        version.to_string().cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );
    println!(
        " {} {} {}",
        "extracting java".bright_black().italic(),
        version.to_string().cyan().italic(),
        "...".bright_black().italic()
    );

    if binary.binary.package.name.ends_with(".zip") {
        let mut archive = ZipArchive::new(File::open(&destination).unwrap()).unwrap();
        archive.extract(format!("{}/{}", *LOCATION, version))?;
    } else {
        let mut archive = TarArchive::new(GzDecoder::new(File::open(&destination).unwrap()));
        archive.unpack(format!("{}/{}", *LOCATION, version))?;
    }

    std::fs::remove_file(&destination)?;

    let entries = std::fs::read_dir(format!("{}/{}", *LOCATION, version))?;
    if entries.count() == 1 {
        let entry = std::fs::read_dir(format!("{}/{}", *LOCATION, version))
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let path = entry.path();

        for file in std::fs::read_dir(&path)?.flatten() {
            let file_path = file.path();

            std::fs::rename(
                &file_path,
                format!(
                    "{}/{}/{}",
                    *LOCATION,
                    version,
                    file_path.file_name().unwrap().to_str().unwrap()
                ),
            )?;
        }

        std::fs::remove_dir_all(&path)?;
    }

    println!(
        " {} {} {} {}",
        "extracting java".bright_black().italic(),
        version.to_string().cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    Ok(())
}

pub async fn versions() -> Vec<u8> {
    let res = api::CLIENT
        .get("https://api.adoptium.net/v3/info/available_releases")
        .send()
        .await
        .unwrap();
    let data = res.json::<ApiResponse>().await.unwrap();

    #[derive(Deserialize)]
    struct ApiResponse {
        available_releases: Vec<u8>,
    }

    data.available_releases
}
