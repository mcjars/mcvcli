use crate::api;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

pub struct MojangApi;

impl MojangApi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn format_uuid(&self, uuid: &str) -> Option<String> {
        let uuid = uuid.replace("-", "");

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

    pub async fn get_profile_uuid(&self, uuid: &str) -> Result<Profile, reqwest::Error> {
        let uuid = self.format_uuid(uuid).unwrap_or_default();

        let res = api::CLIENT
            .get(format!(
                "https://sessionserver.mojang.com/session/minecraft/profile/{}",
                uuid
            ))
            .send()
            .await?;

        res.json::<Profile>().await
    }

    pub async fn get_profile_name(&self, name: &str) -> Result<Profile, reqwest::Error> {
        let res = api::CLIENT
            .get(format!(
                "https://api.mojang.com/users/profiles/minecraft/{}",
                name
            ))
            .send()
            .await?;

        res.json::<Profile>().await
    }
}
