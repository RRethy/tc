#[derive(Debug)]
pub struct Config {
    pub bytes: bool,
    pub chars: bool,
    pub words: bool,
    pub tokens: bool,
    pub lines: bool,
}

impl Config {
    pub fn to_vec(&self) -> Vec<&str> {
        let mut vec = Vec::new();
        if self.bytes {
            vec.push("bytes");
        }
        if self.chars {
            vec.push("chars");
        }
        if self.words {
            vec.push("words");
        }
        if self.lines {
            vec.push("lines");
        }
        vec
    }
}
