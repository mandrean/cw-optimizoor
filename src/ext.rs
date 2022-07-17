use unicode_segmentation::UnicodeSegmentation;

pub trait StringRTake {
    /// Returns the last n "characters" (actually graphemes) of the string.
    fn rtake(&self, n: usize) -> String;
}

impl StringRTake for String {
    /// Returns the last n "characters" (actually graphemes) of the string.
    fn rtake(&self, n: usize) -> String {
        let gs = self.graphemes(true).collect::<Vec<&str>>();
        gs[gs.len() - n..].concat()
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::StringRTake;

    #[test]
    fn returns_last_n_graphemes() {
        assert_eq!("World 👋", "Hello World 👋".to_string().rtake(7));
    }
}
