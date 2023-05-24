use chrono::{Datelike, Local};
use std::{
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

    // TODO: Pre-check input file count, sizes, and permissions, and warn user of any potential issues. Then ask user if they want to start the copy

    // Iterate over input paths, copying them to their respective destination paths
    // TODO: Ask user if they want to continue when encountering an error instead of exiting the program
    let mut dest_path = PathBuf::new();

    for l in std::fs::read_to_string(input_file_path)?.lines() {
        if l.is_empty() {
            continue;
        }

        dest_path.clear();
        dest_path.push(&output_path);

        let input_path = PathBuf::from(l);
        let mut input_path_iter = input_path.components();

        // Extract the drive letter and append it to the destination path
        if let Some(drive) = input_path_iter.next() {
            if let Component::Prefix(p) = drive {
                if let Prefix::Disk(d) | Prefix::VerbatimDisk(d) = p.kind() {
                    dest_path.push(String::from(d as char));
                } else {
                    return Err("Input paths must begin with a drive letter")?;
                }
            } else {
                return Err("Input paths must begin with a drive letter")?;
            }
        }

        // Skip the root component so destination path doesn't get overridden, then append the rest of the input path
        for c in input_path_iter.skip(1) {
            dest_path.push(c);
        }

        let status = Command::new("robocopy")
            .args([input_path.as_os_str(), dest_path.as_os_str()])
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
