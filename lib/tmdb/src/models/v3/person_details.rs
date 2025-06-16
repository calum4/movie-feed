use crate::models::v3::gender::Gender;
use crate::{IMDB_SITE_URL, SITE_URL};
use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};
use serde_utils::deserialize_potentially_empty_string;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct PersonDetails {
    #[serde(default = "serde_utils::bool_true")]
    pub adult: bool,
    #[serde(default = "serde_utils::vec_zero_size")]
    pub also_known_as: Vec<String>,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub biography: Option<String>,
    #[serde(deserialize_with = "deserialize_date", default)]
    pub birthday: Option<NaiveDate>,
    #[serde(deserialize_with = "deserialize_date", default)]
    pub deathday: Option<NaiveDate>,
    #[serde(default)]
    pub gender: Gender,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub id: i32,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub imdb_id: Option<String>,
    pub known_for_department: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub place_of_birth: Option<String>,
    #[serde(default)]
    pub popularity: f32,
    #[serde(deserialize_with = "deserialize_potentially_empty_string", default)]
    pub profile_path: Option<String>,
}

pub(super) fn deserialize_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let date = match deserialize_potentially_empty_string(deserializer)? {
        None => return Ok(None),
        Some(date) => date,
    };

    Ok(NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d").ok())
}

impl PersonDetails {
    pub fn tmdb_url(&self) -> Url {
        SITE_URL
            .join(format!("person/{}", self.id).as_str())
            .expect("url should always be valid")
    }

    pub fn imdb_url(&self) -> Option<Url> {
        self.imdb_id.as_ref().map(|id| {
            IMDB_SITE_URL
                .join(format!("/name/{id}").as_str())
                .expect("url should always be valid")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() -> PersonDetails {
        PersonDetails {
            adult: false,
            also_known_as: vec![
                "乔·本恩瑟".to_string(),
                "جان برانتال".to_string(),
                "존 번탈".to_string(),
                "Jonathan E. Bernthal".to_string(),
                "جان برنتال".to_string(),
            ],
            biography: Some("Jonathan Edward Bernthal is an American actor....".to_string()), // truncated
            birthday: NaiveDate::parse_from_str("1976-09-20", "%Y-%m-%d").ok(),
            deathday: None,
            gender: Gender::Male,
            homepage: None,
            id: 19498,
            imdb_id: Some("nm1256532".to_string()),
            known_for_department: "Acting".to_string(),
            name: "Jon Bernthal".to_string(),
            place_of_birth: Some("Washington, D.C., USA".to_string()),
            popularity: 10.3331,
            profile_path: Some("/o0t6EVkJOrFAjESDilZUlf46IbQ.jpg".to_string()),
        }
    }

    #[test]
    fn test_tmdb_url() {
        let details = init();
        assert_eq!(
            details.tmdb_url().as_str(),
            "https://www.themoviedb.org/person/19498"
        );
    }

    #[test]
    fn test_imdb_url() {
        let details = init();
        assert_eq!(
            details.imdb_url().unwrap().as_str(),
            "https://www.imdb.com/name/nm1256532"
        );
    }
}
