use reqwest::{Client, ClientBuilder};
use serde::Deserialize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
        let client = ClientBuilder::new().user_agent(format!("mcvcli-rust/{}", VERSION));

        return Self {
            client: client.build().unwrap(),
        };
    }

    pub fn format_uuid(&self, raw_uuid: &str) -> Option<String> {
        let uuid = raw_uuid.replace("-", "");

        if uuid.len() < 32 {
            return None;
        }

        return Some(format!(
            "{}-{}-{}-{}-{}",
            &uuid[0..8],
            &uuid[8..12],
            &uuid[12..16],
            &uuid[16..20],
            &uuid[20..32]
        ));
    }

    pub async fn get_profile_uuid(&self, raw_uuid: &str) -> Result<Profile, reqwest::Error> {
        let uuid = self.format_uuid(raw_uuid).unwrap_or(String::new());

        let res = self
            .client
            .get(format!(
                "https://sessionserver.mojang.com/session/minecraft/profile/{}",
                uuid
            ))
            .send()
            .await?;

        return res.json::<Profile>().await;
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

        return res.json::<Profile>().await;
    }
}
