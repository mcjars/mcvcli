use crate::api::{self, Progress};

use clap::ArgMatches;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use human_bytes::human_bytes;
use serde::Deserialize;
use std::{env::temp_dir, fs::File, io::Write, path::Path};
use tar::Archive as TarArchive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    size: u64,
    browser_download_url: String,
}

pub async fn upgrade(_matches: &ArgMatches) -> Result<i32, anyhow::Error> {
    println!("{}", "checking for updates ...".bright_black());

    let releases = api::CLIENT
        .get("https://api.github.com/repos/mcjars/mcvcli/releases")
        .send()
        .await?
        .json::<Vec<Release>>()
        .await?;

    let release = releases
        .first()
        .ok_or_else(|| anyhow::anyhow!("no releases found"))?;

    println!(
        "{} {}",
        "checking for updates ...".bright_black(),
        "DONE".green().bold()
    );
    println!();

    if release.tag_name == VERSION {
        println!("{}", "you are already on the latest version".green());
        return Ok(0);
    }

    let binary = std::env::current_exe()?.to_string_lossy().to_string();

    if binary.contains(".cargo") {
        println!(
            "{} {}",
            "unable to upgrade, installed through cargo, use".red(),
            "cargo install mcvcli".cyan()
        );
        return Ok(1);
    }

    let confirm_upgrade = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Are you sure you want to upgrade? ({} -> {})",
            VERSION, release.tag_name
        ))
        .default(false)
        .interact()?;

    if !confirm_upgrade {
        return Ok(1);
    }

    let arch = match std::env::consts::ARCH {
        "x86" => "x86_64",
        "arm" => "aarch64",
        _ => std::env::consts::ARCH,
    };

    let asset_name = match std::env::consts::OS {
        "macos" => format!("mcvcli-{arch}-macos.tar.xz"),
        "windows" => format!("mcvcli-{arch}-windows.zip"),
        _ => format!("mcvcli-{arch}-linux.tar.xz"),
    };

    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| anyhow::anyhow!("no release asset found for {asset_name}"))?;

    println!("{}", "installing update ...".bright_black());

    println!(
        "{} {} {}",
        " downloading".bright_black().italic(),
        asset.name.cyan().italic(),
        "...".bright_black().italic()
    );

    let mut res = api::CLIENT.get(&asset.browser_download_url).send().await?;
    let mut file = File::create(Path::new(&temp_dir()).join(&asset.name))?;

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

    while let Some(chunk) = res.chunk().await? {
        file.write_all(&chunk)?;
        progress.incr(chunk.len());
    }

    file.sync_all()?;
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
        let mut archive = ZipArchive::new(File::open(Path::new(&temp_dir()).join(&asset.name))?)?;
        archive.extract(temp_dir())?;
    } else if asset.name.ends_with(".tar.xz") {
        let mut archive = TarArchive::new(XzDecoder::new(File::open(
            Path::new(&temp_dir()).join(&asset.name),
        )?));
        archive.unpack(temp_dir())?;
    }

    std::fs::remove_file(Path::new(&temp_dir()).join(&asset.name))?;

    println!(
        " {} {} {} {}",
        "extracting".bright_black().italic(),
        asset.name.cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    let new_binary = std::fs::read_dir(
        Path::new(&temp_dir()).join(asset.name.replace(".tar.xz", "").replace(".zip", "")),
    )?
    .next()
    .ok_or_else(|| anyhow::anyhow!("extracted archive is empty"))??
    .path();
    let new_binary_parent = new_binary
        .parent()
        .ok_or_else(|| anyhow::anyhow!("invalid extracted binary path"))?
        .to_path_buf();

    println!(
        " {} {} {} {} {}",
        "moving".bright_black().italic(),
        new_binary.display().to_string().cyan().italic(),
        "to".bright_black().italic(),
        binary.cyan().italic(),
        "...".bright_black().italic()
    );

    if std::env::consts::OS == "windows" {
        let batch_path = Path::new(&temp_dir()).join("update_mcvcli.bat");
        let mut batch_file = File::create(&batch_path)?;

        writeln!(batch_file, "@echo off")?;
        writeln!(batch_file, "echo Waiting for mcvcli to exit...")?;
        writeln!(batch_file, "ping -n 2 127.0.0.1 > nul")?;
        writeln!(batch_file, "echo Updating mcvcli...")?;
        writeln!(
            batch_file,
            "copy /b /y \"{}\" \"{}\" > nul",
            new_binary.display(),
            binary
        )?;
        writeln!(batch_file, "echo Cleaning up...")?;
        writeln!(
            batch_file,
            "rmdir /s /q \"{}\" > nul",
            new_binary_parent.display()
        )?;
        writeln!(batch_file, "echo Update complete!")?;
        writeln!(batch_file, "exit")?;

        batch_file.sync_all()?;

        #[allow(clippy::zombie_processes)]
        std::process::Command::new("cmd")
            .args([
                "/C",
                "start",
                "/min",
                "",
                batch_path.to_string_lossy().as_ref(),
            ])
            .spawn()?;

        println!(
            " {} {} {} {} {} {}",
            "moving".bright_black().italic(),
            new_binary.display().to_string().cyan().italic(),
            "to".bright_black().italic(),
            binary.cyan().italic(),
            "...".bright_black().italic(),
            "SCHEDULED".yellow().bold().italic()
        );
    } else {
        std::fs::remove_file(&binary).unwrap_or_default();
        std::fs::copy(&new_binary, &binary)?;
        std::fs::remove_dir_all(&new_binary_parent)?;

        println!(
            " {} {} {} {} {} {}",
            "moving".bright_black().italic(),
            new_binary.display().to_string().cyan().italic(),
            "to".bright_black().italic(),
            binary.cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    }

    println!(
        "{} {}",
        "installing update ...".bright_black(),
        "DONE".green().bold()
    );

    Ok(0)
}
