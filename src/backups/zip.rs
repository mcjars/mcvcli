use crate::api::Progress;

use colored::Colorize;
use std::{fs::File, io::Write, path::Path};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

fn recursive_add_directory(
    zip: &mut ZipWriter<std::fs::File>,
    directory: &Path,
    root: &Path,
    options: SimpleFileOptions,
    progress: &mut Progress,
) {
    let entries = std::fs::read_dir(directory).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.file_name().unwrap() == ".mcvcli.backups"
            || path.file_name().unwrap() == ".mcvcli.profiles"
        {
            continue;
        }

        if path.is_dir() {
            zip.add_directory(path.strip_prefix(root).unwrap().to_str().unwrap(), options)
                .unwrap();
            recursive_add_directory(zip, &path, root, options, progress);
        } else {
            zip.start_file_from_path(&path, options).unwrap();
            zip.write_all(&std::fs::read(path).unwrap()).unwrap();

            progress.file_current += 1;
            eprint!(
                "\r{} {}/{} ({}%)      ",
                " backing up...".bright_black().italic(),
                progress.file_current.to_string().cyan().italic(),
                progress.file_count.to_string().cyan().italic(),
                ((progress.file_current as f64 / progress.file_count as f64) * 100.0)
                    .round()
                    .to_string()
                    .cyan()
                    .italic()
            );
        }
    }
}

pub fn create(name: &str) {
    let path = format!(".mcvcli.backups/{}.zip", name);
    let file = File::create(&path).unwrap();

    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Zstd)
        .unix_permissions(0o755);

    let mut file_count = 0;
    for entry in walkdir::WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path().to_str().unwrap();

        if path.contains(".mcvcli.backups") || path.contains(".mcvcli.profiles") {
            continue;
        }

        if entry.path().is_file() {
            file_count += 1;
        }
    }

    recursive_add_directory(
        &mut zip,
        Path::new("."),
        Path::new("."),
        options,
        &mut Progress {
            file_count,
            file_current: 0,
        },
    );

    println!();

    zip.finish().unwrap();
}

pub fn restore(path: &str) {
    let mut archive = ZipArchive::new(File::open(&path).unwrap()).unwrap();
    let mut progress = Progress {
        file_count: archive.len() as u64,
        file_current: 0,
    };

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let path = file.mangled_name();

        if file.is_dir() {
            std::fs::create_dir_all(&path).unwrap();
        } else {
            let mut write_file = std::fs::File::create(&path).unwrap();

            std::io::copy(&mut file, &mut write_file).unwrap();
        }

        progress.file_current += 1;
        eprint!(
            "\r{} {}/{} ({}%)      ",
            " restoring...".bright_black().italic(),
            progress.file_current.to_string().cyan().italic(),
            progress.file_count.to_string().cyan().italic(),
            ((progress.file_current as f64 / progress.file_count as f64) * 100.0)
                .round()
                .to_string()
                .cyan()
                .italic()
        );
    }

    println!();
}
