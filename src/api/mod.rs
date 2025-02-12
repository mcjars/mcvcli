pub mod mcjars;
pub mod modrinth;
pub mod mojang;

use reqwest::Client;

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub struct Progress {
    pub file_count: u64,
    pub file_current: u64,
}

pub fn client() -> Client {
    Client::builder()
        .user_agent(format!("mcvcli-rust/{}", VERSION))
        .build()
        .unwrap()
}
