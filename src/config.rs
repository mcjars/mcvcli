use colored::Colorize;
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    path: String,

    #[serde(rename = "jarFile")]
    pub jar_file: String,

    #[serde(rename = "profileName")]
    pub profile_name: String,

    #[serde(rename = "modpackSlug")]
    pub modpack_slug: Option<String>,

    #[serde(rename = "modpackVersion")]
    pub modpack_version: Option<String>,

    #[serde(rename = "ramMB")]
    pub ram_mb: u16,

    #[serde(rename = "javaVersion", default)]
    pub java_version: u8,

    #[serde(rename = "extraFlags", default)]
    pub extra_flags: Vec<String>,

    #[serde(rename = "extraArgs", default)]
    pub extra_args: Vec<String>,

    pub pid: Option<usize>,
    pub identifier: Option<String>,
}

impl Config {
    pub fn new(path: &str, create: bool) -> Self {
        if !Path::new(path).exists() {
            if create {
                let config = Config {
                    path: path.to_string(),
                    jar_file: "server.jar".to_string(),
                    profile_name: "default".to_string(),
                    modpack_slug: None,
                    modpack_version: None,
                    ram_mb: 2048,
                    java_version: 21,
                    extra_flags: Vec::new(),
                    extra_args: Vec::new(),
                    pid: None,
                    identifier: Some(
                        rand::rng()
                            .sample_iter(&Alphanumeric)
                            .take(7)
                            .map(char::from)
                            .collect(),
                    ),
                };

                let file = File::create(path).unwrap();
                serde_json::to_writer_pretty(file, &config).unwrap();

                return config;
            } else {
                println!("{}", "Config file does not exist!".red());
                println!(
                    "{} {} {}",
                    "Run".red(),
                    "mcvcli init .".cyan(),
                    "to initialize a new server.".red()
                );
                std::process::exit(1);
            }
        }

        let file = File::open(path).unwrap();
        let mut config: Config = serde_json::from_reader(file).unwrap();

        config.path = path.to_string();

        config
    }

    pub fn new_optional(path: &str) -> Option<Self> {
        if !Path::new(path).exists() {
            return None;
        }

        let file = File::open(path).unwrap();
        let mut config: Config = serde_json::from_reader(file).unwrap();

        config.path = path.to_string();

        Some(config)
    }

    pub fn save(&self) {
        let file = File::create(&self.path).unwrap();
        serde_json::to_writer_pretty(file, &self).unwrap();
    }
}
