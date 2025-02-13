use crate::api::{self, Progress};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};
use human_bytes::human_bytes;
use serde::Deserialize;
use std::{env::temp_dir, fs::File, io::Write, path::Path};
use tar::Archive as TarArchive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    size: u64,

    browser_download_url: String,
}

pub async fn upgrade(_matches: &ArgMatches) -> i32 {
    println!("{}", "checking for updates ...".bright_black());

    let releases = api::client()
        .get("https://api.github.com/repos/mcjars/mcvcli/releases")
        .send()
        .await
        .unwrap()
        .json::<Vec<Release>>()
        .await
        .unwrap();

    let release = releases.first().unwrap();

    println!(
        "{} {}",
        "checking for updates ...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    if release.tag_name == VERSION {
        println!("{}", "you are already on the latest version".green());
        return 0;
    }

    let binary = std::env::current_exe()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    if binary.contains(".cargo") {
        println!(
            "{} {}",
            "unable to upgrade, installed through cargo, use".red(),
            "cargo install mcvcli".cyan()
        );
        return 1;
    }

    let confirm_upgrade = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you sure you want to upgrade? ({} -> {})",
            VERSION, release.tag_name
        ))
        .default(false)
        .interact()
        .unwrap();

    if !confirm_upgrade {
        return 1;
    }

    let arch = match std::env::consts::ARCH {
        "x86" => "x86_64",
        "arm" => "aarch64",
        _ => std::env::consts::ARCH,
    };

    let asset = match std::env::consts::OS {
        "macos" => release
            .assets
            .iter()
            .find(|asset| asset.name == format!("mcvcli-{}-macos.tar.xz", arch))
            .unwrap(),
        "windows" => release
            .assets
            .iter()
            .find(|asset| asset.name == format!("mcvcli-{}-windows.zip", arch))
            .unwrap(),
        _ => release
            .assets
            .iter()
            .find(|asset| asset.name == format!("mcvcli-{}-linux.tar.xz", arch))
            .unwrap(),
    };

    println!("{}", "installing update ...".bright_black());

    println!(
        "{} {} {}",
        " downloading".bright_black().italic(),
        asset.name.cyan().italic(),
        "...".bright_black().italic()
    );

    let mut res = api::client()
        .get(&asset.browser_download_url)
        .send()
        .await
        .unwrap();
    let mut file = File::create(Path::new(&temp_dir()).join(&asset.name)).unwrap();

    let mut progress = Progress::new(asset.size as usize);
    progress.spinner(|progress, spinner| {
        format!(
            "\r {} {} {}/{} ({}%)      ",
            "downloading...".bright_black().italic(),
            spinner.cyan(),
            human_bytes(progress.progress() as f64)
                .to_string()
                .cyan()
                .italic(),
            human_bytes(progress.total as f64)
                .to_string()
                .cyan()
                .italic(),
            progress.percent().round().to_string().cyan().italic()
        )
    });

    while let Some(chunk) = res.chunk().await.unwrap() {
        file.write_all(&chunk).unwrap();
        progress.incr(chunk.len());
    }

    file.sync_all().unwrap();
    progress.finish();
    println!();

    println!(
        " {} {} {} {}",
        "downloading".bright_black().italic(),
        asset.name.cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    println!(
        " {} {} {}",
        "extracting".bright_black().italic(),
        asset.name.cyan().italic(),
        "...".bright_black().italic()
    );

    if asset.name.ends_with(".zip") {
        let mut archive =
            ZipArchive::new(File::open(Path::new(&temp_dir()).join(&asset.name)).unwrap()).unwrap();
        archive.extract(temp_dir()).unwrap();
    } else if asset.name.ends_with(".tar.xz") {
        let mut archive = TarArchive::new(XzDecoder::new(
            File::open(Path::new(&temp_dir()).join(&asset.name)).unwrap(),
        ));
        archive.unpack(temp_dir()).unwrap();
    }

    std::fs::remove_file(Path::new(&temp_dir()).join(&asset.name)).unwrap();

    println!(
        " {} {} {} {}",
        "extracting".bright_black().italic(),
        asset.name.cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    let new_binary = std::fs::read_dir(
        Path::new(&temp_dir()).join(asset.name.replace(".tar.xz", "").replace(".zip", "")),
    );
    let new_binary = new_binary.unwrap().next().unwrap().unwrap().path();

    println!(
        " {} {} {} {} {}",
        "moving".bright_black().italic(),
        new_binary.to_str().unwrap().cyan().italic(),
        "to".bright_black().italic(),
        binary.cyan().italic(),
        "...".bright_black().italic()
    );

    if std::env::consts::OS == "windows" {
        std::process::Command::new("cmd")
            .arg("/C")
            .arg("move")
            .arg("/Y")
            .arg(new_binary.to_str().unwrap())
            .arg(&binary)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        std::fs::remove_dir_all(new_binary.parent().unwrap()).unwrap();
    } else {
        std::fs::remove_file(&binary).unwrap_or_default();
        std::fs::copy(&new_binary, &binary).unwrap();
        std::fs::remove_dir_all(new_binary.parent().unwrap()).unwrap();
    }

    println!(
        " {} {} {} {} {} {}",
        "moving".bright_black().italic(),
        new_binary.to_str().unwrap().cyan().italic(),
        "to".bright_black().italic(),
        binary.cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    println!(
        "{} {}",
        "installing update ...".bright_black(),
        "DONE".green().bold()
    );

    0
}
