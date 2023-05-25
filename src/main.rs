use chrono::{Datelike, Local};
use std::{
    io::stdin,
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

    // Append backup folder to output path
    let date = Local::now();
    output_path.push(format!(
        "Backup {:02}-{:02}-{}",
        date.day(),
        date.month(),
        date.year()
    ));

    // TODO: Pre-check directory size, file count, and subdirectory permissions
    println!("Checking source paths...");
    let mut source_paths = Vec::new();
    let mut dest_paths = Vec::new();
    let mut error_found = false;

    for line in std::fs::read_to_string(&input_file_path)?.lines() {
        if line.is_empty() {
            continue;
        }

        let mut source_path = PathBuf::from(line);

        // Check to make sure source path can be read and isn't a symlink
        match source_path.symlink_metadata() {
            Ok(m) => {
                if m.is_symlink() {
                    println!("Error reading \"{}\": cannot copy symlinks", line);
                    error_found = true;
                    continue;
                }
            }
            Err(e) => {
                println!("Error reading \"{}\": {}", line, e);
                error_found = true;
                continue;
            }
        }

        // if path is relative convert it to an absolute path
        source_path = match PathBuf::from(line).canonicalize() {
            Ok(p) => p,
            Err(e) => {
                println!("Error reading \"{}\": {}", line, e);
                error_found = true;
                continue;
            }
        };

        let mut source_path_iter = source_path.components();
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
                "Error reading \"{}\": source paths must begin with a drive letter",
                line
            );
            error_found = true;
            continue;
        }

        // Skip the root component so destination path doesn't get overridden, then append the rest of the input path
        for c in source_path_iter.skip(1) {
            dest_path.push(c);
        }

        source_paths.push(source_path);
        dest_paths.push(dest_path);
    }

    if error_found {
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
        stdin().read_line(&mut buf)?;

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

    // Iterate over source paths, copying them to their respective destination paths
    for (source, dest) in source_paths.iter().zip(dest_paths.iter()) {
        let status = Command::new("robocopy")
            .args([source, dest])
            .args(["/S", "/E", "/DCOPY:DAT", "/xj", "/eta", "/R:10", "/W:5"])
            .status()?
            .code()
            .expect("Error: no exit code returned");

        if status >= 8 {
            return Err(format!("Warning: errors during copy, exit code: {status}"))?;
        } else {
            println!("Copy complete, exit code: {status}");
        }
    }

    println!("Backup complete");
    return Ok(());
}
