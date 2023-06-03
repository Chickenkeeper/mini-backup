mod backup_core;

use backup_core::{BackupError, BackupErrorKind, FileSize, ReadSubDir};
use chrono::{Datelike, Local};
use std::{
    io,
    path::{Component, PathBuf, Prefix},
    process::Command,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read arguments
    let mut args = std::env::args_os().skip(1);
    let input_file_path = match args.next() {
        Some(p) => Ok(PathBuf::from(p)),
        None => Err("Missing input path"),
    }?;
    let mut output_path = match args.next() {
        Some(p) => Ok(PathBuf::from(p)),
        None => Err("Missing output path"),
    }?;

    // Append the backup folder to the output path
    let date = Local::now();
    output_path.push(format!(
        "Backup {:02}-{:02}-{}",
        date.day(),
        date.month(),
        date.year()
    ));

    println!("Checking source paths...");
    let mut source_paths = Vec::new();
    let mut dest_paths = Vec::new();
    let mut file_names = Vec::new();
    let mut byte_count = 0;
    let mut file_count = 0;
    let mut folder_count = 0;
    let mut error_count = 0;

    for line in std::fs::read_to_string(&input_file_path)?.lines() {
        if line.is_empty() {
            continue;
        }

        let mut source_path = PathBuf::from(line);

        // Check to make sure the source path can be read and isn't a symlink
        let source_metadata = match source_path.symlink_metadata() {
            Ok(m) => m,
            Err(e) => {
                println!("{}", BackupError::new(e.into(), source_path));
                error_count += 1;
                continue;
            }
        };

        if source_metadata.is_symlink() {
            println!(
                "{}",
                BackupError::new(BackupErrorKind::IsSymlink, source_path)
            );
            error_count += 1;
            continue;
        }

        let mut file_name = PathBuf::new();

        // If the path points to a file separate the file name from the rest of the path so it can be passed as a separate argument into robocopy
        if source_metadata.is_file() {
            file_name.push(source_path.file_name().unwrap_or_default());
            source_path.pop();
        }

        // if the source path is relative convert it to an absolute path before appending it to the destination path
        let abs_source_path = match source_path.clone().canonicalize() {
            Ok(p) => p,
            Err(e) => {
                println!("{}", BackupError::new(e.into(), source_path));
                error_count += 1;
                continue;
            }
        };
        let mut source_path_iter = abs_source_path.components();
        let mut dest_path = output_path.clone();
        let mut has_drive_letter = false;

        // Extract the drive letter and append it to the destination path
        if let Some(drive) = source_path_iter.next() {
            if let Component::Prefix(p) = drive {
                if let Prefix::Disk(d) | Prefix::VerbatimDisk(d) = p.kind() {
                    dest_path.push(String::from(d as char));
                    has_drive_letter = true;
                }
            }
        }

        if !has_drive_letter {
            println!(
                "{}",
                BackupError::new(BackupErrorKind::NoDriveLetter, source_path)
            );
            error_count += 1;
            continue;
        }

        // Skip the root component so the destination path doesn't get overwritten, then append the rest of the source path
        for c in source_path_iter.skip(1) {
            dest_path.push(c);
        }

        for entry_r in ReadSubDir::new(source_path.to_owned()) {
            let entry = match entry_r {
                Ok(entry) => entry,
                Err(e) => {
                    println!("{}", e);
                    error_count += 1;
                    continue;
                }
            };
            let meta = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    println!("{}", BackupError::new(e.into(), entry.path()));
                    error_count += 1;
                    continue;
                }
            };

            byte_count += meta.len();
            if meta.is_dir() {
                folder_count += 1;
            } else if meta.is_file() {
                file_count += 1;
            }
        }

        source_paths.push(source_path);
        dest_paths.push(dest_path);
        file_names.push(file_name);
    }

    let total_size = FileSize::from_byte_count(byte_count);

    println!(
        "\nSize: {} {}, Files: {}, Folders: {}, Errors: {}",
        total_size.value, total_size.units, file_count, folder_count, error_count
    );

    if error_count > 0 {
        println!("\nWarning: errors found, affected paths will be skipped");
    } else {
        println!("\nAll source paths ok");
    }

    // Ask user to confirm before proceeding
    println!(
        "\nAre you sure you want to backup the paths in \"{}\" to \"{}\"? (y/n)",
        input_file_path.display(),
        output_path.display(),
    );
    let mut buf = String::new();
    loop {
        io::stdin().read_line(&mut buf)?;

        match buf.trim_end() {
            "y" | "Y" => break,
            "n" | "N" => {
                println!("Program quit");
                return Ok(());
            }
            _ => println!("Unrecognised input"),
        }
        buf.clear();
    }

    // Iterate over the source paths, copying them to their respective destination paths
    for ((source, dest), file) in source_paths
        .iter()
        .zip(dest_paths.iter())
        .zip(file_names.iter())
    {
        let status;

        if *file == PathBuf::new() {
            status = Command::new("robocopy")
                .args([source, dest])
                .args(["/S", "/E", "/DCOPY:DAT", "/xj", "/eta", "/R:10", "/W:5"])
                .status()?
                .code()
        } else {
            status = Command::new("robocopy")
                .args([source, dest, file])
                .args(["/DCOPY:DAT", "/xj", "/eta", "/R:10", "/W:5"])
                .status()?
                .code()
        }

        if let Some(code) = status {
            if code >= 8 {
                return Err(format!("Warning: Errors during copy, exit code: {}", code))?;
            } else {
                println!("Copy complete, exit code: {}", code);
            }
        } else {
            return Err("Warning: No exit code returned")?;
        }
    }

    println!("Backup complete");
    return Ok(());
}
