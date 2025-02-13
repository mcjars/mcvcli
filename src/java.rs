use crate::api::Progress;

use colored::Colorize;
use dirs::home_dir;
use flate2::read::GzDecoder;
use human_bytes::human_bytes;
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;
use std::{fs::File, io::Write, path::Path};
use tar::Archive as TarArchive;
use zip::ZipArchive;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

pub struct Java {
    location: String,
    client: Client,
}

impl Java {
    pub fn new() -> Self {
        let location = format!("{}/.mcvcli/java", home_dir().unwrap().to_str().unwrap());
        let client = ClientBuilder::new().user_agent(format!("mcvcli/{}", VERSION));

        Self {
            location,
            client: client.build().unwrap(),
        }
    }

    pub fn installed(&self) -> Vec<u8> {
        let mut installed: Vec<u8> = Vec::new();

        if !Path::new(&self.location).exists() {
            return installed;
        }

        let entries = std::fs::read_dir(&self.location).unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                let version =
                    str::parse::<u8>(path.file_name().unwrap().to_str().unwrap()).unwrap_or(0);

                if version != 0 {
                    installed.push(version);
                }
            }
        }

        installed
    }

    pub async fn binary(&self, version: u8) -> [String; 2] {
        println!(
            "{} {} {}",
            "checking for java".bright_black(),
            version.to_string().cyan(),
            "...".bright_black()
        );

        let installed = self.installed();

        if !installed.contains(&version) {
            println!(
                "{} {} {}",
                "java".bright_black(),
                version.to_string().cyan(),
                "not found, installing...".bright_black()
            );

            self.install(version).await;

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
            format!("{}/{}/bin/java", self.location, version),
            format!("{}/{}", self.location, version),
        ]
    }

    pub async fn install(&self, version: u8) {
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

        let res = self
            .client
            .get(format!(
                "https://api.adoptium.net/v3/assets/latest/{}/hotspot?os={}&architecture={}",
                version, query_os, query_arch
            ))
            .send()
            .await
            .unwrap();
        let data = res.json::<Vec<ApiResponse>>().await.unwrap();

        #[derive(Deserialize)]
        struct ApiResponse {
            binary: Binary,
        }

        let binary = data.iter().find(|binary| {
            binary.binary.image_type == "jdk"
                && (binary.binary.package.name.ends_with("tar.gz")
                    || binary.binary.package.name.ends_with("zip"))
        });

        if binary.is_none() {
            panic!("no binary found for Java {}", version);
        }

        let binary = binary.unwrap();
        let destination = format!("{}/{}/java.archive", self.location, version);

        std::fs::create_dir_all(format!("{}/{}", self.location, version)).unwrap();

        let mut res = self
            .client
            .get(&binary.binary.package.link)
            .send()
            .await
            .unwrap();
        let mut file = File::create(&destination).unwrap();

        let mut progress = Progress::new(res.content_length().unwrap() as usize);
        progress.spinner(|progress, spinner| {
            format!(
                "\r {} {} {}/{} ({}%)      ",
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

        while let Some(chunk) = res.chunk().await.unwrap() {
            file.write_all(&chunk).unwrap();
            progress.incr(chunk.len());
        }

        file.sync_all().unwrap();
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
            archive
                .extract(format!("{}/{}", self.location, version))
                .unwrap();
        } else {
            let mut archive = TarArchive::new(GzDecoder::new(File::open(&destination).unwrap()));
            archive
                .unpack(format!("{}/{}", self.location, version))
                .unwrap();
        }

        std::fs::remove_file(&destination).unwrap();

        let entries = std::fs::read_dir(format!("{}/{}", self.location, version)).unwrap();
        if entries.count() == 1 {
            let entry = std::fs::read_dir(format!("{}/{}", self.location, version))
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
            let path = entry.path();

            let files = std::fs::read_dir(&path).unwrap();
            for file in files {
                let file = file.unwrap();
                let file_path = file.path();
                std::fs::rename(
                    &file_path,
                    format!(
                        "{}/{}/{}",
                        self.location,
                        version,
                        file_path.file_name().unwrap().to_str().unwrap()
                    ),
                )
                .unwrap();
            }

            std::fs::remove_dir_all(&path).unwrap();
        }

        println!(
            " {} {} {} {}",
            "extracting java".bright_black().italic(),
            version.to_string().cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    }

    pub async fn versions(&self) -> Vec<u8> {
        let res = self
            .client
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
}
