mod args;
mod color;
mod entry;
mod format;

use std::fs;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use crate::args::Args;
use crate::entry::FileEntry;
use crate::format::{long_format, short_format};

fn main() -> ExitCode {
    let args = Args::parse();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprint!("ls-rs {}: {}", args.path, err);
            ExitCode::from(2)
        }
    }
}

fn run(args: &Args) -> io::Result<()> {
    let path = Path::new(&args.path);
    let entries = collect_entries(path, args.all)?;
    let output = if args.long {
        long_format(&entries)
    } else {
        short_format(&entries)
    };
    print!("{}", output);
    Ok(())
}

fn collect_entries(path: &Path, show_hidden: bool) -> io::Result<Vec<FileEntry>> {
    let metadata = path.symlink_metadata()?;
    if !metadata.is_dir() {
        return Ok(vec![FileEntry::from_path(path)?]);
    }

    let mut entries: Vec<FileEntry> = fs::read_dir(path)?
        .filter_map(|res| res.ok())
        .filter_map(|dir_entry| FileEntry::from_path(&dir_entry.path()).ok())
        .filter(|e| show_hidden || !e.is_hidden)
        .collect();

    entries.sort_by_key(|e| e.name.to_lowercase());
    Ok(entries)
}
