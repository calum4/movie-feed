use crate::models::v3::credit::{CreditType, IsCredit};
use crate::models::v3::genre_id::GenreId;
use crate::models::v3::genres::{Genre, MovieGenre, TvGenre};
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};
use serde_utils::deserialize_potentially_empty_string;
use tmdb_macros::IsCredit;

#[derive(Debug, Deserialize, Hash, IsCredit, Clone)]
#[serde(tag = "media_type")]
pub enum Cast {
    #[serde(rename = "movie")]
    Movie(MovieCast),
    #[serde(rename = "tv")]
    Tv(TvCast),
}

impl Cast {
    pub fn character(&self) -> Option<&String> {
        match self {
            Cast::Movie(credit) => credit.character.as_ref(),
            Cast::Tv(credit) => credit.character.as_ref(),
        }
    }
}

#[derive(Debug, Deserialize, Hash, Clone)]
pub struct MovieCast {
    pub id: usize,
    pub title: String,
    pub original_title: String,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub character: Option<String>,
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
pub struct TvCast {
    pub id: usize,
    pub name: String,
    pub original_name: String,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub character: Option<String>,
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
    let date = match deserialize_potentially_empty_string(deserializer)? {
        None => return Ok(None),
        Some(date) => date,
    };

    Ok(NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").ok())
}

impl MovieCast {
    const MEDIA_TYPE: MediaType = MediaType::Movie;
    const CREDIT_TYPE: CreditType = CreditType::Cast;
}

impl IsCredit for MovieCast {
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

    #[inline]
    fn credit_type(&self) -> CreditType {
        Self::CREDIT_TYPE
    }
}

impl TvCast {
    const MEDIA_TYPE: MediaType = MediaType::Tv;
    const CREDIT_TYPE: CreditType = CreditType::Cast;
}

impl IsCredit for TvCast {
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

    #[inline]
    fn credit_type(&self) -> CreditType {
        Self::CREDIT_TYPE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_movie_cast() -> MovieCast {
        MovieCast {
            id: 273481,
            title: "Sicario".to_string(),
            original_title: "Sicario".to_string(),
            character: Some("Ted".to_string()),
            genres: vec![MovieGenre::Action, MovieGenre::Crime, MovieGenre::Thriller],
            release_date: NaiveDate::parse_from_str("2015-09-17", "%Y-%m-%d").ok(),
            overview: Some(
                include_str!(
                    "../../../tests/assets/api/person/combined_credits/273481_overview.txt"
                )
                .to_string(),
            ),
            original_language: "en".to_string(),
            credit_id: "example-credit-id".to_string(),
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
            overview: Some(
                include_str!(
                    "../../../tests/assets/api/person/combined_credits/273481_overview.txt"
                )
                .to_string(),
            ),
            original_language: "en".to_string(),
            credit_id: "example-credit-id".to_string(),
        }
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

    #[test]
    fn test_credit_type() {
        assert_eq!(MovieCast::CREDIT_TYPE, CreditType::Cast);
        assert_eq!(TvCast::CREDIT_TYPE, CreditType::Cast);
    }
}
