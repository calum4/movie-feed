use chrono::NaiveDate;
use serde::Deserialize;
use crate::models::genres::{MovieGenre, TvGenre};
use crate::models::cast::{deserialize_movie_genre, deserialize_tv_genre, deserialize_release_date};

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize)]
#[serde(tag = "media_type")]
pub enum Crew {
    #[serde(rename = "movie")]
    Movie(MovieCrew),

    #[serde(rename = "tv")]
    Tv(TvCrew),
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize)]
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
}

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize)]
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
}
