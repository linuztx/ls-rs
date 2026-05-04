use colored::{ColoredString, Colorize};

use crate::entry::FileEntry;

pub fn colorize(entry: &FileEntry) -> ColoredString {
    if entry.is_symlink {
        entry.name.cyan().bold()
    } else if entry.is_dir {
        entry.name.blue().bold()
    } else if entry.is_executable {
        entry.name.green().bold()
    } else {
        entry.name.normal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Color;
    use std::time::SystemTime;

    fn entry(name: &str, kind: &str) -> FileEntry {
        FileEntry {
            name: name.into(),
            is_dir: kind == "dir",
            is_symlink: kind == "symlink",
            is_executable: kind == "exec",
            is_hidden: false,
            size: 0,
            mode: 0,
            nlink: 0,
            uid: 0,
            gid: 0,
            modified: SystemTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn directories_are_blue() {
        let result = colorize(&entry("src", "dir"));
        assert_eq!(result.fgcolor, Some(Color::Blue));
    }

    #[test]
    fn symlinks_are_cyan() {
        let result = colorize(&entry("link", "symlink"));
        assert_eq!(result.fgcolor, Some(Color::Cyan));
    }

    #[test]
    fn executables_are_green() {
        let result = colorize(&entry("run.sh", "exec"));
        assert_eq!(result.fgcolor, Some(Color::Green));
    }

    #[test]
    fn regular_files_have_no_color() {
        let result = colorize(&entry("README.md", "file"));
        assert_eq!(result.fgcolor, None)
    }
}
