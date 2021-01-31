use colored::*;
use num_format::{Locale, ToFormattedString};
use std::io::stdin;
use std::path::PathBuf;
use structopt::StructOpt;
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::Table;

#[derive(StructOpt)]
struct Cli {
    #[structopt(short, long)]
    bytes: bool,

    #[structopt(short, long)]
    chars: bool,

    #[structopt(short, long)]
    words: bool,

    #[structopt(short, long)]
    lines: bool,

    #[structopt(long)]
    stdin: bool,

    #[structopt(long)]
    file_from: PathBuf,

    files: Vec<PathBuf>,
}

impl From<&Cli> for tc::Config {
    fn from(cli: &Cli) -> Self {
        if !(cli.bytes || cli.chars || cli.words || cli.lines) {
            tc::Config {
                bytes: true,
                chars: true,
                words: true,
                lines: true,
            }
        } else {
            tc::Config {
                bytes: cli.bytes,
                chars: cli.chars,
                words: cli.words,
                lines: cli.lines,
            }
        }
    }
}

fn read_files_stdin() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    loop {
        let mut buf = String::new();
        let n = stdin().read_line(&mut buf).unwrap();
        if n == 0 {
            break;
        } else {
            if buf.ends_with('\n') {
                buf.pop();
            }
            if buf.len() > 0 {
                // TODO ensure it's a valid pathbuf, maybe this should be done in the lib
                paths.push(PathBuf::from(buf));
            }
        }
    }
    paths
}

fn pprint(results: Vec<Result<tc::count::Count, tc::count::Error>>, config: &tc::Config) {
    let mut table = Table::new();
    table.style = term_table::TableStyle::thin();
    let mut headers: Vec<TableCell> = Vec::new();
    headers.reserve_exact(results.len() + 1);
    headers.push(TableCell::new(""));
    headers.extend(
        config.to_vec().iter().map(|s| {
            TableCell::new_with_alignment(s.blue().bold().underline(), 1, Alignment::Center)
        }),
    );
    table.add_row(Row::new(headers));
    for res in &results {
        if let Ok(counts) = res {
            let mut cells: Vec<TableCell> = Vec::new();
            cells.reserve_exact(results.len() + 1);
            cells.push(TableCell::new_with_alignment(
                counts.groupname().cyan().italic().bold(),
                1,
                Alignment::Left,
            ));
            cells.extend(counts.to_counts_vec().iter().map(|count| {
                TableCell::new_with_alignment(
                    count.to_formatted_string(&Locale::en),
                    1,
                    Alignment::Right,
                )
            }));
            table.add_row(Row::new(cells));
        }
    }
    println!("{}", table.render());
}

fn run() -> Result<(), tc::error::Error> {
    let cli = Cli::from_args();
    let config = tc::Config::from(&cli);
    let mut files = cli.files;
    if files.len() == 0 && !cli.stdin {
        files.extend(read_files_stdin());
    }
    let results = tc::count::files(&files, &config);
    pprint(results, &config);
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(_) => {}
    }
}
