use crate::api;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

#[inline]
pub fn format_uuid(uuid: &str) -> Option<String> {
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

#[inline]
pub async fn get_profile_uuid(uuid: &str) -> Result<Profile, reqwest::Error> {
    let uuid = format_uuid(uuid).unwrap_or_default();

    let res = api::CLIENT
        .get(format!(
            "https://sessionserver.mojang.com/session/minecraft/profile/{uuid}"
        ))
        .send()
        .await?;

    res.json::<Profile>().await
}

#[inline]
pub async fn get_profile_name(name: &str) -> Result<Profile, reqwest::Error> {
    let res = api::CLIENT
        .get(format!(
            "https://api.mojang.com/users/profiles/minecraft/{name}"
        ))
        .send()
        .await?;

    res.json::<Profile>().await
}
