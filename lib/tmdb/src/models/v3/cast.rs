use crate::SITE_URL;
use crate::models::v3::genre_id::GenreId;
use crate::models::v3::genres::{MovieGenre, TvGenre};
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use serde_utils::deserialize_potentially_empty_string;
use url::Url;

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Hash)]
pub enum Cast {
    Movie(MovieCast),
    Tv(TvCast),
}

impl<'de> Deserialize<'de> for Cast {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct JoinedCast {
            // Common
            pub id: usize,
            #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
            pub character: Option<String>,
            pub genre_ids: Vec<GenreId>,
            pub overview: String,
            pub original_language: String,
            #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
            pub credit_id: Option<String>,
            pub media_type: Option<MediaType>,
            // Movie
            #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
            pub title: Option<String>,
            #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
            pub original_title: Option<String>,
            #[serde(deserialize_with = "deserialize_release_date", default)]
            pub release_date: Option<NaiveDate>,
            // Tv
            #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
            pub name: Option<String>,
            #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
            pub original_name: Option<String>,
            #[serde(deserialize_with = "deserialize_release_date", default)]
            pub first_air_date: Option<NaiveDate>,
        }

        let data = JoinedCast::deserialize(deserializer)?;

        let media_type = data
            .media_type
            .or_else(|| {
                if data.title.is_some() || data.original_title.is_some() {
                    Some(MediaType::Movie)
                } else if data.name.is_some() || data.original_name.is_some() {
                    Some(MediaType::Tv)
                } else {
                    None
                }
            })
            .ok_or_else(|| DeError::custom("unable to discern the media type"))?;

        Ok(match media_type {
            MediaType::Movie => {
                let (title, original_title) = match (data.title, data.original_title) {
                    (Some(title), Some(original_title)) => (title, original_title),
                    (Some(title), None) => (title.clone(), title),
                    (None, Some(original_title)) => (original_title.clone(), original_title),
                    (None, None) => return Err(DeError::missing_field("title or original_title")),
                };

                MovieCast {
                    id: data.id,
                    title,
                    original_title,
                    character: data.character,
                    genres: data.genre_ids.into_iter().map(MovieGenre::from).collect(),
                    release_date: data.release_date,
                    overview: data.overview,
                    original_language: data.original_language,
                    credit_id: data.credit_id,
                }
                .into()
            }
            MediaType::Tv => {
                let (name, original_name) = match (data.name, data.original_name) {
                    (Some(name), Some(original_name)) => (name, original_name),
                    (Some(name), None) => (name.clone(), name),
                    (None, Some(original_name)) => (original_name.clone(), original_name),
                    (None, None) => return Err(DeError::missing_field("name or original_name")),
                };

                TvCast {
                    id: data.id,
                    name,
                    original_name,
                    character: data.character,
                    genres: data.genre_ids.into_iter().map(TvGenre::from).collect(),
                    first_air_date: data.first_air_date,
                    overview: data.overview,
                    original_language: data.original_language,
                    credit_id: data.credit_id,
                }
                .into()
            }
            _ => unreachable!(),
        })
    }
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Hash)]
pub struct MovieCast {
    pub id: usize,
    pub title: String,
    pub original_title: String,
    pub character: Option<String>,
    pub genres: Vec<MovieGenre>,
    pub release_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
    pub credit_id: Option<String>,
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Hash)]
pub struct TvCast {
    pub id: usize,
    pub name: String,
    pub original_name: String,
    pub character: Option<String>,
    pub genres: Vec<TvGenre>,
    pub first_air_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
    pub credit_id: Option<String>,
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

impl From<MovieCast> for Cast {
    fn from(value: MovieCast) -> Self {
        Self::Movie(value)
    }
}

impl From<TvCast> for Cast {
    fn from(value: TvCast) -> Self {
        Self::Tv(value)
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
            character: Some("Ted".to_string()),
            genres: vec![MovieGenre::Action, MovieGenre::Crime, MovieGenre::Thriller],
            release_date: NaiveDate::parse_from_str("2015-09-17", "%Y-%m-%d").ok(),
            overview: "An idealistic FBI agent is enlisted by a government task force to aid in the escalating war against drugs at the border area between the U.S. and Mexico.".to_string(),
            original_language: "en".to_string(),
            credit_id: Some("example-credit-id".to_string()),
        }
    }

    fn init_tv_cast() -> TvCast {
        TvCast {
            id: 67178,
            name: "Marvel's The Punisher".to_string(),
            original_name: "Marvel's The Punisher".to_string(),
            character: Some("Frank Castle / Punisher".to_string()),
            genres: vec![TvGenre::ActionAndAdventure, TvGenre::Crime, TvGenre::Drama],
            first_air_date: NaiveDate::parse_from_str("2017-11-17", "%Y-%m-%d").ok(),
            overview: "A former Marine out to punish the criminals responsible for his family's murder finds himself ensnared in a military conspiracy.".to_string(),
            original_language: "en".to_string(),
            credit_id: Some("example-credit-id".to_string()),
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
