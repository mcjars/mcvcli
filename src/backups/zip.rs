use crate::{api::Progress, backups::counting_reader::CountingReader};
use colored::Colorize;
use std::{fs::File, path::Path, sync::Arc};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

fn recursive_add_directory(
    zip: &mut ZipWriter<std::fs::File>,
    directory: &Path,
    root: &Path,
    mut options: SimpleFileOptions,
    progress: &mut Progress,
) {
    for entry in std::fs::read_dir(directory).unwrap().flatten() {
        let path = entry.path();

        if path.file_name().unwrap() == ".mcvcli.backups"
            || path.file_name().unwrap() == ".mcvcli.profiles"
        {
            continue;
        }

        if path.is_dir() {
            #[cfg(unix)]
            if let Ok(metadata) = path.metadata() {
                use std::os::unix::fs::PermissionsExt;

                options = options.unix_permissions(metadata.permissions().mode());
            }

            zip.add_directory(path.strip_prefix(root).unwrap().to_str().unwrap(), options)
                .unwrap();
            recursive_add_directory(zip, &path, root, options, progress);
        } else {
            #[cfg(unix)]
            if let Ok(metadata) = path.metadata() {
                use std::os::unix::fs::PermissionsExt;

                options = options
                    .unix_permissions(metadata.permissions().mode())
                    .large_file(metadata.len() >= 4 * 1024 * 1024 * 1024);
            }

            zip.start_file_from_path(&path, options).unwrap();

            let mut reader =
                CountingReader::new(File::open(&path).unwrap(), Arc::clone(&progress.progress));
            std::io::copy(&mut reader, zip).unwrap();

            progress.incr(1usize);
            eprint!(
                "\r{} {}/{} ({}%)      ",
                " backing up...".bright_black().italic(),
                progress.progress().to_string().cyan().italic(),
                progress.total.to_string().cyan().italic(),
                progress.percent().round().to_string().cyan().italic()
            );
        }
    }
}

pub fn create(name: &str) {
    let path = format!(".mcvcli.backups/{name}.zip");
    let file = File::create(&path).unwrap();

    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Zstd)
        .unix_permissions(0o755);

    let mut total_size = 0;
    for entry in walkdir::WalkDir::new(".").into_iter().flatten() {
        let path = entry.path().to_str().unwrap();

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

    recursive_add_directory(
        &mut zip,
        Path::new("."),
        Path::new("."),
        options,
        &mut progress,
    );

    progress.finish();
    println!();

    zip.finish().unwrap();
}

pub fn restore(path: &str) {
    let mut archive = ZipArchive::new(File::open(path).unwrap()).unwrap();

    let mut progress = Progress::new(archive.len());
    progress.spinner(|progress, spinner| {
        format!(
            "\r {} {} {}/{} ({}%)      ",
            "restoring...".bright_black().italic(),
            spinner.cyan(),
            progress.progress().to_string().cyan().italic(),
            progress.total.to_string().cyan().italic(),
            progress.percent().round().to_string().cyan().italic()
        )
    });

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let path = file.mangled_name();

        if file.is_dir() {
            std::fs::create_dir_all(&path).unwrap();

            #[cfg(unix)]
            if let Some(mode) = file.unix_mode() {
                use std::os::unix::fs::PermissionsExt;

                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        } else {
            let mut write_file = std::fs::File::create(&path).unwrap();

            std::io::copy(&mut file, &mut write_file).unwrap();

            write_file.sync_all().unwrap();
            drop(write_file);

            #[cfg(unix)]
            if let Some(mode) = file.unix_mode() {
                use std::os::unix::fs::PermissionsExt;

                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    progress.finish();
    println!();
}
