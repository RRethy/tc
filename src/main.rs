use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    bytes: bool,

    #[structopt(short, long)]
    chars: bool,

    #[structopt(short, long)]
    words: bool,

    #[structopt(short, long)]
    tokens: bool,

    #[structopt(short, long)]
    lines: bool,

    #[structopt(short, long, parse(from_os_str))]
    files: Option<Vec<PathBuf>>,
}

impl From<&Cli> for tc::Config {
    fn from(cli: &Cli) -> Self {
        tc::Config {
            bytes: cli.bytes,
            chars: cli.chars,
            words: cli.words,
            tokens: cli.tokens,
            lines: cli.lines,
        }
    }
}

fn main() {
    let cli = Cli::from_args();
    if let Some(files) = &cli.files {
        let counts = tc::count_files(files, &tc::Config::from(&cli));
        for count in counts {
            println!("{:?}", count);
        }
    }
}
