use crate::SITE_URL;
use crate::models::v3::cast::{
    IsCredit, MediaPageUrl, MediaTypeDefinition, deserialize_release_date,
};
use crate::models::v3::genres::{MovieGenre, TvGenre};
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

use crate::models::v3::genre_id::GenreId;
use serde::de::Error as DeError;
use url::Url;

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Hash)]
pub enum Crew {
    Movie(MovieCrew),
    Tv(TvCrew),
}

impl<'de> Deserialize<'de> for Crew {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct JoinedCrew {
            // Common
            pub id: usize,
            pub department: String,
            pub job: String,
            pub genre_ids: Vec<GenreId>,
            pub overview: String,
            pub original_language: String,
            pub credit_id: Option<String>,
            pub media_type: Option<MediaType>,
            // Movie
            pub title: Option<String>,
            pub original_title: Option<String>,
            #[serde(deserialize_with = "deserialize_release_date", default)]
            pub release_date: Option<NaiveDate>,
            // Tv
            pub name: Option<String>,
            pub original_name: Option<String>,
            #[serde(deserialize_with = "deserialize_release_date", default)]
            pub first_air_date: Option<NaiveDate>,
        }

        let data = JoinedCrew::deserialize(deserializer)?;

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

                MovieCrew {
                    id: data.id,
                    title,
                    original_title,
                    department: data.department,
                    job: data.job,
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

                TvCrew {
                    id: data.id,
                    name,
                    original_name,
                    department: data.department,
                    job: data.job,
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
pub struct MovieCrew {
    pub id: usize,
    pub title: String,
    pub original_title: String,
    pub department: String,
    pub job: String,
    pub genres: Vec<MovieGenre>,
    pub release_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
    pub credit_id: Option<String>,
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Hash)]
pub struct TvCrew {
    pub id: usize,
    pub name: String,
    pub original_name: String,
    pub department: String,
    pub job: String,
    pub genres: Vec<TvGenre>,
    pub first_air_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
    pub credit_id: Option<String>,
}

impl IsCredit for MovieCrew {}
impl IsCredit for TvCrew {}

impl MediaTypeDefinition for MovieCrew {
    const MEDIA_TYPE: MediaType = MediaType::Movie;
}

impl MediaTypeDefinition for TvCrew {
    const MEDIA_TYPE: MediaType = MediaType::Tv;
}

impl MediaPageUrl for MovieCrew {
    fn tmdb_media_url(&self) -> Url {
        let media_url_prefix = Self::MEDIA_TYPE.tmdb_url_prefix().expect(
            "Self::MEDIA_TYPE is const and is guaranteed by tests to always return Some(_)",
        );

        SITE_URL
            .join(format!("{media_url_prefix}/{}", self.id).as_str())
            .expect("url guaranteed to be valid")
    }
}

impl MediaPageUrl for TvCrew {
    fn tmdb_media_url(&self) -> Url {
        let media_url_prefix = Self::MEDIA_TYPE.tmdb_url_prefix().expect(
            "Self::MEDIA_TYPE is const and is guaranteed by tests to always return Some(_)",
        );

        SITE_URL
            .join(format!("{media_url_prefix}/{}", self.id).as_str())
            .expect("url guaranteed to be valid")
    }
}

impl From<MovieCrew> for Crew {
    fn from(value: MovieCrew) -> Self {
        Self::Movie(value)
    }
}

impl From<TvCrew> for Crew {
    fn from(value: TvCrew) -> Self {
        Self::Tv(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    fn init_movie_crew() -> MovieCrew {
        MovieCrew {
            id: 1290379,
            title: "Road House 2".to_string(),
            original_title: "Road House 2".to_string(),
            department: "Directing".to_string(),
            job: "Directing".to_string(),
            genres: vec![MovieGenre::Action],
            release_date: None,
            overview: "The sequel to the 2024 reboot.".to_string(),
            original_language: "en".to_string(),
            credit_id: Some("example-credit-id".to_string()),
        }
    }

    fn init_tv_crew() -> TvCrew {
        TvCrew {
            id: 236235,
            name: "The Gentlemen".to_string(),
            original_name: "The Gentlemen".to_string(),
            department: "Creator".to_string(),
            job: "Creator".to_string(),
            genres: vec![TvGenre::Comedy, TvGenre::Drama, TvGenre::Crime],
            first_air_date: NaiveDate::parse_from_str("2024-03-07", "%Y-%m-%d").ok(),
            overview: "When aristocratic Eddie inherits the family estate, he discovers that it's home to an enormous weed empire — and its proprietors aren't going anywhere.".to_string(),
            original_language: "en".to_string(),
            credit_id: Some("example-credit-id".to_string()),
        }
    }

    #[test]
    fn test_movie_crew_traits() {
        assert_impl_all!(MovieCrew: IsCredit, MediaTypeDefinition, MediaPageUrl<MovieCrew>);
    }

    #[test]
    fn test_tv_crew_traits() {
        assert_impl_all!(TvCrew: IsCredit, MediaTypeDefinition, MediaPageUrl<TvCrew>);
    }

    #[test]
    fn test_movie_crew_media_type() {
        assert_eq!(MovieCrew::MEDIA_TYPE, MediaType::Movie);
    }

    #[test]
    fn test_tv_crew_media_type() {
        assert_eq!(TvCrew::MEDIA_TYPE, MediaType::Tv);
    }

    #[test]
    fn test_movie_crew_tmdb_media_url() {
        let cast = init_movie_crew();
        assert_eq!(
            cast.tmdb_media_url().as_str(),
            "https://www.themoviedb.org/movie/1290379"
        );
    }

    #[test]
    fn test_tv_crew_tmdb_media_url() {
        let cast = init_tv_crew();
        assert_eq!(
            cast.tmdb_media_url().as_str(),
            "https://www.themoviedb.org/tv/236235"
        );
    }
}
