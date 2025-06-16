use crate::models::v3::cast::{
    deserialize_movie_genre, deserialize_release_date, deserialize_tv_genre,
};
use crate::models::v3::credit::IsCredit;
use crate::models::v3::genres::{Genre, MovieGenre, TvGenre};
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_utils::deserialize_potentially_empty_string;
use tmdb_macros::IsCredit;

#[derive(Debug, Deserialize, Hash, IsCredit, Clone)]
#[serde(tag = "media_type")]
pub enum Crew {
    #[serde(rename = "movie")]
    Movie(MovieCrew),

    #[serde(rename = "tv")]
    Tv(TvCrew),
}

impl Crew {
    pub fn department(&self) -> &str {
        match self {
            Crew::Movie(credit) => credit.department.as_str(),
            Crew::Tv(credit) => credit.department.as_str(),
        }
    }

    pub fn job(&self) -> &str {
        match self {
            Crew::Movie(credit) => credit.job.as_str(),
            Crew::Tv(credit) => credit.job.as_str(),
        }
    }
}

#[derive(Debug, Deserialize, Hash, Clone)]
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

#[derive(Debug, Deserialize, Hash, Clone)]
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

impl MovieCrew {
    const MEDIA_TYPE: MediaType = MediaType::Movie;
}

impl IsCredit for MovieCrew {
    #[inline]
    fn id(&self) -> usize {
        self.id
    }

    #[inline]
    fn title(&self) -> &str {
        self.title.as_str()
    }

    #[inline]
    fn original_title(&self) -> &str {
        self.original_title.as_str()
    }

    #[inline]
    fn genres(&self) -> Vec<&dyn Genre> {
        self.genres
            .iter()
            .map(|genre| genre as &dyn Genre)
            .collect()
    }

    #[inline]
    fn release_date(&self) -> Option<&NaiveDate> {
        self.release_date.as_ref()
    }

    #[inline]
    fn original_language(&self) -> &str {
        self.original_language.as_str()
    }

    #[inline]
    fn overview(&self) -> Option<&String> {
        self.overview.as_ref()
    }

    #[inline]
    fn credit_id(&self) -> &str {
        self.credit_id.as_str()
    }

    #[inline]
    fn media_type(&self) -> MediaType {
        Self::MEDIA_TYPE
    }
}

impl TvCrew {
    const MEDIA_TYPE: MediaType = MediaType::Tv;
}

impl IsCredit for TvCrew {
    #[inline]
    fn id(&self) -> usize {
        self.id
    }

    #[inline]
    fn title(&self) -> &str {
        self.name.as_str()
    }

    #[inline]
    fn original_title(&self) -> &str {
        self.original_name.as_str()
    }

    #[inline]
    fn genres(&self) -> Vec<&dyn Genre> {
        self.genres
            .iter()
            .map(|genre| genre as &dyn Genre)
            .collect()
    }

    #[inline]
    fn release_date(&self) -> Option<&NaiveDate> {
        self.first_air_date.as_ref()
    }

    #[inline]
    fn original_language(&self) -> &str {
        self.original_language.as_str()
    }

    #[inline]
    fn overview(&self) -> Option<&String> {
        self.overview.as_ref()
    }

    #[inline]
    fn credit_id(&self) -> &str {
        self.credit_id.as_str()
    }

    #[inline]
    fn media_type(&self) -> MediaType {
        Self::MEDIA_TYPE
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
            overview: Some(
                include_str!(
                    "../../../tests/assets/api/person/combined_credits/1290379_overview.txt"
                )
                .to_string(),
            ),
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
            overview: Some(
                include_str!(
                    "../../../tests/assets/api/person/combined_credits/236235_overview.txt"
                )
                .to_string(),
            ),
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
