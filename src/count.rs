use crate::config::Config;
use bytecount;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;
use utf8::{BufReadDecoder, BufReadDecoderError};

const BUFFER_SIZE: usize = 1048576;

// count bytes, words, lines
pub(crate) fn binary<T: Read>(mut reader: BufReader<T>) -> (usize, usize, usize) {
    let (mut bytes, mut words, mut lines) = (0, 0, 0);
    let mut in_word = false;
    loop {
        let buffer = match reader.fill_buf() {
            Ok(b) => b,
            Err(_) => return (0, 0, 0), // TODO
        };
        let len = buffer.len();
        if len == 0 {
            break;
        }
        bytes += len;
        for &b in buffer {
            lines += if b == b'\n' { 1 } else { 0 };
            if b.is_ascii_whitespace() {
                words += if in_word { 1 } else { 0 };
                in_word = false;
            } else {
                in_word = true;
            }
        }
        reader.consume(len);
    }
    if in_word {
        words += 1;
    }
    (bytes, words, lines)
}

// count bytes, chars, words, lines
pub(crate) fn utf8<T: Read>(reader: BufReader<T>) -> (usize, usize, usize, usize) {
    let (mut bytes, mut chars, mut words, mut lines) = (0, 0, 0, 0);
    let mut in_word = false;
    let mut decoder = BufReadDecoder::new(reader);
    loop {
        if let Some(res) = decoder.next_strict() {
            match res {
                Ok(str) => {
                    bytes += str.len();
                    for c in str.chars() {
                        chars += 1;
                        lines += if c == '\n' { 1 } else { 0 };
                        if c.is_ascii_whitespace() {
                            words += if in_word { 1 } else { 0 };
                            in_word = false;
                        } else {
                            in_word = true;
                        }
                    }
                }
                Err(_) => return (0, 0, 0, 0), // TODO fail over to binary file
            }
        } else {
            break;
        }
    }
    if in_word {
        words += 1;
    }
    (bytes, chars, words, lines)
}

// count bytes, lines
pub(crate) fn hyperscreamingcount<T: Read>(mut reader: BufReader<T>) -> (usize, usize) {
    let (mut bytes, mut lines) = (0, 0);
    loop {
        let buffer = match reader.fill_buf() {
            Ok(b) => b,
            Err(_) => return (0, 0), // TODO
        };
        let len = buffer.len();
        if len == 0 {
            break;
        }
        bytes += len;
        lines += bytecount::count(buffer, b'\n');
        reader.consume(len);
    }
    (bytes, lines)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Context<'pathbuf> {
    File { path: &'pathbuf PathBuf },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Count<'pathbuf> {
    pub context: Context<'pathbuf>,
    pub bytes: Option<usize>,
    pub chars: Option<usize>,
    pub words: Option<usize>,
    pub lines: Option<usize>,
}

impl<'pathbuf> Count<'pathbuf> {
    pub fn to_counts_vec(&self) -> Vec<usize> {
        let mut vec = Vec::new();
        if let Some(bytes) = self.bytes {
            vec.push(bytes)
        }
        if let Some(chars) = self.chars {
            vec.push(chars)
        }
        if let Some(words) = self.words {
            vec.push(words)
        }
        if let Some(lines) = self.lines {
            vec.push(lines)
        }
        vec
    }

    pub fn to_str_vec(&self) -> Vec<String> {
        let mut vec = vec![self.groupname()];
        vec.extend(self.to_counts_vec().iter().map(ToString::to_string));
        vec
    }

    pub fn groupname(&self) -> String {
        match self.context {
            Context::File { path } => path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    NotUtf8EncodedFile,
    Io(io::Error),
}

fn binary_file<'a>(
    path: &'a PathBuf,
    only_bytes: bool,
) -> Result<(usize, usize, usize, usize), Error> {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut lines = 0;
    let mut bytes = 0;
    let mut words = 0;
    let mut in_word = false;

    loop {
        let buffer = match reader.fill_buf() {
            Ok(b) => b,
            Err(e) => return Err(Error::Io(e)),
        };
        let len = buffer.len();
        if len == 0 {
            break;
        }
        bytes += len;
        if !only_bytes {
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
        }
        reader.consume(len);
    }
    Ok((bytes, 0, words, lines))
}

fn utf8_file<'a>(path: &'a PathBuf) -> Result<(usize, usize, usize, usize), Error> {
    let file = File::open(path).unwrap();
    let reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut decoder = BufReadDecoder::new(reader);
    let mut lines = 0;
    let mut bytes = 0;
    let mut words = 0;
    let mut chars = 0;
    let mut in_word = false;
    loop {
        if let Some(res) = decoder.next_strict() {
            match res {
                Ok(str) => {
                    bytes += str.len();
                    for c in str.chars() {
                        chars += 1;
                        if c == '\n' {
                            lines += 1;
                        }
                        if c.is_ascii_whitespace() {
                            if in_word {
                                words += 1;
                                in_word = false;
                            }
                        } else {
                            in_word = true;
                        }
                    }
                }
                Err(e) => {
                    match e {
                        BufReadDecoderError::InvalidByteSequence(_) => {
                            return Err(Error::NotUtf8EncodedFile)
                        }
                        BufReadDecoderError::Io(ioerr) => return Err(Error::Io(ioerr)),
                    };
                }
            }
        } else {
            break;
        }
    }
    Ok((bytes, chars, words, lines))
}

pub fn files<'a>(paths: &'a Vec<PathBuf>, config: &Config) -> Vec<Result<Count<'a>, Error>> {
    paths.into_iter().map(|path| file(path, config)).collect()
}

pub fn file<'a>(path: &'a PathBuf, config: &Config) -> Result<Count<'a>, Error> {
    let (bytes, chars, words, lines) = if config.chars {
        utf8_file(path)?
    } else {
        binary_file(path, !config.words && !config.lines)?
    };
    Ok(Count {
        context: Context::File { path },
        bytes: if config.bytes { Some(bytes) } else { None },
        chars: if config.chars { Some(chars) } else { None },
        words: if config.words { Some(words) } else { None },
        lines: if config.lines { Some(lines) } else { None },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_all_true() -> Config {
        Config {
            bytes: true,
            chars: true,
            words: true,
            lines: true,
        }
    }

    fn config_all_false() -> Config {
        Config {
            bytes: false,
            chars: false,
            words: false,
            lines: false,
        }
    }

    fn count_empty(path: &PathBuf) -> Count {
        Count {
            context: Context::File { path },
            bytes: None,
            chars: None,
            words: None,
            lines: None,
        }
    }

    fn count_for_default_file(path: &PathBuf) -> Count {
        Count {
            context: Context::File { path },
            bytes: Some(1048697),
            chars: Some(726780),
            words: Some(183155),
            lines: Some(20681),
        }
    }

    fn default_file_path() -> PathBuf {
        ["test_data", "default.txt"].iter().collect()
    }

    #[test]
    fn it_counts_file() {
        let path: PathBuf = default_file_path();
        let count = file(&path, &config_all_true());
        assert_eq!(count.unwrap(), count_for_default_file(&path));
    }

    #[test]
    fn it_counts_bytes_in_file() {
        let path: PathBuf = default_file_path();
        let count = file(
            &path,
            &Config {
                bytes: true,
                ..config_all_false()
            },
        );
        assert_eq!(
            count.unwrap(),
            Count {
                bytes: Some(1048697),
                ..count_empty(&path)
            }
        );
    }

    #[test]
    fn it_counts_chars_in_file() {
        let path: PathBuf = default_file_path();
        let count = file(
            &path,
            &Config {
                chars: true,
                ..config_all_false()
            },
        );
        assert_eq!(
            count.unwrap(),
            Count {
                chars: Some(726780),
                ..count_empty(&path)
            }
        );
    }

    #[test]
    fn it_counts_words_in_file() {
        let path: PathBuf = default_file_path();
        let count = file(
            &path,
            &Config {
                words: true,
                ..config_all_false()
            },
        );
        assert_eq!(
            count.unwrap(),
            Count {
                words: Some(183155),
                ..count_empty(&path)
            }
        );
    }

    #[test]
    fn it_counts_lines_in_file() {
        let path: PathBuf = default_file_path();
        let count = file(
            &path,
            &Config {
                lines: true,
                ..config_all_false()
            },
        );
        assert_eq!(
            count.unwrap(),
            Count {
                lines: Some(20681),
                ..count_empty(&path)
            }
        );
    }

    #[test]
    fn binary_reader_has_correct_counts() {
        let text: &[u8] =
            "hello???????????????????????????????????????????????? hello world 12345\n67890???? ???? ???? ????".as_bytes();
        let reader = BufReader::with_capacity(10, text);
        let (bytes, words, lines) = binary(reader);
        assert_eq!(
            96, bytes,
            "expected byte count does not match actual byte count"
        );
        assert_eq!(
            8, words,
            "expected word count does not match actual word count"
        );
        assert_eq!(
            1, lines,
            "expected line count does not match actual line count"
        );
    }

    #[test]
    fn utf8_reader_has_correct_counts() {
        let text: &[u8] =
            "hello???????????????????????????????????????????????? hello world 12345\n67890???? ???? ???? ????".as_bytes();
        let reader = BufReader::with_capacity(10, text);
        let (bytes, chars, words, lines) = utf8(reader);
        assert_eq!(
            96, bytes,
            "expected byte count does not match actual byte count"
        );
        assert_eq!(
            48, chars,
            "expected char count does not match actual char count"
        );
        assert_eq!(
            8, words,
            "expected word count does not match actual word count"
        );
        assert_eq!(
            1, lines,
            "expected line count does not match actual line count"
        );
    }

    #[test]
    fn hyperscreamingcount_has_correct_counts() {
        let text: &[u8] =
            "hello???????????????????????????????????????????????? hello world 12345\n67890???? ???? ???? ????".as_bytes();
        let reader = BufReader::with_capacity(10, text);
        let (bytes, lines) = hyperscreamingcount(reader);
        assert_eq!(
            96, bytes,
            "expected byte count does not match actual byte count"
        );
        assert_eq!(
            1, lines,
            "expected line count does not match actual line count"
        );
    }
}
