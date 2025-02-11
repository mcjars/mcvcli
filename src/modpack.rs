use crate::{
    api::{mcjars::McjarsApi, modrinth::Version, Progress},
    jar,
};

use std::{fs::File, io::Write, path::Path};
use colored::Colorize;
use human_bytes::human_bytes;
use serde::Deserialize;
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
struct IndexJson {
    dependencies: IndexJsonDependencies,
    files: Vec<IndexJsonFile>,
}

#[derive(Debug, Deserialize)]
struct IndexJsonDependencies {
    minecraft: String,

    #[serde(rename = "fabric-loader")]
    fabric_loader: Option<String>,

    #[serde(rename = "quilt-loader")]
    quilt_loader: Option<String>,

    forge: Option<String>,
    neoforge: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IndexJsonFile {
    path: String,
    downloads: Vec<String>,
    env: Option<IndexJsonFileEnv>,

    #[serde(rename = "fileSize")]
    file_size: u64,
}

#[derive(Debug, Deserialize)]
struct IndexJsonFileEnv {
    server: String,
}

pub async fn install(directory: &str, api: &McjarsApi, version: &Version) {
    let file = version.files.iter().find(|file| file.primary).unwrap();

    println!(
        " {} {} {}",
        "downloading".bright_black(),
        file.filename.cyan(),
        "...".bright_black()
    );

    let mut res = reqwest::get(&file.url).await.unwrap();
    let mut mrpack_file = File::create(Path::new(directory).join(&file.filename)).unwrap();
    let mut progress = Progress {
        file_count: file.size,
        file_current: 0,
    };

    while let Some(chunk) = res.chunk().await.unwrap() {
        mrpack_file.write_all(&chunk).unwrap();

        progress.file_current += chunk.len() as u64;
        eprint!(
            "\r  {} {}/{} ({}%)      ",
            "downloading...".bright_black().italic(),
            human_bytes(progress.file_current as f64)
                .to_string()
                .cyan()
                .italic(),
            human_bytes(progress.file_count as f64)
                .to_string()
                .cyan()
                .italic(),
            ((progress.file_current as f64 / progress.file_count as f64) * 100.0)
                .round()
                .to_string()
                .cyan()
                .italic()
        );
    }

    mrpack_file.sync_all().unwrap();
    println!();

    println!(
        " {} {} {} {}",
        "downloading".bright_black().italic(),
        file.filename.cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    let mut archive =
        ZipArchive::new(File::open(Path::new(directory).join(&file.filename)).unwrap()).unwrap();
    let index: IndexJson =
        serde_json::from_reader(archive.by_name("modrinth.index.json").unwrap()).unwrap();

    println!(" {}", "extracting overrides...".bright_black().italic());

    std::fs::remove_dir_all(Path::new(directory).join("overrides")).unwrap_or_default();
    archive.extract(directory).unwrap();

    std::fs::remove_file(Path::new(directory).join("modrinth.index.json")).unwrap_or_default();

    if Path::new(directory).join("overrides").exists() {
        let files = std::fs::read_dir(Path::new(directory).join("overrides")).unwrap();
        for file in files {
            let file = file.unwrap();
            let file_path = file.path();
            let new_path =
                Path::new(directory).join(file_path.file_name().unwrap().to_str().unwrap());

            if new_path.exists() {
                if new_path.is_dir() {
                    std::fs::remove_dir_all(&new_path).unwrap();
                } else {
                    std::fs::remove_file(&new_path).unwrap();
                }
            }

            std::fs::rename(&file_path, &new_path).unwrap();
        }

        std::fs::remove_dir_all(Path::new(directory).join("overrides")).unwrap();
    }

    std::fs::remove_file(&file.filename).unwrap_or_default();

    println!(
        " {} {}",
        "extracting overrides...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    for file in index.files {
        if file.env.is_some() && file.env.unwrap().server == "unsupported" {
            continue;
        }

        let file_path = Path::new(directory).join(&file.path);
        let file_name = file_path.display().to_string();

        if !file_path.parent().unwrap().exists() {
            std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        }

        println!(
            " {} {} {}",
            "downloading".bright_black().italic(),
            file.path.cyan().italic(),
            "...".bright_black().italic()
        );

        let mut res = reqwest::get(&file.downloads[0]).await.unwrap();
        let mut mod_file = File::create(&file_name).unwrap();
        let mut progress = Progress {
            file_count: file.file_size,
            file_current: 0,
        };

        while let Some(chunk) = res.chunk().await.unwrap() {
            mod_file.write_all(&chunk).unwrap();

            progress.file_current += chunk.len() as u64;
            eprint!(
                "\r  {} {}/{} ({}%)      ",
                "downloading...".bright_black().italic(),
                human_bytes(progress.file_current as f64)
                    .to_string()
                    .cyan()
                    .italic(),
                human_bytes(progress.file_count as f64)
                    .to_string()
                    .cyan()
                    .italic(),
                ((progress.file_current as f64 / progress.file_count as f64) * 100.0)
                    .round()
                    .to_string()
                    .cyan()
                    .italic()
            );
        }

        mod_file.sync_all().unwrap();
        println!();

        println!(
            " {} {} {} {}",
            "downloading".bright_black().italic(),
            file.path.cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    }

    let minecraft = index.dependencies.minecraft;
    if index.dependencies.fabric_loader.is_some() {
        let fabric_loader = index.dependencies.fabric_loader.unwrap();
        let builds = api.builds("FABRIC", &minecraft).await.unwrap();

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref().unwrap() == &fabric_loader)
            .unwrap();

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "Fabric".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await.unwrap();

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "Fabric".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    } else if index.dependencies.quilt_loader.is_some() {
        let quilt_loader = index.dependencies.quilt_loader.unwrap();
        let builds = api.builds("QUILT", &minecraft).await.unwrap();

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref().unwrap() == &quilt_loader)
            .unwrap();

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "Quilt".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await.unwrap();

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "Quilt".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    } else if index.dependencies.forge.is_some() {
        let forge = index.dependencies.forge.unwrap();
        let builds = api.builds("FORGE", &minecraft).await.unwrap();

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref().unwrap() == &forge)
            .unwrap();

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "Forge".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await.unwrap();

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "Forge".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    } else if index.dependencies.neoforge.is_some() {
        let neoforge = index.dependencies.neoforge.unwrap();
        let builds = api.builds("NEOFORGE", &minecraft).await.unwrap();

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref().unwrap() == &neoforge)
            .unwrap();

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "NeoForge".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await.unwrap();

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "NeoForge".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    }
}
