use crate::SITE_URL;
use crate::models::v3::genre_id::GenreId;
use crate::models::v3::genres::{MovieGenre, TvGenre};
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};
use url::Url;

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize, Hash)]
#[serde(tag = "media_type")]
pub enum Cast {
    #[serde(rename = "movie")]
    Movie(MovieCast),
    #[serde(rename = "tv")]
    Tv(TvCast),
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize, Hash)]
pub struct MovieCast {
    pub id: usize,
    pub title: String,
    pub original_title: String,
    pub character: String,
    #[serde(deserialize_with = "deserialize_movie_genre", flatten)]
    pub genres: Vec<MovieGenre>,
    #[serde(deserialize_with = "deserialize_release_date")]
    pub release_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
    pub credit_id: String,
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize, Hash)]
pub struct TvCast {
    pub id: usize,
    pub name: String,
    pub original_name: String,
    pub character: String,
    #[serde(deserialize_with = "deserialize_tv_genre", flatten)]
    pub genres: Vec<TvGenre>,
    #[serde(deserialize_with = "deserialize_release_date")]
    pub first_air_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
    pub credit_id: String,
}

pub(super) fn deserialize_movie_genre<'de, D>(deserializer: D) -> Result<Vec<MovieGenre>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Data {
        genre_ids: Vec<GenreId>,
    }

    let Data { genre_ids } = Data::deserialize(deserializer)?;

    Ok(genre_ids.into_iter().map(MovieGenre::from).collect())
}

pub(super) fn deserialize_tv_genre<'de, D>(deserializer: D) -> Result<Vec<TvGenre>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Data {
        genre_ids: Vec<GenreId>,
    }

    let Data { genre_ids } = Data::deserialize(deserializer)?;

    Ok(genre_ids.into_iter().map(TvGenre::from).collect())
}

pub(super) fn deserialize_release_date<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let date: &str = Deserialize::deserialize(deserializer)?;

    if date.is_empty() {
        return Ok(None);
    }

    Ok(NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
}

pub trait IsCredit {}

impl IsCredit for MovieCast {}
impl IsCredit for TvCast {}

pub trait MediaTypeDefinition {
    const MEDIA_TYPE: MediaType;

    #[inline]
    fn media_type(&self) -> MediaType {
        Self::MEDIA_TYPE
    }
}

impl MediaTypeDefinition for MovieCast {
    const MEDIA_TYPE: MediaType = MediaType::Movie;
}

impl MediaTypeDefinition for TvCast {
    const MEDIA_TYPE: MediaType = MediaType::Tv;
}

pub trait MediaPageUrl<T: MediaTypeDefinition = Self> {
    fn tmdb_media_url(&self) -> Url;
}

impl MediaPageUrl for MovieCast {
    fn tmdb_media_url(&self) -> Url {
        let media_url_prefix = Self::MEDIA_TYPE.tmdb_url_prefix().expect(
            "Self::MEDIA_TYPE is const and is guaranteed by tests to always return Some(_)",
        );

        SITE_URL
            .join(format!("/{media_url_prefix}/{}", self.id).as_str())
            .expect("url guaranteed to be valid")
    }
}

impl MediaPageUrl for TvCast {
    fn tmdb_media_url(&self) -> Url {
        let media_url_prefix = Self::MEDIA_TYPE.tmdb_url_prefix().expect(
            "Self::MEDIA_TYPE is const and is guaranteed by tests to always return Some(_)",
        );

        SITE_URL
            .join(format!("{media_url_prefix}/{}", self.id).as_str())
            .expect("url guaranteed to be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    fn init_movie_cast() -> MovieCast {
        MovieCast {
            id: 273481,
            title: "Sicario".to_string(),
            original_title: "Sicario".to_string(),
            character: "Ted".to_string(),
            genres: vec![MovieGenre::Action, MovieGenre::Crime, MovieGenre::Thriller],
            release_date: NaiveDate::parse_from_str("2015-09-17", "%Y-%m-%d").ok(),
            overview: "An idealistic FBI agent is enlisted by a government task force to aid in the escalating war against drugs at the border area between the U.S. and Mexico.".to_string(),
            original_language: "en".to_string(),
            credit_id: "example-credit-id".to_string(),
        }
    }

    fn init_tv_cast() -> TvCast {
        TvCast {
            id: 67178,
            name: "Marvel's The Punisher".to_string(),
            original_name: "Marvel's The Punisher".to_string(),
            character: "Frank Castle / Punisher".to_string(),
            genres: vec![TvGenre::ActionAndAdventure, TvGenre::Crime, TvGenre::Drama],
            first_air_date: NaiveDate::parse_from_str("2017-11-17", "%Y-%m-%d").ok(),
            overview: "A former Marine out to punish the criminals responsible for his family's murder finds himself ensnared in a military conspiracy.".to_string(),
            original_language: "en".to_string(),
            credit_id: "example-credit-id".to_string(),
        }
    }

    #[test]
    fn test_movie_cast_traits() {
        assert_impl_all!(MovieCast: IsCredit, MediaTypeDefinition, MediaPageUrl<MovieCast>);
    }

    #[test]
    fn test_tv_cast_traits() {
        assert_impl_all!(TvCast: IsCredit, MediaTypeDefinition, MediaPageUrl<TvCast>);
    }

    #[test]
    fn test_movie_cast_media_type() {
        assert_eq!(MovieCast::MEDIA_TYPE, MediaType::Movie);
    }

    #[test]
    fn test_tv_cast_media_type() {
        assert_eq!(TvCast::MEDIA_TYPE, MediaType::Tv);
    }

    #[test]
    fn test_movie_cast_tmdb_media_url() {
        let cast = init_movie_cast();
        assert_eq!(
            cast.tmdb_media_url().as_str(),
            "https://www.themoviedb.org/movie/273481"
        );
    }

    #[test]
    fn test_tv_cast_tmdb_media_url() {
        let cast = init_tv_cast();
        assert_eq!(
            cast.tmdb_media_url().as_str(),
            "https://www.themoviedb.org/tv/67178"
        );
    }
}
