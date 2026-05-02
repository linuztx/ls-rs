use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::time::SystemTime;

pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub is_executable: bool,
    pub is_hidden: bool,
    pub size: u64,
    pub mode: u32,
    pub nlink: u64,
    pub uid: u32,
    pub gid: u32,
    pub modified: SystemTime,
}

impl FileEntry {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let metadata = path.symlink_metadata()?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let is_hidden = name.starts_with('.');
        let file_type = metadata.file_type();
        let mode = metadata.permissions().mode();
        let is_executable = !file_type.is_dir() && (mode & 0o111) != 0;
        Ok(FileEntry {
            name,
            is_dir: file_type.is_dir(),
            is_symlink: file_type.is_symlink(),
            is_executable,
            is_hidden,
            size: metadata.len(),
            mode,
            nlink: metadata.nlink(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reaads_cargo_toml() {
        let entry = FileEntry::from_path(Path::new("Cargo.toml")).unwrap();

        assert_eq!(entry.name, "Cargo.toml");
        assert!(!entry.is_dir);
        assert!(!entry.is_symlink);
        assert!(!entry.is_hidden);
        assert!(entry.size > 0);
    }

    #[test]
    fn reads_src_directory() {
        let entry = FileEntry::from_path(Path::new("src")).unwrap();

        assert_eq!(entry.name, "src");
        assert!(entry.is_dir);
        assert!(!entry.is_symlink);
    }

    #[test]
    fn detects_hidden_file() {
        let entry = FileEntry::from_path(Path::new(".gitignore")).unwrap();

        assert!(entry.is_hidden);
        assert!(!entry.is_dir);
    }

    #[test]
    fn return_error_for_missing_file() {
        let entry = FileEntry::from_path(Path::new("dummy.txt"));
        assert!(entry.is_err())
    }
}
