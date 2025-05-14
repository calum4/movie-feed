use crate::models::genre_id::GenreId;
use crate::models::genres::{MovieGenre, TvGenre};
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize)]
#[serde(tag = "media_type")]
pub enum Cast {
    #[serde(rename = "movie")]
    Movie(MovieCast),
    #[serde(rename = "tv")]
    Tv(TvCast),
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize)]
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
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize)]
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
