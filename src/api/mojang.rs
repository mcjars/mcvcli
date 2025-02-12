use crate::api;

use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

pub struct MojangApi {
    client: Client,
}

impl MojangApi {
    pub fn new() -> Self {
        Self {
            client: api::client(),
        }
    }

    pub fn format_uuid(&self, raw_uuid: &str) -> Option<String> {
        let uuid = raw_uuid.replace("-", "");

        if uuid.len() < 32 {
            return None;
        }

        Some(format!(
            "{}-{}-{}-{}-{}",
            &uuid[0..8],
            &uuid[8..12],
            &uuid[12..16],
            &uuid[16..20],
            &uuid[20..32]
        ))
    }

    pub async fn get_profile_uuid(&self, raw_uuid: &str) -> Result<Profile, reqwest::Error> {
        let uuid = self.format_uuid(raw_uuid).unwrap_or_default();

        let res = self
            .client
            .get(format!(
                "https://sessionserver.mojang.com/session/minecraft/profile/{}",
                uuid
            ))
            .send()
            .await?;

        res.json::<Profile>().await
    }

    pub async fn get_profile_name(&self, name: &str) -> Result<Profile, reqwest::Error> {
        let res = self
            .client
            .get(format!(
                "https://api.mojang.com/users/profiles/minecraft/{}",
                name
            ))
            .send()
            .await?;

        res.json::<Profile>().await
    }
}
