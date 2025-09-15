use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use std::num::{NonZeroUsize, TryFromIntError};
use std::ops::Deref;
use thiserror::Error;
use utils::const_assert;

const DEFAULT_SIZE_USIZE: usize = 20;
const MAX_SIZE_USIZE: usize = 50;

const_assert!(
    DEFAULT_SIZE_USIZE <= MAX_SIZE_USIZE,
    "MAX_SIZE must not exceed DEFAULT_SIZE"
);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(super) struct Size(NonZeroUsize);

impl Size {
    const DEFAULT_SIZE: NonZeroUsize = NonZeroUsize::new(DEFAULT_SIZE_USIZE).unwrap();
    pub(super) const MAX_SIZE: NonZeroUsize = NonZeroUsize::new(MAX_SIZE_USIZE).unwrap();
}

impl Default for Size {
    fn default() -> Self {
        Self(NonZeroUsize::try_from(Size::DEFAULT_SIZE).unwrap())
    }
}

impl Deref for Size {
    type Target = NonZeroUsize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<usize> for Size {
    type Error = SizeError;

    fn try_from(size: usize) -> Result<Self, Self::Error> {
        let size = NonZeroUsize::try_from(size).map_err(SizeError::from)?;

        Size::try_from(size)
    }
}

impl TryFrom<NonZeroUsize> for Size {
    type Error = SizeError;

    fn try_from(size: NonZeroUsize) -> Result<Self, Self::Error> {
        if size > Size::MAX_SIZE {
            Err(SizeError::ExceedsMaxSize)
        } else {
            Ok(Self(size))
        }
    }
}

impl<'de> Deserialize<'de> for Size {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let size = NonZeroUsize::deserialize(deserializer)?;
        Size::try_from(size).map_err(DeError::custom)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Error)]
pub(super) enum SizeError {
    #[error("size must not exceed the max size")]
    ExceedsMaxSize,
    #[error(transparent)]
    TryFromInt(#[from] TryFromIntError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lt_max() {
        assert!(Size::DEFAULT_SIZE <= Size::MAX_SIZE);
    }

    #[test]
    fn test_default() {
        assert_eq!(*Size::default(), Size::DEFAULT_SIZE);
    }

    #[test]
    fn test_exceeding_max() {
        let err = Size::try_from(Size::MAX_SIZE.get() + 1).unwrap_err();
        assert_eq!(err, SizeError::ExceedsMaxSize);
    }

    #[test]
    fn test_zero() {
        let err = Size::try_from(0).unwrap_err();
        assert_eq!(
            err.to_string(),
            "out of range integral type conversion attempted"
        );
    }
}
