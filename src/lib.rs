use std::fs::{metadata, File};
use std::io::{BufReader, Read, Stdin};
use std::path::PathBuf;

mod config;
pub mod count;
pub mod error;

pub use config::Config;
pub use count::file;
pub use count::files;

const BUFFER_SIZE: usize = 1048576;

#[derive(Debug)]
pub struct Counted {
    pub bytes: Option<usize>,
    pub chars: Option<usize>,
    pub words: Option<usize>,
    pub lines: Option<usize>,
}

pub trait Count<T> {
    fn count(self) -> Counted;
}

pub trait Countable<T> {
    fn countable(self) -> Counter<T>;
}

impl Countable<Stdin> for Stdin {
    fn countable(self) -> Counter<Stdin> {
        Counter {
            data: self,
            bytes: false,
            chars: false,
            words: false,
            lines: false,
        }
    }
}
impl Countable<File> for File {
    fn countable(self) -> Counter<File> {
        Counter {
            data: self,
            bytes: false,
            chars: false,
            words: false,
            lines: false,
        }
    }
}
impl Countable<PathBuf> for PathBuf {
    fn countable(self) -> Counter<PathBuf> {
        Counter {
            data: self,
            bytes: false,
            chars: false,
            words: false,
            lines: false,
        }
    }
}

#[derive(Debug)]
pub struct Counter<T> {
    data: T,
    bytes: bool,
    chars: bool,
    words: bool,
    lines: bool,
}

impl Count<PathBuf> for Counter<PathBuf> {
    fn count(self) -> Counted {
        if self.bytes && !(self.chars || self.words || self.lines) {
            let bytes = metadata(self.data).unwrap().len() as usize; // TODO
            Counted {
                bytes: Some(bytes),
                chars: None,
                words: None,
                lines: None,
            }
        } else {
            count_readable(Counter {
                data: File::open(self.data).unwrap(), // TODO
                bytes: self.bytes,
                chars: self.chars,
                words: self.words,
                lines: self.lines,
            })
        }
    }
}

impl Count<File> for Counter<File> {
    fn count(self) -> Counted {
        count_readable(self)
    }
}

impl Count<Stdin> for Counter<Stdin> {
    fn count(self) -> Counted {
        count_readable(self)
    }
}

impl<T> Counter<T> {
    fn bytes(self) -> Counter<T> {
        Counter {
            bytes: true,
            ..self
        }
    }
    fn chars(self) -> Counter<T> {
        Counter {
            chars: true,
            ..self
        }
    }
    fn words(self) -> Counter<T> {
        Counter {
            words: true,
            ..self
        }
    }
    fn lines(self) -> Counter<T> {
        Counter {
            lines: true,
            ..self
        }
    }
}

fn count_readable<R: Read>(counter: Counter<R>) -> Counted {
    if counter.chars {
        let reader = BufReader::with_capacity(BUFFER_SIZE, counter.data);
        let (bytes, chars, words, lines) = count::utf8(reader);
        Counted {
            bytes: if counter.bytes { Some(bytes) } else { None },
            chars: if counter.chars { Some(chars) } else { None },
            words: if counter.words { Some(words) } else { None },
            lines: if counter.lines { Some(lines) } else { None },
        }
    } else {
        let reader = BufReader::with_capacity(BUFFER_SIZE, counter.data);
        let (bytes, words, lines) = count::binary(reader);
        Counted {
            bytes: if counter.bytes { Some(bytes) } else { None },
            chars: None,
            words: if counter.words { Some(words) } else { None },
            lines: if counter.lines { Some(lines) } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correct_defaults_for_counter() {
        let c = std::io::stdin().countable();
        assert_eq!(
            (false, false, false, false),
            (c.bytes, c.chars, c.words, c.lines)
        );
        let c = File::open("src/lib.rs").unwrap().countable();
        assert_eq!(
            (false, false, false, false),
            (c.bytes, c.chars, c.words, c.lines)
        );
        let c = PathBuf::new().countable();
        assert_eq!(
            (false, false, false, false),
            (c.bytes, c.chars, c.words, c.lines)
        );
    }

    #[test]
    fn correct_counter_changes() {
        let c = PathBuf::new().countable().bytes();
        assert_eq!(
            (true, false, false, false),
            (c.bytes, c.chars, c.words, c.lines)
        );
        let c = PathBuf::new().countable().chars();
        assert_eq!(
            (false, true, false, false),
            (c.bytes, c.chars, c.words, c.lines)
        );
        let c = PathBuf::new().countable().words();
        assert_eq!(
            (false, false, true, false),
            (c.bytes, c.chars, c.words, c.lines)
        );
        let c = PathBuf::new().countable().lines();
        assert_eq!(
            (false, false, false, true),
            (c.bytes, c.chars, c.words, c.lines)
        );
    }

    #[test]
    fn correct_count_for_only_bytes() {
        let path: PathBuf = ["test_data", "default.txt"].iter().collect();
        let bytes = path.countable().bytes().count().bytes.unwrap();
        assert_eq!(1048697, bytes);
    }

    #[test]
    fn correct_count_for_utf8_file() {
        let path: PathBuf = ["test_data", "default.txt"].iter().collect();
        let c = path.countable().bytes().chars().words().lines().count();
        assert_eq!(1048697, c.bytes.unwrap());
        assert_eq!(726780, c.chars.unwrap());
        assert_eq!(183155, c.words.unwrap());
        assert_eq!(20681, c.lines.unwrap());
    }

    #[test]
    fn correct_count_for_binary_file() {
        let path: PathBuf = ["test_data", "default.txt"].iter().collect();
        let c = path.countable().bytes().chars().words().lines().count();
        assert_eq!(1048697, c.bytes.unwrap());
        assert_eq!(183155, c.words.unwrap());
        assert_eq!(20681, c.lines.unwrap());
    }
}
