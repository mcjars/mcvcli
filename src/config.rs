use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

fn default_stop_command() -> String {
    "stop".to_string()
}
fn default_detached_log_max_mb() -> u64 {
    10
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip)]
    path: String,

    pub jar_file: String,
    #[serde(default = "default_stop_command")]
    pub stop_command: String,
    pub profile_name: String,

    pub modpack_slug: Option<String>,
    pub modpack_version: Option<String>,

    #[serde(alias = "ramMB")]
    pub ram_mb: u32,

    pub java_version: u8,

    pub extra_flags: Vec<String>,
    pub extra_args: Vec<String>,

    #[serde(default = "default_detached_log_max_mb")]
    pub detached_log_max_mb: u64,
}

impl Config {
    pub fn new(path: &str, create: bool) -> Self {
        if !Path::new(path).exists() {
            if create {
                let config = Config {
                    path: path.to_string(),
                    jar_file: "server.jar".to_string(),
                    stop_command: "stop".to_string(),
                    profile_name: "default".to_string(),
                    modpack_slug: None,
                    modpack_version: None,
                    ram_mb: 2048,
                    java_version: 21,
                    extra_flags: Vec::new(),
                    extra_args: Vec::new(),
                    detached_log_max_mb: default_detached_log_max_mb(),
                };

                let file = File::create(path).expect("failed to create config file");
                serde_json::to_writer_pretty(file, &config).expect("failed to write config file");

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

        let file = File::open(path).expect("failed to open config file");
        let mut config: Config =
            serde_json::from_reader(file).expect("failed to parse config file");

        config.path = path.to_string();

        config
    }

    pub fn new_optional(path: &str) -> Option<Self> {
        let file = File::open(path).ok()?;
        let mut config: Config =
            serde_json::from_reader(file).expect("failed to parse config file");

        config.path = path.to_string();

        Some(config)
    }

    pub fn save(&self) {
        let file = File::create(&self.path).expect("failed to create config file");
        serde_json::to_writer_pretty(file, &self).expect("failed to write config file");
    }
}
