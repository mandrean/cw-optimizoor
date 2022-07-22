use std::path::PathBuf;

pub trait TakeExt<R> {
    /// Returns the first n items in `R`.
    fn ltake(&self, n: usize) -> R;

    /// Returns the last n items in `R`.
    fn rtake(&self, n: usize) -> R;

    /// Skips the first n items in `R`.
    fn lskip(&self, n: usize) -> R;

    /// Skips the last n items in `R`.
    fn rskip(&self, n: usize) -> R;
}

impl<T> TakeExt<Vec<T>> for [T]
where
    T: Clone,
{
    fn ltake(&self, n: usize) -> Vec<T> {
        self[0..n].to_vec()
    }

    fn rtake(&self, n: usize) -> Vec<T> {
        self[self.len() - n..].to_vec()
    }

    fn lskip(&self, n: usize) -> Vec<T> {
        self[n..].to_vec()
    }

    fn rskip(&self, n: usize) -> Vec<T> {
        self[0..self.len() - n].to_vec()
    }
}

impl TakeExt<PathBuf> for PathBuf {
    fn ltake(&self, n: usize) -> PathBuf {
        self.iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>()
            .ltake(n)
            .iter()
            .fold(PathBuf::new(), |acc, i| acc.join(i))
    }

    fn rtake(&self, n: usize) -> PathBuf {
        self.iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>()
            .rtake(n)
            .iter()
            .fold(PathBuf::new(), |acc, i| acc.join(i))
    }

    fn lskip(&self, n: usize) -> PathBuf {
        self.iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>()
            .lskip(n)
            .iter()
            .fold(PathBuf::new(), |acc, i| acc.join(i))
    }

    fn rskip(&self, n: usize) -> PathBuf {
        self.iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>()
            .rskip(n)
            .iter()
            .fold(PathBuf::new(), |acc, i| acc.join(i))
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::TakeExt;

    #[test]
    fn returns_first_n_items() {
        assert_eq!(
            vec!["May", "I", "Speak"],
            vec!["May", "I", "Speak", "To", "The", "Manager"].ltake(3)
        );
    }

    #[test]
    fn returns_last_n_items() {
        assert_eq!(
            vec!["The", "Manager"],
            vec!["May", "I", "Speak", "To", "The", "Manager"].rtake(2)
        );
    }

    #[test]
    fn skips_first_n_items() {
        assert_eq!(
            vec!["The", "Manager"],
            vec!["May", "I", "Speak", "To", "The", "Manager"].lskip(4)
        );
    }

    #[test]
    fn skips_last_n_items() {
        assert_eq!(
            vec!["May", "I", "Speak"],
            vec!["May", "I", "Speak", "To", "The", "Manager"].rskip(3)
        );
    }
}
