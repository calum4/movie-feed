use crate::models::v3::cast::{
    deserialize_movie_genre, deserialize_release_date, deserialize_tv_genre,
};
use crate::models::v3::credit::IsCredit;
use crate::models::v3::genres::{Genre, MovieGenre, TvGenre};
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_utils::deserialize_potentially_empty_string;

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
    #[serde(
        deserialize_with = "deserialize_movie_genre",
        flatten,
        default = "serde_utils::vec_zero_size"
    )]
    pub genres: Vec<MovieGenre>,
    #[serde(deserialize_with = "deserialize_release_date", default)]
    pub release_date: Option<NaiveDate>,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub overview: Option<String>,
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
    #[serde(
        deserialize_with = "deserialize_tv_genre",
        flatten,
        default = "serde_utils::vec_zero_size"
    )]
    pub genres: Vec<TvGenre>,
    #[serde(deserialize_with = "deserialize_release_date", default)]
    pub first_air_date: Option<NaiveDate>,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub overview: Option<String>,
    pub original_language: String,
    pub credit_id: String,
}

impl IsCredit for MovieCrew {
    const MEDIA_TYPE: MediaType = MediaType::Movie;

    fn id(&self) -> usize {
        self.id
    }

    fn title(&self) -> &str {
        self.title.as_str()
    }

    fn original_title(&self) -> &str {
        self.original_language.as_str()
    }

    fn genres(&self) -> &[impl Genre] {
        &self.genres
    }

    fn release_date(&self) -> Option<&NaiveDate> {
        self.release_date.as_ref()
    }

    fn original_language(&self) -> &str {
        self.original_language.as_str()
    }

    fn overview(&self) -> Option<&String> {
        self.overview.as_ref()
    }

    fn credit_id(&self) -> &str {
        self.credit_id.as_str()
    }
}

impl IsCredit for TvCrew {
    const MEDIA_TYPE: MediaType = MediaType::Tv;

    fn id(&self) -> usize {
        self.id
    }

    fn title(&self) -> &str {
        self.name.as_str()
    }

    fn original_title(&self) -> &str {
        self.original_language.as_str()
    }

    fn genres(&self) -> &[impl Genre] {
        &self.genres
    }

    fn release_date(&self) -> Option<&NaiveDate> {
        self.first_air_date.as_ref()
    }

    fn original_language(&self) -> &str {
        self.original_language.as_str()
    }

    fn overview(&self) -> Option<&String> {
        self.overview.as_ref()
    }

    fn credit_id(&self) -> &str {
        self.credit_id.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_movie_crew() -> MovieCrew {
        MovieCrew {
            id: 1290379,
            title: "Road House 2".to_string(),
            original_title: "Road House 2".to_string(),
            department: "Directing".to_string(),
            job: "Directing".to_string(),
            genres: vec![MovieGenre::Action],
            release_date: None,
            overview: Some("The sequel to the 2024 reboot.".to_string()),
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
            overview: Some("When aristocratic Eddie inherits the family estate, he discovers that it's home to an enormous weed empire — and its proprietors aren't going anywhere.".to_string()),
            original_language: "en".to_string(),
            credit_id: "example-credit-id".to_string(),
        }
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
