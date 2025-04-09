use crate::{api, config};

use clap::ArgMatches;
use colored::Colorize;
use flate2::bufread::GzDecoder;
use serde::Deserialize;
use std::{fmt::Debug, fs::File, io::BufReader, path::Path};

#[derive(Debug, Deserialize)]
struct PlayerData {
    #[serde(rename = "Health")]
    health: f64,

    #[serde(rename = "Pos")]
    position: [f64; 3],

    #[serde(rename = "Dimension")]
    dimension: String,

    #[serde(rename = "Inventory")]
    inventory: Option<Vec<InventoryItem>>,

    attributes: Option<Vec<Attribute>>,
}

#[derive(Debug, Deserialize)]
struct InventoryItem {
    count: u8,
}

#[derive(Debug, Deserialize)]
struct Attribute {
    id: String,
    base: f64,
}

#[derive(Debug, Deserialize)]
struct PlayerStats {
    stats: Stats,
}

#[derive(Debug, Deserialize)]
struct Stats {
    #[serde(rename = "minecraft:custom")]
    custom: Custom,
}

#[derive(Debug, Deserialize)]
struct Custom {
    #[serde(rename = "minecraft:deaths")]
    deaths: Option<u32>,

    #[serde(rename = "minecraft:player_kills")]
    player_kills: Option<u32>,

    #[serde(rename = "minecraft:play_time")]
    play_time: Option<u32>,
}

pub async fn lookup(matches: &ArgMatches) -> i32 {
    let player = matches.get_one::<String>("player").expect("required");
    let _config = config::Config::new(".mcvcli.json", false);

    println!("{}", "looking up player...".bright_black());

    let player = if let Some(uuid) = api::mojang::format_uuid(player) {
        api::mojang::get_profile_uuid(&uuid).await
    } else {
        api::mojang::get_profile_name(player).await
    };

    if player.is_err() {
        println!(
            "{} {}",
            "looking up player...".bright_black(),
            "FAILED".red().bold()
        );
        return 1;
    }

    println!(
        "{} {}",
        "looking up player...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    let player = player.unwrap();

    println!("{}", player.name.cyan().bold());
    println!(
        "  {} {}",
        "uuid:".bright_black(),
        api::mojang::format_uuid(&player.id).unwrap().cyan()
    );

    if !Path::new(&format!(
        "world/playerdata/{}.dat",
        api::mojang::format_uuid(&player.id).unwrap()
    ))
    .exists()
    {
        println!(
            "  {} {}",
            "player data:".bright_black(),
            "not found".red().bold()
        );
        return 0;
    }

    println!("  {}", "player data:".bright_black());

    let player_data = File::open(format!(
        "world/playerdata/{}.dat",
        api::mojang::format_uuid(&player.id).unwrap()
    ))
    .unwrap();
    let player_data = GzDecoder::new(BufReader::new(player_data));
    let player_data: Result<PlayerData, _> = fastnbt::from_reader(player_data);

    let player_stats: Option<PlayerStats> = match File::open(format!(
        "world/stats/{}.json",
        api::mojang::format_uuid(&player.id).unwrap()
    )) {
        Ok(player_stats) => serde_json::from_reader(player_stats).ok(),
        Err(_) => None,
    };

    if player_data.is_err() {
        println!(
            "    {} {}",
            "player data:".bright_black(),
            "unable to read".red().bold()
        );
        return 1;
    }

    let player_data = player_data.unwrap();

    let mut max_health: f64 = 20.0;
    if let Some(attributes) = player_data.attributes {
        for attribute in attributes {
            if attribute.id == "minecraft:max_health" {
                max_health = attribute.base;
                break;
            }
        }
    }

    println!(
        "    {} {}/{} ({}%)",
        "health:".bright_black(),
        player_data.health.round().to_string().cyan(),
        max_health.round().to_string().cyan(),
        ((player_data.health / max_health) * 100.0)
            .round()
            .to_string()
            .cyan()
    );

    println!("    {}", "position:".bright_black());
    println!(
        "      {} {}",
        "x:".bright_black(),
        player_data.position[0].to_string().cyan()
    );
    println!(
        "      {} {}",
        "y:".bright_black(),
        player_data.position[1].to_string().cyan()
    );
    println!(
        "      {} {}",
        "z:".bright_black(),
        player_data.position[2].to_string().cyan()
    );

    println!(
        "    {} {}",
        "dimension:".bright_black(),
        player_data.dimension.cyan()
    );

    if let Some(inventory) = player_data.inventory {
        println!("    {}", "inventory:".bright_black());

        let mut total: u32 = 0;
        let mut filled: u32 = 0;
        for item in inventory {
            total += item.count as u32;
            filled += 1;
        }

        println!(
            "      {} {}",
            "total items: ".bright_black(),
            total.to_string().cyan()
        );
        println!(
            "      {} {}",
            "filled slots:".bright_black(),
            filled.to_string().cyan()
        );
    }

    if let Some(player_stats) = player_stats {
        println!("    {}", "stats:".bright_black());

        if let Some(deaths) = player_stats.stats.custom.deaths {
            println!(
                "      {} {}",
                "deaths:      ".bright_black(),
                deaths.to_string().cyan()
            );
        }

        if let Some(player_kills) = player_stats.stats.custom.player_kills {
            println!(
                "      {} {}",
                "player kills:".bright_black(),
                player_kills.to_string().cyan()
            );
        }

        if let Some(play_time) = player_stats.stats.custom.play_time {
            println!(
                "      {} {}h {}m {}s",
                "play time:   ".bright_black(),
                (play_time / 72000).to_string().cyan(),
                ((play_time % 72000) / 1200).to_string().cyan(),
                ((play_time % 72000) % 1200 / 20).to_string().cyan()
            );
        }
    }

    0
}
