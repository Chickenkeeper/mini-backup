use std::{
    fmt,
    fs::{DirEntry, ReadDir},
    io,
    path::PathBuf,
};

#[derive(Debug)]
pub enum BackupErrorKind {
    Io(io::Error),
    IsSymlink,
    NoDriveLetter,
}

impl From<io::Error> for BackupErrorKind {
    fn from(err: io::Error) -> Self {
        return Self::Io(err);
    }
}

impl fmt::Display for BackupErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(e) => return e.fmt(f),
            Self::IsSymlink => return write!(f, "Cannot copy symlinks"),
            Self::NoDriveLetter => {
                return write!(f, "Absolute paths must begin with a drive letter")
            }
        }
    }
}

#[derive(Debug)]
pub struct BackupError {
    kind: BackupErrorKind,
    path: Option<PathBuf>,
}

impl BackupError {
    pub fn new(kind: BackupErrorKind, path: PathBuf) -> Self {
        return Self {
            kind,
            path: Some(path),
        };
    }
}

impl From<BackupErrorKind> for BackupError {
    fn from(kind: BackupErrorKind) -> Self {
        return Self { kind, path: None };
    }
}

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.path {
            Some(p) => {
                return write!(f, "Error: {}. Path: {}", self.kind, p.display());
            }
            None => {
                return write!(f, "Error: {}", self.kind);
            }
        }
    }
}

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
    type Item = Result<DirEntry, BackupError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(entries) = &mut self.entries {
                if let Some(entry_r) = entries.next() {
                    match entry_r {
                        Ok(entry) => {
                            if let Ok(file) = entry.file_type() {
                                // Symlinks will return false, so will be treated as files and skipped
                                if file.is_symlink() {
                                    return Some(Err(BackupError::new(
                                        BackupErrorKind::IsSymlink,
                                        entry.path(),
                                    )));
                                }
                                if file.is_dir() {
                                    self.dirs.push(entry.path());
                                }
                            }
                            return Some(Ok(entry));
                        }
                        Err(e) => {
                            return Some(Err(BackupError::from(BackupErrorKind::Io(e))));
                        }
                    }
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
                            return Some(Err(BackupError::new(e.into(), path)));
                        }
                    }
                } else {
                    return None;
                }
            }
        }
    }
}
