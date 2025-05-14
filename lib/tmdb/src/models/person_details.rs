use crate::SITE_URL;
use crate::models::gender::Gender;
use chrono::NaiveDate;
use serde::Deserialize;

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

impl PersonDetails {
    pub fn tmdb_url(&self) -> String {
        format!("{SITE_URL}/person/{}", self.id)
    }

    pub fn imdb_url(&self) -> Option<String> {
        self.imdb_id
            .as_ref()
            .map(|id| format!("https://www.imdb.com/name/{id}"))
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
            details.tmdb_url(),
            "https://www.themoviedb.org/person/19498".to_string()
        );
    }

    #[test]
    fn test_imdb_url() {
        let details = init();
        assert_eq!(
            details.imdb_url(),
            Some("https://www.imdb.com/name/nm1256532".to_string())
        );
    }
}
