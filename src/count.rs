use crate::config::Config;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const BUFFER_SIZE: usize = 512 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub enum Context<'a> {
    File { path: &'a PathBuf },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Count<'a> {
    context: Context<'a>,
    bytes: Option<usize>,
    words: Option<usize>,
    lines: Option<usize>,
}

pub fn files<'a>(paths: &'a Vec<PathBuf>, config: &Config) -> Vec<Count<'a>> {
    paths.into_iter().map(|path| file(path, config)).collect()
}

pub fn file<'a>(path: &'a PathBuf, config: &Config) -> Count<'a> {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut lines = 0;
    let mut bytes = 0;
    let mut words = 0;
    let mut in_word = false;
    loop {
        let buffer = reader.fill_buf().unwrap();
        let l = buffer.len();
        if l == 0 {
            break;
        }
        bytes += l;
        for &b in buffer {
            if b == b'\n' {
                lines += 1;
            }
            if b.is_ascii_whitespace() {
                if in_word {
                    words += 1;
                    in_word = false;
                }
            } else {
                in_word = true;
            }
        }
        reader.consume(l);
    }
    Count {
        context: Context::File { path },
        bytes: if config.bytes { Some(bytes) } else { None },
        words: if config.words { Some(words) } else { None },
        lines: if config.lines { Some(lines) } else { None },
    }
}

// pub fn file<'a>(f: &'a PathBuf, config: &Config) -> Count<'a> {
//     let text = &std::fs::read(&f).unwrap();
//     let mut lines = 0;
//     let bytes = text.len();
//     let mut words = 0;
//     let mut in_word = false;
//     for &b in text {
//         if b == b'\n' {
//             lines += 1;
//         }
//         if b.is_ascii_whitespace() {
//             if in_word {
//                 words += 1;
//                 in_word = false;
//             }
//         } else {
//             in_word = true;
//         }
//     }
//     Count {
//         context: Context::File { path: &f },
//         bytes: if config.bytes { Some(bytes) } else { None },
//         words: if config.words { Some(words) } else { None },
//         lines: if config.lines { Some(lines) } else { None },
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config {
            bytes: true,
            chars: true,
            words: true,
            tokens: true,
            lines: true,
        }
    }

    fn count_for_tiny_file(path: &PathBuf) -> Count {
        Count {
            context: Context::File { path: path },
            bytes: Some(172),
            words: Some(33),
            lines: Some(5),
        }
    }

    fn tiny_file_path() -> PathBuf {
        ["test_data", "tiny.txt"].iter().collect()
    }

    #[test]
    fn it_counts_file() {
        let path: PathBuf = tiny_file_path();
        let count = file(&path, &default_config());
        assert_eq!(count, count_for_tiny_file(&path),);
    }

    #[test]
    fn it_does_not_count_bytes_in_file() {
        let path: PathBuf = tiny_file_path();
        let count = file(
            &path,
            &Config {
                bytes: false,
                ..default_config()
            },
        );
        assert_eq!(
            count,
            Count {
                bytes: None,
                ..count_for_tiny_file(&path)
            }
        );
    }

    #[test]
    fn it_does_not_count_words_in_file() {
        let path: PathBuf = tiny_file_path();
        let count = file(
            &path,
            &Config {
                words: false,
                ..default_config()
            },
        );
        assert_eq!(
            count,
            Count {
                words: None,
                ..count_for_tiny_file(&path)
            }
        );
    }

    #[test]
    fn it_does_not_count_lines_in_file() {
        let path: PathBuf = tiny_file_path();
        let count = file(
            &path,
            &Config {
                lines: false,
                ..default_config()
            },
        );
        assert_eq!(
            count,
            Count {
                lines: None,
                ..count_for_tiny_file(&path)
            }
        );
    }
}
