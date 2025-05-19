use crate::SITE_URL;
use crate::models::v3::cast::{
    IsCredit, MediaPageUrl, MediaTypeDefinition, deserialize_movie_genre, deserialize_release_date,
    deserialize_tv_genre,
};
use crate::models::v3::genres::{MovieGenre, TvGenre};
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use serde::Deserialize;
use url::Url;

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize, Hash)]
#[serde(tag = "media_type")]
pub enum Crew {
    #[serde(rename = "movie")]
    Movie(MovieCrew),

    #[serde(rename = "tv")]
    Tv(TvCrew),
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize, Hash)]
pub struct MovieCrew {
    pub id: usize,
    pub title: String,
    pub original_title: String,
    pub department: String,
    pub job: String,
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
pub struct TvCrew {
    pub id: usize,
    pub name: String,
    pub original_name: String,
    pub department: String,
    pub job: String,
    #[serde(deserialize_with = "deserialize_tv_genre", flatten)]
    pub genres: Vec<TvGenre>,
    #[serde(deserialize_with = "deserialize_release_date")]
    pub first_air_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
    pub credit_id: String,
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
            credit_id: "example-credit-id".to_string(),
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
            credit_id: "example-credit-id".to_string(),
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
