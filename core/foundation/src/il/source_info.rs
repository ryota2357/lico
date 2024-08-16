use rowan::TextRange;
use rustc_hash::FxHashMap;

// TODO: compiler/src/compile/icodesource.rs にメモしてある extra の部分の使用を明文化する

pub struct SourceInfo {
    data: FxHashMap<(usize, usize), TextRange>,
}

impl SourceInfo {
    pub fn new() -> Self {
        SourceInfo {
            data: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, index: usize, extra: usize, range: TextRange) -> Option<TextRange> {
        self.data.insert((index, extra), range)
    }

    pub fn get(&self, index: usize, extra: usize) -> Option<TextRange> {
        self.data.get(&(index, extra)).cloned()
    }
}

impl Default for SourceInfo {
    fn default() -> Self {
        Self::new()
    }
}
