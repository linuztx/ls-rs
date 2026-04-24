use clap::Parser;


#[derive(Parser)]
#[command(name = "ls-rs", about = "A modern ls replacement written in Rust")]
pub struct Args {
    #[arg(default_value = ".")]
    pub path: String,

    #[arg(short = 'a', long)]
    pub all: bool,

    #[arg(short = 'l', long)]
    pub long: bool,
}


#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn defaults_when_no_args() {
        let args = Args::parse_from(["ls-rs"]);
        assert_eq!(args.path, ".");
        assert!(!args.all);
        assert!(!args.long);
    }


    #[test]
    fn parse_flags() {
        let args = Args::parse_from(["ls-rs", "-a", "-l", "/tmp"]);
        assert_eq!(args.path, "/tmp");
        assert!(args.all);
        assert!(args.long);
    }
}
