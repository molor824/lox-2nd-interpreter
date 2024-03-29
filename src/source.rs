use std::str::CharIndices;

// Handles iterating over the source code
// converts CRLF to LF (and CR to LF)
#[derive(Debug, Clone)]
pub struct SourceIter<'a> {
    pub iter: CharIndices<'a>,
}
impl<'a> From<&'a str> for SourceIter<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            iter: value.char_indices(),
        }
    }
}
impl<'a> Iterator for SourceIter<'a> {
    type Item = (usize, char);
    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.iter.next();

        if let Some((i, '\r')) = ch {
            let old = self.iter.clone();

            if let Some((i, '\n')) = self.iter.next() {
                return Some((i, '\n'));
            }

            self.iter = old;
            return Some((i, '\n'));
        }

        ch
    }
}
