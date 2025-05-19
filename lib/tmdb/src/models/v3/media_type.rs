use serde::{Deserialize, Deserializer};
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MediaType {
    Movie,
    Tv,
    Unknown(String),
}

impl MediaType {
    pub fn tmdb_url_prefix(&self) -> Option<&'static str> {
        match self {
            MediaType::Movie => Some("movie"),
            MediaType::Tv => Some("tv"),
            MediaType::Unknown(_) => None,
        }
    }
}

impl FromStr for MediaType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[inline]
        fn test(s: &str) -> Option<MediaType> {
            match s {
                "movie" => Some(MediaType::Movie),
                "tv" => Some(MediaType::Tv),
                _ => None,
            }
        }

        let s = s.trim();

        Ok(test(s)
            .or_else(|| test(s.to_lowercase().as_str()))
            .unwrap_or(Self::Unknown(s.to_string())))
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Movie => f.write_str("Movie"),
            MediaType::Tv => f.write_str("TV"),
            MediaType::Unknown(s) => write!(f, "Unknown({s})"),
        }
    }
}

impl<'de> Deserialize<'de> for MediaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;

        Ok(MediaType::from_str(s).expect("infallible"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infallible() {
        assert!(MediaType::from_str("Lorem ipsum dolor sit amet, consectetur adipiscing").is_ok());
    }

    #[test]
    fn test_movie() {
        assert_eq!(MediaType::from_str("movie").unwrap(), MediaType::Movie);
    }

    #[test]
    fn test_tv() {
        assert_eq!(MediaType::from_str("tv").unwrap(), MediaType::Tv);
    }

    #[test]
    fn test_to_lowercase() {
        assert_eq!(MediaType::from_str("TV").unwrap(), MediaType::Tv);
    }

    #[test]
    fn test_trimmed_whitespace() {
        assert_eq!(
            MediaType::from_str("    movie   ").unwrap(),
            MediaType::Movie
        );
    }

    #[test]
    fn test_unknown() {
        assert_eq!(
            MediaType::from_str("radio").unwrap(),
            MediaType::Unknown("radio".to_string())
        );
    }

    #[test]
    fn test_tmdb_url_prefix() {
        assert_eq!(MediaType::Tv.tmdb_url_prefix(), Some("tv"));
        assert_eq!(MediaType::Movie.tmdb_url_prefix(), Some("movie"));
        assert_eq!(
            MediaType::Unknown("UNKNOWN".to_string()).tmdb_url_prefix(),
            None
        );
    }
}
