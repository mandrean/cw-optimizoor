use std::path::PathBuf;

pub trait RTake<R> {
    /// Returns the last n items of a slice.
    fn rtake(&self, n: usize) -> R;
}

impl<T> RTake<Vec<T>> for [T]
where
    T: Clone,
{
    fn rtake(&self, n: usize) -> Vec<T> {
        (&self[self.len() - n..]).to_vec()
    }
}

impl RTake<PathBuf> for PathBuf {
    fn rtake(&self, n: usize) -> PathBuf {
        self.iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>()
            .rtake(n)
            .iter()
            .fold(PathBuf::new(), |acc, i| acc.join(i))
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::RTake;

    #[test]
    fn returns_last_n_items() {
        assert_eq!(
            vec!["The", "Manager"],
            vec!["May", "I", "Speak", "To", "The", "Manager"].rtake(2)
        );
    }
}
