use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Hash)]
pub struct GenreId(usize);

impl From<usize> for GenreId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Deref for GenreId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for GenreId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let id = 5usize;

        let genre_id = GenreId::from(id);
        assert_eq!(genre_id.0, id);
    }

    #[test]
    fn test_deref() {
        let id = 5usize;

        let genre_id = GenreId::from(id);
        assert_eq!(*genre_id, id);
    }
}
