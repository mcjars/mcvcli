use crate::api::{
    self, Progress,
    mcjars::{Build, InstallationStep, Version},
    modrinth::Project,
};
use crate::config::Config;

use colored::Colorize;
use human_bytes::human_bytes;
use indexmap::IndexMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::ZipArchive;

pub async fn install(build: &Build, directory: &str, spaces: usize) -> Result<(), anyhow::Error> {
    if Path::new(directory).join("libraries").exists() {
        std::fs::remove_dir_all(Path::new(directory).join("libraries")).unwrap_or_default();
    }

    for group in build.installation.iter() {
        for step in group.iter() {
            match step {
                InstallationStep::Download(step) => {
                    println!(
                        "{}{} {} {}",
                        " ".repeat(spaces),
                        "downloading".bright_black().italic(),
                        step.url.cyan().italic(),
                        "...".bright_black().italic()
                    );

                    let target = Path::new(directory).join(&step.file);
                    if let Some(parent) = target.parent()
                        && !parent.exists()
                    {
                        std::fs::create_dir_all(parent)?;
                    }

                    let mut res = api::CLIENT.get(&step.url).send().await?;
                    let mut file = File::create(&target)?;

                    let mut progress = Progress::new(step.size as usize);
                    progress.spinner(move |progress, spinner| {
                        format!(
                            "\r{} {} {} {}/{} ({}%)      ",
                            " ".repeat(spaces),
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
                        "{}{} {} {} {}",
                        " ".repeat(spaces),
                        "downloading".bright_black().italic(),
                        step.url.cyan().italic(),
                        "...".bright_black().italic(),
                        "DONE".green().bold().italic()
                    );
                }
                InstallationStep::Unzip(step) => {
                    println!(
                        "{}{} {} {}",
                        " ".repeat(spaces),
                        "extracting".bright_black().italic(),
                        step.file.cyan().italic(),
                        "...".bright_black().italic()
                    );

                    if !Path::new(&step.location).exists() {
                        std::fs::create_dir_all(Path::new(directory).join(&step.location))?;
                    }

                    let mut archive =
                        ZipArchive::new(File::open(Path::new(directory).join(&step.file))?)?;
                    archive.extract(Path::new(directory).join(&step.location))?;

                    println!(
                        "{}{} {} {} {}",
                        " ".repeat(spaces),
                        "extracting".bright_black().italic(),
                        step.file.cyan().italic(),
                        "...".bright_black().italic(),
                        "DONE".green().bold().italic()
                    );
                }
                InstallationStep::Remove(step) => {
                    println!(
                        "{}{} {} {}",
                        " ".repeat(spaces),
                        "removing".bright_black().italic(),
                        step.location.cyan().italic(),
                        "...".bright_black().italic()
                    );

                    if Path::new(directory).join(&step.location).is_dir() {
                        std::fs::remove_dir_all(Path::new(directory).join(&step.location))
                            .unwrap_or(());
                    } else {
                        std::fs::remove_file(Path::new(directory).join(&step.location))
                            .unwrap_or(());
                    }

                    println!(
                        "{}{} {} {} {}",
                        " ".repeat(spaces),
                        "removing".bright_black().italic(),
                        step.location.cyan().italic(),
                        "...".bright_black().italic(),
                        "DONE".green().bold().italic()
                    );
                }
            }
        }
    }

    Ok(())
}

pub async fn detect(
    directory: &str,
    config: &Config,
) -> Option<([Build; 2], IndexMap<String, Version>, Option<Project>)> {
    let mut file = Path::new(directory)
        .join(&config.jar_file)
        .to_string_lossy()
        .to_string();

    if let Ok(entries) =
        std::fs::read_dir(Path::new(directory).join("libraries/net/minecraftforge/forge"))
    {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                let Ok(entries) = std::fs::read_dir(&path) else {
                    continue;
                };

                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_file() {
                        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                            continue;
                        };

                        if name.ends_with("-server.jar") || name.ends_with("-universal.jar") {
                            file = path.to_string_lossy().to_string();
                            break;
                        }
                    }
                }
            }
        }
    } else if let Ok(entries) =
        std::fs::read_dir(Path::new(directory).join("libraries/net/neoforged/neoforge"))
    {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                let Ok(entries) = std::fs::read_dir(&path) else {
                    continue;
                };

                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_file() {
                        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                            continue;
                        };

                        if name.ends_with("-server.jar") || name.ends_with("-universal.jar") {
                            file = path.to_string_lossy().to_string();
                            break;
                        }
                    }
                }
            }
        }
    }

    if !Path::new(&file).exists() {
        return None;
    }

    if let Ok(([build, latest], versions)) = api::mcjars::lookup(&file).await {
        if let Some(modpack_slug) = &config.modpack_slug
            && let Ok(modpack) = api::modrinth::project(modpack_slug).await
        {
            return Some(([build, latest], versions, Some(modpack)));
        }

        Some(([build, latest], versions, None))
    } else {
        None
    }
}

#[inline]
pub fn is_latest_version(build: &Build, versions: &IndexMap<String, Version>) -> bool {
    let Some(version) = build
        .version_id
        .as_deref()
        .or(build.project_version_id.as_deref())
    else {
        return false;
    };

    let Some(version_type) = versions.get(version).map(|v| v.r#type.clone()) else {
        return false;
    };
    let latest_version = versions
        .iter()
        .rev()
        .find(|(_, v)| v.r#type == version_type);

    if let Some((k, _)) = latest_version
        && k == version
    {
        return true;
    }

    false
}
