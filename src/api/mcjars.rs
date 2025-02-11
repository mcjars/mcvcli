use indexmap::IndexMap;
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use tokio::io::AsyncReadExt;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Serialize, Deserialize)]
pub struct Type {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    pub r#type: String,
    pub java: u8,

    pub latest: Build,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Build {
    pub id: u32,
    pub r#type: String,
    pub name: String,

    #[serde(rename = "versionId")]
    pub version_id: Option<String>,
    #[serde(rename = "projectVersionId")]
    pub project_version_id: Option<String>,

    pub installation: Vec<Vec<InstallationStep>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstallationStep {
    #[serde(rename = "download")]
    Download(InstallationStepDownload),
    #[serde(rename = "unzip")]
    Unzip(InstallationStepUnzip),
    #[serde(rename = "remove")]
    Remove(InstallationStepRemove),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationStepDownload {
    pub url: String,
    pub file: String,
    pub size: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationStepUnzip {
    pub file: String,
    pub location: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationStepRemove {
    pub location: String,
}

pub struct McjarsApi {
    url: String,
    fields: String,
    client: Client,
}

impl McjarsApi {
    pub fn new() -> Self {
        let client = ClientBuilder::new().user_agent(format!("mcvcli-rust/{}", VERSION));

        Self {
            url: std::env::var("MCJARS_URL").unwrap_or("https://versions.mcjars.app".to_string()),
            fields: "id,type,versionId,projectVersionId,name,installation,changes".to_string(),
            client: client.build().unwrap(),
        }
    }

    pub async fn lookup(
        &self,
        file: &str,
    ) -> Result<([Build; 2], IndexMap<String, Version>), reqwest::Error> {
        let mut sha512 = sha2::Sha512::new();
        let mut file = tokio::fs::File::open(file).await.unwrap();

        loop {
            let mut buffer = vec![0; 1024];
            let count = file.read(&mut buffer).await.unwrap();

            if count == 0 {
                break;
            }

            sha512.update(&buffer[..count]);
        }

        let res = self
            .client
            .post(format!("{}/api/v2/build?fields={}", self.url, self.fields))
            .json(&json!({
                "hash": {
                    "sha512": format!("{:x}", sha512.finalize())
                }
            }))
            .send()
            .await?;
        let data = res.json::<ApiResponse>().await?;

        #[derive(Deserialize)]
        struct ApiResponse {
            build: Build,
            latest: Build,
        }

        let versions = self.versions(&data.build.r#type).await.unwrap();
        Ok(([data.build, data.latest], versions))
    }

    pub async fn types(&self) -> Result<IndexMap<String, Type>, reqwest::Error> {
        let res = self
            .client
            .get(format!("{}/api/v2/types", self.url))
            .send()
            .await?;
        let data = res.json::<ApiResponse>().await?;

        #[derive(Deserialize)]
        struct ApiResponse {
            types: IndexMap<String, IndexMap<String, Type>>,
        }

        let mut types = IndexMap::new();

        for (_, value) in data.types {
            for (key, value) in value {
                types.insert(key, value);
            }
        }

        Ok(types)
    }

    pub async fn versions(
        &self,
        type_identifier: &str,
    ) -> Result<IndexMap<String, Version>, reqwest::Error> {
        let res = self
            .client
            .get(format!(
                "{}/api/v2/builds/{}?fields={}",
                self.url, type_identifier, self.fields
            ))
            .send()
            .await?;
        let data = res.json::<ApiResponse>().await?;

        #[derive(Deserialize)]
        struct ApiResponse {
            builds: IndexMap<String, Version>,
        }

        Ok(data.builds)
    }

    pub async fn builds(
        &self,
        type_identifier: &str,
        version_identifier: &str,
    ) -> Result<Vec<Build>, reqwest::Error> {
        let res = self
            .client
            .get(format!(
                "{}/api/v2/builds/{}/{}?fields={}",
                self.url, type_identifier, version_identifier, self.fields
            ))
            .send()
            .await?;
        let data = res.json::<ApiResponse>().await?;

        #[derive(Deserialize)]
        struct ApiResponse {
            builds: Vec<Build>,
        }

        Ok(data.builds)
    }
}
