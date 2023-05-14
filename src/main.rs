use chrono::{Datelike, Local};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let input_file_arg = match args.next() {
        Some(p) => Ok(p),
        None => Err("Missing input path"),
    }?;
    let mut output_path_arg = match args.next() {
        Some(p) => Ok(p),
        None => Err("Missing output path"),
    }?;

    let date = Local::now();
    let output_dir_name = format!(
        "\\Backup {:02}-{:02}-{}\\",
        date.day(),
        date.month(),
        date.year()
    );
    output_path_arg.push(output_dir_name);

    for l in std::fs::read_to_string(input_file_arg)?.lines() {
        if l.is_empty() {
            continue;
        }

        let source_path = l.replace(':', "");
        let mut dest_path = output_path_arg
            .clone()
            .to_str()
            .ok_or("Error: destination path is not valid unicode")?
            .to_string();
        dest_path.push_str(source_path.as_str());

        let status = Command::new("robocopy")
            .args([
                l,
                dest_path.as_str(),
                "/S",
                "/E",
                "/COPYALL",
                "/DCOPY:DAT",
                "/xj",
                "/eta",
                "/R:10",
                "/W:5",
            ])
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
