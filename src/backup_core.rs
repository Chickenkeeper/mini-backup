use std::{
    fs::{DirEntry, ReadDir},
    io,
    path::PathBuf,
};

pub struct FileSize {
    pub value: f32,
    pub units: &'static str,
}

impl FileSize {
    pub fn from_byte_count(byte_count: u64) -> Self {
        let value;
        let units;

        if byte_count > 1_000_000_000 {
            value = ((byte_count as f32) / 10_000_000.0).round() / 100.0;
            units = "GB";
        } else if byte_count > 1_000_000 {
            value = ((byte_count as f32) / 10_000.0).round() / 100.0;
            units = "MB";
        } else if byte_count > 1_000 {
            value = ((byte_count as f32) / 10.0).round() / 100.0;
            units = "KB";
        } else {
            value = byte_count as f32;
            units = "B";
        }

        return Self { value, units };
    }
}

pub struct ReadSubDir {
    dirs: Vec<PathBuf>,
    entries: Option<ReadDir>,
}

impl ReadSubDir {
    pub fn new(path: PathBuf) -> Self {
        ReadSubDir {
            dirs: vec![path.clone()],
            entries: None,
        }
    }
}

impl Iterator for ReadSubDir {
    type Item = io::Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entries) = &mut self.entries {
                if let Some(entry_r) = entries.next() {
                    if let Ok(entry) = &entry_r {
                        if let Ok(file) = entry.file_type() {
                            // Symlinks will return false, so will be treated as files and skipped
                            if file.is_dir() {
                                self.dirs.push(entry.path());
                            }
                        }
                    }
                    return Some(entry_r);
                } else {
                    self.entries = None;
                }
            } else {
                if let Some(path) = self.dirs.pop() {
                    match path.read_dir() {
                        Ok(entries) => {
                            self.entries = Some(entries);
                        }
                        Err(e) => {
                            return Some(Err(io::Error::new(
                                io::ErrorKind::Other,
                                format!("{}. Path: \"{}\"", e, path.display()),
                            )));
                        }
                    }
                } else {
                    return None;
                }
            }
        }
    }
}
