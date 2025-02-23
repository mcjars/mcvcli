use crate::api::Progress;

use colored::Colorize;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use std::{fs::File, io::Write, path::Path};
use tar::{Archive, Builder};
use xz2::{read::XzDecoder, write::XzEncoder};

fn recursive_add_directory(
    tar: &mut Builder<Box<dyn Write>>,
    directory: &Path,
    root: &Path,
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
            tar.append_dir(path.strip_prefix(root).unwrap().to_str().unwrap(), &path)
                .unwrap();
            recursive_add_directory(tar, &path, root, progress);
        } else {
            tar.append_file(&path, &mut File::open(&path).unwrap())
                .unwrap();

            progress.incr(1usize);
        }
    }
}

pub enum TarEncoder {
    Tar,
    Gz,
    Xz,
}

pub fn create(name: &str, encoder: TarEncoder, extension: &str) {
    let path = format!(".mcvcli.backups/{}.{}", name, extension);
    let file = File::create(&path).unwrap();

    let file: Box<dyn Write> = match encoder {
        TarEncoder::Tar => Box::new(file),
        TarEncoder::Gz => Box::new(GzEncoder::new(file, Compression::default())),
        TarEncoder::Xz => Box::new(XzEncoder::new(file, 9)),
    };

    let mut tar = Builder::new(file);

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

    let mut progress = Progress::new(file_count);
    progress.spinner(|progress, spinner| {
        format!(
            "\r {} {} {}/{} ({}%)      ",
            "backing up...".bright_black().italic(),
            spinner.cyan(),
            progress.progress().to_string().cyan().italic(),
            progress.total.to_string().cyan().italic(),
            progress.percent().round().to_string().cyan().italic()
        )
    });

    recursive_add_directory(&mut tar, Path::new("."), Path::new("."), &mut progress);

    progress.finish();
    println!();

    tar.finish().unwrap();
}

pub fn restore(path: &str, decoder: TarEncoder) {
    println!(" {}", "reading backup...".bright_black().italic());

    let file = File::open(path).unwrap();
    let mut archive: Archive<Box<dyn std::io::Read>> = match decoder {
        TarEncoder::Tar => Archive::new(Box::new(file)),
        TarEncoder::Gz => Archive::new(Box::new(GzDecoder::new(file))),
        TarEncoder::Xz => Archive::new(Box::new(XzDecoder::new(file))),
    };

    let total = {
        let mut archive: Archive<Box<dyn std::io::Read>> = match decoder {
            TarEncoder::Tar => Archive::new(Box::new(File::open(path).unwrap())),
            TarEncoder::Gz => Archive::new(Box::new(GzDecoder::new(File::open(path).unwrap()))),
            TarEncoder::Xz => Archive::new(Box::new(XzDecoder::new(File::open(path).unwrap()))),
        };

        archive.entries().unwrap().count()
    };

    println!(
        " {} {}",
        "reading backup...".bright_black().italic(),
        "DONE".green().bold().italic()
    );

    let mut progress = Progress::new(total);
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

    for file in archive.entries().unwrap() {
        let mut file = file.unwrap();
        let path = file.path().unwrap().to_path_buf();

        if file.header().entry_type().is_dir() {
            std::fs::create_dir_all(&path).unwrap();
        } else {
            let mut write_file = std::fs::File::create(&path).unwrap();

            std::io::copy(&mut file, &mut write_file).unwrap();
        }

        progress.incr(1usize);
    }

    progress.finish();
    println!();
}
