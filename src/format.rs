use chrono::{DateTime, Datelike, Local};
use terminal_size::{Width, terminal_size};

use crate::color::colorize;
use crate::entry::FileEntry;

const COLUMN_PADDING: usize = 2;

pub fn short_format(entries: &[FileEntry]) -> String {
    if entries.is_empty() {
        return "\n".to_string();
    }

    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    let (cols, rows, col_widths) = compute_layout(entries, term_width);
    let mut out = String::with_capacity(entries.len() * 16);

    for row in 0..rows {
        for (col, &width) in col_widths.iter().enumerate() {
            let idx = col * rows + row;
            if let Some(entry) = entries.get(idx) {
                out.push_str(&colorize(entry).to_string());
                if col + 1 < cols {
                    let pad = width - entry.name.len() + COLUMN_PADDING;
                    for _ in 0..pad {
                        out.push(' ');
                    }
                }
            }
        }
        out.push('\n');
    }
    out
}

pub fn long_format(entries: &[FileEntry]) -> String {
    let now = Local::now();

    let rows: Vec<[String; 7]> = entries
        .iter()
        .map(|e| {
            [
                permissions_string(e),
                e.nlink.to_string(),
                user_name(e.uid),
                group_name(e.gid),
                e.size.to_string(),
                format_time(e.modified, now),
                colorize(e).to_string(),
            ]
        })
        .collect();

    let widths = column_widths(&rows);
    let total_blocks: u64 = entries.iter().map(|e| e.blocks / 2).sum();

    let mut out = format!("total {}\n", total_blocks);
    for row in &rows {
        out.push_str(&format!(
            "{} {:>w1$} {:<w2$} {:<w3$} {:>w4$} {} {}\n",
            row[0],
            row[1],
            row[2],
            row[3],
            row[4],
            row[5],
            row[6],
            w1 = widths[1],
            w2 = widths[2],
            w3 = widths[3],
            w4 = widths[4],
        ))
    }

    out
}

fn compute_layout(entries: &[FileEntry], term_width: usize) -> (usize, usize, Vec<usize>) {
    let n = entries.len();

    for cols in (1..=n).rev() {
        let rows = n.div_ceil(cols);
        let mut col_widths = vec![0usize; cols];
        for (i, entry) in entries.iter().enumerate() {
            let col = i / rows;
            col_widths[col] = col_widths[col].max(entry.name.len());
        }
        let total: usize = col_widths
            .iter()
            .enumerate()
            .map(|(i, w)| if i + 1 < cols { w + COLUMN_PADDING } else { *w })
            .sum();

        if total <= term_width {
            return (cols, rows, col_widths);
        }
    }

    let max_w = entries.iter().map(|e| e.name.len()).max().unwrap_or(0);
    (1, n, vec![max_w])
}

fn column_widths(rows: &[[String; 7]]) -> [usize; 7] {
    let mut widths = [0usize; 7];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if cell.len() > widths[i] {
                widths[i] = cell.len();
            }
        }
    }
    widths
}

fn permissions_string(entry: &FileEntry) -> String {
    let mut s = String::with_capacity(10);
    s.push(if entry.is_symlink {
        'l'
    } else if entry.is_dir {
        'd'
    } else {
        '-'
    });
    let mode = entry.mode;
    for shift in [6, 3, 0] {
        s.push(if mode >> (shift + 2) & 1 == 1 {
            'r'
        } else {
            '-'
        });
        s.push(if mode >> (shift + 1) & 1 == 1 {
            'w'
        } else {
            '-'
        });
        s.push(if mode >> shift & 1 == 1 { 'x' } else { '-' });
    }
    s
}

fn user_name(uid: u32) -> String {
    users::get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| uid.to_string())
}

fn group_name(gid: u32) -> String {
    users::get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| gid.to_string())
}

fn format_time(t: std::time::SystemTime, now: DateTime<Local>) -> String {
    let dt: DateTime<Local> = t.into();
    if dt.year() == now.year() {
        dt.format("%b %e %H:%M").to_string()
    } else {
        dt.format("%b %e %Y").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            blocks: 0,
        }
    }

    #[test]
    fn empty_list_returns_just_newline() {
        let result = short_format(&[]);
        assert_eq!(result, "\n");
    }

    #[test]
    fn single_regular_file_has_name_and_newline() {
        let result = short_format(&[entry("README.md", "file")]);
        assert_eq!(result, "README.md\n");
    }

    #[test]
    fn multiple_files_are_separated_by_two_spaces() {
        let entries = [
            entry("a.txt", "file"),
            entry("b.txt", "file"),
            entry("c.txt", "file"),
        ];
        let result = short_format(&entries);
        assert_eq!(result, "a.txt  b.txt  c.txt\n");
    }

    #[test]
    fn no_separator_before_first_or_after_last() {
        let result = short_format(&[entry("only.txt", "file")]);
        assert!(!result.starts_with(' '));
        assert!(result.ends_with("only.txt\n"));
    }

    #[test]
    fn output_always_ends_with_newline() {
        let result = short_format(&[entry("a.txt", "file"), entry("b.txt", "file")]);
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn colored_entries_still_contain_the_name() {
        let result = short_format(&[entry("src", "dir")]);
        assert!(result.contains("src"));
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn uid_zero_is_root() {
        assert_eq!(user_name(0), "root");
    }

    #[test]
    fn gid_zero_is_root() {
        assert_eq!(group_name(0), "root");
    }

    #[test]
    fn unknown_uid_falls_back_to_number_string() {
        assert_eq!(user_name(u32::MAX), u32::MAX.to_string());
    }

    #[test]
    fn unknown_gid_falls_back_to_number_string() {
        assert_eq!(group_name(u32::MAX), u32::MAX.to_string());
    }

    use chrono::TimeZone;

    fn local_dt(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn system_time_from(dt: DateTime<Local>) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64)
    }

    #[test]
    fn same_year_shows_time_of_day() {
        let now = local_dt(2026, 5, 5, 12, 0);
        let file = system_time_from(local_dt(2026, 1, 3, 14, 23));
        assert_eq!(format_time(file, now), "Jan  3 14:23");
    }

    #[test]
    fn past_year_shows_the_year() {
        let now = local_dt(2026, 5, 5, 12, 0);
        let file = system_time_from(local_dt(2024, 1, 3, 14, 23));
        assert_eq!(format_time(file, now), "Jan  3 2024");
    }

    fn sized_entry(name: &str, size: u64) -> FileEntry {
        let mut e = entry(name, "file");
        e.size = size;
        e.nlink = 1;
        e
    }

    #[test]
    fn long_format_empty_list_only_has_total_header() {
        let result = long_format(&[]);
        assert_eq!(result, "total 0\n");
    }

    #[test]
    fn long_format_starts_with_total_header() {
        let result = long_format(&[sized_entry("a.txt", 47)]);
        assert!(result.starts_with("total "));
    }

    #[test]
    fn long_format_total_line_sums_blocks_across_entries() {
        // total uses e.blocks / 2 (512-byte units → 1024-byte units)
        let mut small = sized_entry("small.txt", 1);
        small.blocks = 2; // 1 KiB block
        let mut medium = sized_entry("medium.bin", 1025);
        medium.blocks = 4; // 2 KiB blocks
        let mut big = sized_entry("big.bin", 4096);
        big.blocks = 8; // 4 KiB blocks
        let result = long_format(&[small, medium, big]);
        assert!(result.starts_with("total 7\n")); // (2+4+8)/2 = 7
    }

    #[test]
    fn long_format_produces_one_line_per_entry_plus_header() {
        let entries = [
            sized_entry("a.txt", 10),
            sized_entry("b.txt", 20),
            sized_entry("c.txt", 30),
        ];
        let result = long_format(&entries);
        assert_eq!(result.lines().count(), 4); // 1 total + 3 entries
    }

    #[test]
    fn long_format_includes_each_file_name() {
        let entries = [sized_entry("alpha.txt", 1), sized_entry("beta.bin", 2)];
        let result = long_format(&entries);
        assert!(result.contains("alpha.txt"));
        assert!(result.contains("beta.bin"));
    }

    #[test]
    fn long_format_includes_size_in_output() {
        let result = long_format(&[sized_entry("file.txt", 12345)]);
        assert!(result.contains("12345"));
    }

    #[test]
    fn long_format_always_ends_with_newline() {
        let result = long_format(&[sized_entry("a.txt", 1)]);
        assert!(result.ends_with('\n'));
    }
}
