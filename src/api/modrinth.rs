use crate::api;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::json;
use sha2::Digest;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use tokio::io::AsyncReadExt;

#[derive(Debug, Deserialize, Clone)]
pub struct Project {
    pub id: Option<String>,
    pub project_id: Option<String>,
    pub title: String,
    pub description: String,
    pub downloads: u32,

    pub installed_version: Option<Version>,
    pub installed_latest_version: Option<Version>,

    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Version {
    pub id: String,
    pub project_id: String,
    pub game_versions: Vec<String>,

    pub name: Option<String>,
    pub version_number: Option<String>,

    pub files: Vec<File>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct File {
    pub primary: bool,
    pub filename: String,
    pub url: String,
    pub size: u64,
}

pub struct ModrinthApi {
    url: String,
}

impl ModrinthApi {
    pub fn new() -> Self {
        Self {
            url: std::env::var("MODRINTH_API_URL")
                .unwrap_or("https://api.modrinth.com".to_string()),
        }
    }

    pub async fn projects(
        &self,
        query: &str,
        facets: &str,
    ) -> Result<Vec<Project>, reqwest::Error> {
        let response = api::CLIENT
            .get(format!(
                "{}/v2/search?query={}&facets={}&limit=9",
                self.url, query, facets
            ))
            .send()
            .await?;
        let data = response.json::<ApiResponse>().await?;

        #[derive(Deserialize)]
        struct ApiResponse {
            hits: Vec<Project>,
        }

        Ok(data.hits)
    }

    pub async fn project(&self, project_id: &str) -> Result<Project, reqwest::Error> {
        let response = api::CLIENT
            .get(format!("{}/v2/project/{}", self.url, project_id))
            .send()
            .await?;
        let data = response.json::<Project>().await?;

        Ok(data)
    }

    pub async fn versions(&self, project_id: &str) -> Result<Vec<Version>, reqwest::Error> {
        let response = api::CLIENT
            .get(format!("{}/v2/project/{}/version", self.url, project_id))
            .send()
            .await?;
        let data = response.json::<Vec<Version>>().await?;

        Ok(data)
    }

    pub async fn lookup(
        &self,
        folder: &str,
        loader: Option<&str>,
        version: Option<&str>,
    ) -> Result<IndexMap<PathBuf, Project>, Box<dyn std::error::Error>> {
        let mut read_dir = tokio::fs::read_dir(folder).await?;
        let mut hashes = HashMap::new();

        let mut sha512 = sha2::Sha512::new();
        while let Some(entry) = read_dir.next_entry().await? {
            if entry.path().extension().unwrap_or_default() != "jar" {
                continue;
            }

            if let Ok(mut file) = tokio::fs::File::open(entry.path()).await {
                let mut buffer = vec![0; 1024];

                loop {
                    let count = match file.read(&mut buffer).await {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => break,
                    };

                    sha512.update(&buffer[..count]);
                }

                hashes.insert(format!("{:x}", sha512.finalize_reset()), entry.path());
            }
        }

        let response = api::CLIENT
            .post(format!("{}/v2/version_files", self.url))
            .json(&json!({
                "hashes": hashes.keys().collect::<Vec<&String>>(),
                "algorithm": "sha512",
            }))
            .send()
            .await?;
        let data = response.json::<HashMap<String, Version>>().await?;

        let mut latest_data = HashMap::new();
        if let Some(loader) = loader {
            let response = api::CLIENT
                .post(format!("{}/v2/version_files/update", self.url))
                .json(&json!({
                    "hashes": hashes.keys().collect::<Vec<&String>>(),
                    "algorithm": "sha512",
                    "loaders": [loader],
                    "game_versions": [version.unwrap()],
                }))
                .send()
                .await?;

            latest_data = response.json::<HashMap<String, Version>>().await?;
        }

        let projects: HashSet<String> = data
            .values()
            .map(|version| version.project_id.clone())
            .collect();

        let response = api::CLIENT
            .get(format!(
                "{}/v2/projects?ids={}",
                self.url,
                serde_json::to_string(&projects).unwrap()
            ))
            .send()
            .await?;
        let mut projects_data = response.json::<Vec<Project>>().await?;
        let mut result = IndexMap::new();

        projects_data.sort_by(|a, b| a.title.cmp(&b.title));

        for project in projects_data {
            for (hash, version) in data.iter() {
                if version.project_id == project.id.clone().unwrap() {
                    let latest_version_data = latest_data.get(hash);

                    result.insert(
                        hashes.get(hash).unwrap().clone(),
                        Project {
                            installed_version: Some(Version {
                                project_id: project.id.clone().unwrap(),
                                ..version.clone()
                            }),
                            installed_latest_version: latest_version_data.cloned(),
                            ..project.clone()
                        },
                    );
                }
            }
        }

        Ok(result)
    }
}
