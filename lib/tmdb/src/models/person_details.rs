use chrono::NaiveDate;
use serde::Deserialize;
use crate::models::gender::Gender;

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize)]
pub struct PersonDetails {
    #[serde(default = "serde_utils::bool_true")]
    pub adult: bool,
    #[serde(default = "serde_utils::vec_zero_size")]
    pub also_known_as: Vec<String>,
    pub biography: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub deathday: Option<NaiveDate>,
    #[serde(default)]
    pub gender: Gender,
    pub homepage: Option<String>,
    #[serde(default)]
    pub id: i32,
    pub imdb_id: Option<String>,
    pub known_for_department: String,
    pub name: String,
    pub place_of_birth: Option<String>,
    #[serde(default)]
    pub popularity: f32,
    pub profile_path: Option<String>,
}
