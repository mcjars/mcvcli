use crate::{
    api::{self, Progress, modrinth::Version},
    jar,
    progress::Progress as ProgressBar,
};

use colored::Colorize;
use human_bytes::human_bytes;
use serde::Deserialize;
use std::{fs::File, io::Write, path::Path, sync::Arc};
use tokio::sync::Mutex;
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
struct IndexJson {
    dependencies: IndexJsonDependencies,
    files: Vec<IndexJsonFile>,
}

#[derive(Debug, Deserialize)]
struct IndexJsonDependencies {
    minecraft: String,

    forge: Option<String>,
    neoforge: Option<String>,
    #[serde(rename = "fabric-loader")]
    fabric_loader: Option<String>,
    #[serde(rename = "quilt-loader")]
    quilt_loader: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndexJsonFile {
    path: String,
    downloads: Vec<String>,
    env: Option<IndexJsonFileEnv>,
    file_size: u64,
}

#[derive(Debug, Deserialize)]
struct IndexJsonFileEnv {
    server: String,
}

pub async fn install(directory: &str, version: &Version) -> Result<(), anyhow::Error> {
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .ok_or_else(|| anyhow::anyhow!("modpack version has no primary file"))?;

    println!(
        " {} {} {}",
        "downloading".bright_black(),
        file.filename.cyan(),
        "...".bright_black()
    );

    let mut res = reqwest::get(&file.url).await?;
    let mut mrpack_file = File::create(Path::new(directory).join(&file.filename))?;

    let mut progress = Progress::new(file.size as usize);
    progress.spinner(|progress, spinner| {
        format!(
            "\r  {} {} {}/{} ({}%)      ",
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
        mrpack_file.write_all(&chunk)?;
        progress.incr(chunk.len());
    }

    mrpack_file.sync_all()?;
    progress.finish();
    println!();

    println!(
        " {} {} {} {}",
        "downloading".bright_black().italic(),
        file.filename.cyan().italic(),
        "...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    let mut archive = ZipArchive::new(File::open(Path::new(directory).join(&file.filename))?)?;
    let index: IndexJson = serde_json::from_reader(archive.by_name("modrinth.index.json")?)?;

    println!(" {}", "extracting overrides...".bright_black().italic());

    std::fs::remove_dir_all(Path::new(directory).join("overrides")).unwrap_or_default();
    archive.extract(directory)?;

    std::fs::remove_file(Path::new(directory).join("modrinth.index.json")).unwrap_or_default();

    if let Ok(files) = std::fs::read_dir(Path::new(directory).join("overrides")) {
        for file in files.flatten() {
            let file_path = file.path();
            let Some(file_name) = file_path.file_name() else {
                continue;
            };
            let new_path = Path::new(directory).join(file_name);

            if new_path.exists() {
                if new_path.is_dir() {
                    std::fs::remove_dir_all(&new_path)?;
                } else {
                    std::fs::remove_file(&new_path)?;
                }
            }

            std::fs::rename(&file_path, &new_path)?;
        }

        std::fs::remove_dir_all(Path::new(directory).join("overrides"))?;
    }

    let _ = std::fs::remove_file(Path::new(directory).join(&file.filename));

    println!(
        " {} {}",
        "extracting overrides...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    println!(" {}", "downloading files...".bright_black().italic());

    let terminal_width = term_size::dimensions().map(|d| d.0).unwrap_or(80);
    for files in index.files.chunks(10) {
        let progress = Arc::new(Mutex::new(ProgressBar::with_capacity(10)));
        let mut handles = Vec::new();

        for file in files {
            if file
                .env
                .as_ref()
                .map(|e| e.server == "unsupported")
                .unwrap_or(true)
            {
                continue;
            }

            let progress = Arc::clone(&progress);
            let download = file.downloads[0].clone();
            let directory = directory.to_string();
            let file_path = file.path.clone();
            let mut file_display = file_path.clone();

            if file_display.len() > (terminal_width / 2) - 17 {
                file_display = format!("{}...", &file_display[..(terminal_width / 2) - 17]);
            }

            let bar = progress.lock().await.bar(
                file.file_size as usize,
                format!("  {}", file_display.cyan().italic()),
            );

            handles.push(async move {
                let file_path = Path::new(&directory).join(file_path);
                let file_name = file_path.display().to_string();

                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let mut res = reqwest::get(download).await?;
                let mut mod_file = File::create(&file_name)?;

                while let Some(chunk) = res.chunk().await? {
                    mod_file.write_all(&chunk)?;
                    progress.lock().await.inc_and_draw(&bar, chunk.len());
                }

                mod_file.sync_all()?;

                Ok::<_, anyhow::Error>(())
            });
        }

        futures::future::try_join_all(handles).await?;
    }

    println!(
        " {} {}",
        "downloading files...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    let minecraft = index.dependencies.minecraft;
    if let Some(fabric_loader) = index.dependencies.fabric_loader {
        let builds = api::mcjars::builds("FABRIC", &minecraft).await?;

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref() == Some(&fabric_loader))
            .ok_or_else(|| anyhow::anyhow!("no Fabric build found for {fabric_loader}"))?;

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "Fabric".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await?;

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "Fabric".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    } else if let Some(quilt_loader) = index.dependencies.quilt_loader {
        let builds = api::mcjars::builds("QUILT", &minecraft).await?;

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref() == Some(&quilt_loader))
            .ok_or_else(|| anyhow::anyhow!("no Quilt build found for {quilt_loader}"))?;

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "Quilt".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await?;

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "Quilt".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    } else if let Some(forge) = index.dependencies.forge {
        let builds = api::mcjars::builds("FORGE", &minecraft).await?;

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref() == Some(&forge))
            .ok_or_else(|| anyhow::anyhow!("no Forge build found for {forge}"))?;

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "Forge".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await?;

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "Forge".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    } else if let Some(neoforge) = index.dependencies.neoforge {
        let builds = api::mcjars::builds("NEOFORGE", &minecraft).await?;

        let build = builds
            .iter()
            .find(|build| build.project_version_id.as_ref() == Some(&neoforge))
            .ok_or_else(|| anyhow::anyhow!("no NeoForge build found for {neoforge}"))?;

        println!(
            " {} {} {}",
            "installing".bright_black().italic(),
            "NeoForge".cyan().italic(),
            "...".bright_black().italic()
        );

        jar::install(build, directory, 2).await?;

        println!(
            " {} {} {} {}",
            "installing".bright_black().italic(),
            "NeoForge".cyan().italic(),
            "...".bright_black().italic(),
            "DONE".green().bold().italic()
        );
    }

    Ok(())
}
