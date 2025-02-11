use reqwest::{Client, ClientBuilder};
use serde::Deserialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: Option<String>,
    pub project_id: Option<String>,
    pub title: String,
    pub description: String,
    pub downloads: u32,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub id: String,

    pub name: Option<String>,
    pub version_number: Option<String>,

    pub files: Vec<File>,
}

#[derive(Debug, Deserialize)]
pub struct File {
    pub primary: bool,
    pub filename: String,
    pub url: String,
    pub size: u64,
}

pub struct ModrinthApi {
    url: String,
    client: Client,
}

impl ModrinthApi {
    pub fn new() -> Self {
        let client = ClientBuilder::new().user_agent(format!("mcvcli-rust/{}", VERSION));

        Self {
            url: std::env::var("MODRINTH_API_URL")
                .unwrap_or("https://api.modrinth.com".to_string()),
            client: client.build().unwrap(),
        }
    }

    pub async fn projects(
        &self,
        query: &str,
        facets: &str,
    ) -> Result<Vec<Project>, reqwest::Error> {
        let url = format!(
            "{}/v2/search?query={}&facets={}&limit=9",
            self.url, query, facets
        );
        let response = self.client.get(&url).send().await?;
        let data = response.json::<ApiResponse>().await?;

        #[derive(Deserialize)]
        struct ApiResponse {
            hits: Vec<Project>,
        }

        Ok(data.hits)
    }

    pub async fn project(&self, project_id: &str) -> Result<Project, reqwest::Error> {
        let url = format!("{}/v2/project/{}", self.url, project_id);
        let response = self.client.get(&url).send().await?;
        let data = response.json::<Project>().await?;

        Ok(data)
    }

    pub async fn versions(&self, project_id: &str) -> Result<Vec<Version>, reqwest::Error> {
        let url = format!("{}/v2/project/{}/version", self.url, project_id);
        let response = self.client.get(&url).send().await?;
        let data = response.json::<Vec<Version>>().await?;

        Ok(data)
    }
}
