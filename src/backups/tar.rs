use crate::{api::Progress, backups::counting_reader::CountingReader};
use colored::Colorize;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::{fs::File, io::Write, path::Path, sync::Arc};
use tar::{Archive, Builder};
use xz2::{read::XzDecoder, write::XzEncoder};

fn recursive_add_directory(
    tar: &mut Builder<Box<dyn Write>>,
    directory: &Path,
    root: &Path,
    progress: &mut Progress,
) -> Result<(), anyhow::Error> {
    for entry in std::fs::read_dir(directory)?.flatten() {
        let path = entry.path();

        let Some(file_name) = path.file_name() else {
            continue;
        };
        if file_name == ".mcvcli.backups" || file_name == ".mcvcli.profiles" {
            continue;
        }

        let metadata = match path.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        if metadata.is_dir() {
            tar.append_dir(path.strip_prefix(root)?, &path)?;
            recursive_add_directory(tar, &path, root, progress)?;
        } else if metadata.is_file() {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(metadata.len());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                header.set_mode(metadata.permissions().mode());
            }

            header.set_mtime(
                metadata
                    .modified()
                    .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default())
                    .unwrap_or_default()
                    .as_secs(),
            );
            let mut reader =
                CountingReader::new(File::open(&path)?, Arc::clone(&progress.progress));

            tar.append_data(&mut header, &path, &mut reader)?;
        }
    }

    Ok(())
}

pub enum TarEncoder {
    Tar,
    Gz,
    Xz,
}

pub fn create(name: &str, encoder: TarEncoder, extension: &str) -> Result<(), anyhow::Error> {
    let path = format!(".mcvcli.backups/{name}.{extension}");
    let file = File::create(&path)?;

    let file: Box<dyn Write> = match encoder {
        TarEncoder::Tar => Box::new(file),
        TarEncoder::Gz => Box::new(GzEncoder::new(file, Compression::default())),
        TarEncoder::Xz => Box::new(XzEncoder::new(file, 9)),
    };

    let mut tar = Builder::new(file);

    let mut total_size = 0;
    for entry in walkdir::WalkDir::new(".").into_iter().flatten() {
        let path = entry.path().to_string_lossy();

        if path.contains(".mcvcli.backups") || path.contains(".mcvcli.profiles") {
            continue;
        }

        match entry.metadata() {
            Ok(metadata) => {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
            Err(_) => continue,
        }
    }

    let mut progress = Progress::new(total_size as usize);
    progress.spinner(|progress, spinner| {
        format!(
            "\r {} {} {}/{} ({}%)      ",
            "backing up...".bright_black().italic(),
            spinner.cyan(),
            human_bytes::human_bytes(progress.progress() as f64)
                .cyan()
                .italic(),
            human_bytes::human_bytes(progress.total as f64)
                .cyan()
                .italic(),
            progress.percent().round().to_string().cyan().italic()
        )
    });

    recursive_add_directory(&mut tar, Path::new("."), Path::new("."), &mut progress)?;

    progress.finish();
    println!();

    tar.finish()?;

    Ok(())
}

pub fn restore(path: &str, decoder: TarEncoder) -> Result<(), anyhow::Error> {
    println!(" {}", "reading backup...".bright_black().italic());

    let file = File::open(path)?;
    let total = file.metadata()?.len() as usize;
    let mut progress = Progress::new(total);

    let reader = CountingReader::new(file, Arc::clone(&progress.progress));
    let mut archive: Archive<Box<dyn std::io::Read>> = match decoder {
        TarEncoder::Tar => Archive::new(Box::new(reader)),
        TarEncoder::Gz => Archive::new(Box::new(GzDecoder::new(reader))),
        TarEncoder::Xz => Archive::new(Box::new(XzDecoder::new(reader))),
    };

    println!(
        " {} {}",
        "reading backup...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    progress.spinner(|progress, spinner| {
        format!(
            "\r {} {} {}/{} ({}%)      ",
            "restoring...".bright_black().italic(),
            spinner.cyan(),
            human_bytes::human_bytes(progress.progress() as f64)
                .cyan()
                .italic(),
            human_bytes::human_bytes(progress.total as f64)
                .cyan()
                .italic(),
            progress.percent().round().to_string().cyan().italic()
        )
    });

    for file in archive.entries()? {
        let mut file = file?;
        let path = file.path()?.to_path_buf();

        if file.header().entry_type().is_dir() {
            std::fs::create_dir_all(&path)?;

            #[cfg(unix)]
            if let Ok(mode) = file.header().mode() {
                use std::os::unix::fs::PermissionsExt;

                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
            }
        } else {
            let mut write_file = File::create(&path)?;

            std::io::copy(&mut file, &mut write_file)?;

            write_file.sync_all()?;
            drop(write_file);

            #[cfg(unix)]
            if let Ok(mode) = file.header().mode() {
                use std::os::unix::fs::PermissionsExt;

                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    progress.finish();
    println!();

    Ok(())
}
