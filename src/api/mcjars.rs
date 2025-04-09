use std::sync::LazyLock;

use crate::api;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::json;
use sha2::Digest;
use tokio::io::AsyncReadExt;

#[derive(Deserialize)]
pub struct Type {
    pub name: String,
}

#[derive(Deserialize)]
pub struct Version {
    pub r#type: String,
    pub java: u8,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Build {
    pub id: u32,
    pub r#type: String,
    pub name: String,

    pub version_id: Option<String>,
    pub project_version_id: Option<String>,

    pub installation: Vec<Vec<InstallationStep>>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum InstallationStep {
    #[serde(rename = "download")]
    Download(InstallationStepDownload),
    #[serde(rename = "unzip")]
    Unzip(InstallationStepUnzip),
    #[serde(rename = "remove")]
    Remove(InstallationStepRemove),
}

#[derive(Deserialize)]
pub struct InstallationStepDownload {
    pub url: String,
    pub file: String,
    pub size: u64,
}
#[derive(Deserialize)]
pub struct InstallationStepUnzip {
    pub file: String,
    pub location: String,
}
#[derive(Deserialize)]
pub struct InstallationStepRemove {
    pub location: String,
}

static MCJARS_URL: LazyLock<String> =
    LazyLock::new(|| std::env::var("MCJARS_URL").unwrap_or("https://mcjars.app".to_string()));
const MCJARS_FIELDS: &str = "id,type,versionId,projectVersionId,name,installation,changes";

pub async fn lookup(file: &str) -> Result<([Build; 2], IndexMap<String, Version>), reqwest::Error> {
    let mut sha512 = sha2::Sha512::new();
    let mut file = tokio::fs::File::open(file).await.unwrap();

    loop {
        let mut buffer = vec![0; 64 * 1024];
        let count = file.read(&mut buffer).await.unwrap();

        if count == 0 {
            break;
        }

        sha512.update(&buffer[..count]);
    }

    let res = api::CLIENT
        .post(format!(
            "{}/api/v2/build?fields={}",
            *MCJARS_URL, MCJARS_FIELDS
        ))
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

    let versions = versions(&data.build.r#type).await.unwrap();
    Ok(([data.build, data.latest], versions))
}

pub async fn lookup_id(id: u32) -> Result<(Build, IndexMap<String, Version>), reqwest::Error> {
    let res = api::CLIENT
        .post(format!(
            "{}/api/v2/build?fields={}",
            *MCJARS_URL, MCJARS_FIELDS
        ))
        .json(&json!({
            "id": id
        }))
        .send()
        .await?;
    let data = res.json::<ApiResponse>().await?;

    #[derive(Deserialize)]
    struct ApiResponse {
        build: Build,
    }

    let versions = versions(&data.build.r#type).await.unwrap();
    Ok((data.build, versions))
}

pub async fn types() -> Result<IndexMap<String, Type>, reqwest::Error> {
    let res = api::CLIENT
        .get(format!("{}/api/v2/types", *MCJARS_URL))
        .send()
        .await?;
    let data = res.json::<ApiResponse>().await?;

    #[derive(Deserialize)]
    struct ApiResponse {
        types: IndexMap<String, IndexMap<String, Type>>,
    }

    let mut types = IndexMap::new();
    for group in data.types.into_values() {
        for (key, value) in group {
            types.insert(key, value);
        }
    }

    Ok(types)
}

pub async fn versions(type_identifier: &str) -> Result<IndexMap<String, Version>, reqwest::Error> {
    let res = api::CLIENT
        .get(format!(
            "{}/api/v2/builds/{}?fields={}",
            *MCJARS_URL, type_identifier, MCJARS_FIELDS
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
    type_identifier: &str,
    version_identifier: &str,
) -> Result<Vec<Build>, reqwest::Error> {
    let res = api::CLIENT
        .get(format!(
            "{}/api/v2/builds/{}/{}?fields={}",
            *MCJARS_URL, type_identifier, version_identifier, MCJARS_FIELDS
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
