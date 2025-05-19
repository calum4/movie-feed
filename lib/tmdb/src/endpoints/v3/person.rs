use crate::Tmdb;
use crate::endpoints::{RequestError, request};
use crate::models::v3::person_details::PersonDetails;
use reqwest::Method;

pub mod combined_credits;

/// [GET: Person Details](https://developer.themoviedb.org/reference/person-details)
///
/// Performs a get request on the `person/{person_id}` endpoint.
pub async fn get(tmdb: &Tmdb, person_id: i32) -> Result<PersonDetails, RequestError> {
    let path = format!("person/{person_id}");

    let response: reqwest::Response = request(tmdb, path, Method::GET).await?;

    dbg!(response.status());
    // TODO - Investigate status

    response
        .json::<PersonDetails>()
        .await
        .map_err(RequestError::Reqwest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::v3::gender::Gender;
    use chrono::NaiveDate;
    use tmdb_test_utils::api::v3::person::mock_get_person_details;
    use tmdb_test_utils::mockito::{Mock, ServerGuard};
    use tmdb_test_utils::start_mock_tmdb_api;

    async fn init(person_id: i32) -> (Tmdb, ServerGuard, Mock) {
        let mut server = start_mock_tmdb_api().await;
        let mock = mock_get_person_details(&mut server, person_id).await;

        let mut tmdb = Tmdb::default();
        tmdb.override_api_url(server.url().as_str()).unwrap();

        (tmdb, server, mock)
    }

    #[tokio::test]
    async fn test_get_19498() {
        const PERSON_ID: i32 = 19498;
        const BIOGRAPHY: &str =
            include_str!("../../../tests/assets/api/person/19498_biography.txt");

        let (tmdb, _server, mock) = init(PERSON_ID).await;
        let response = get(&tmdb, PERSON_ID).await.unwrap();

        assert!(!response.adult);
        assert_eq!(
            response.also_known_as,
            vec![
                "乔·本恩瑟",
                "جان برانتال",
                "존 번탈",
                "Jonathan E. Bernthal",
                "جان برنتال",
            ]
        );
        assert_eq!(response.biography, Some(BIOGRAPHY.to_string()));
        assert_eq!(
            response.birthday,
            NaiveDate::parse_from_str("1976-09-20", "%Y-%m-%d").ok()
        );
        assert_eq!(response.deathday, None);
        assert_eq!(response.gender, Gender::Male);
        assert_eq!(response.homepage, None);
        assert_eq!(response.id, PERSON_ID);
        assert_eq!(response.imdb_id, Some("nm1256532".to_string()));
        assert_eq!(response.known_for_department, "Acting");
        assert_eq!(response.name, "Jon Bernthal");
        assert_eq!(
            response.place_of_birth,
            Some("Washington, D.C., USA".to_string())
        );
        assert_eq!(response.popularity, 10.3331);
        assert_eq!(
            response.profile_path,
            Some("/o0t6EVkJOrFAjESDilZUlf46IbQ.jpg".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_956() {
        const PERSON_ID: i32 = 956;
        const BIOGRAPHY: &str = include_str!("../../../tests/assets/api/person/956_biography.txt");

        let (tmdb, _server, mock) = init(PERSON_ID).await;
        let response = get(&tmdb, PERSON_ID).await.unwrap();

        assert!(!response.adult);
        assert_eq!(
            response.also_known_as,
            vec![
                "佳·烈治",
                "กาย ริตชี",
                "ガイ・リッチー",
                "가이 리치",
                "غاي ريتشي",
                "Γκάι Ρίτσι",
                "Guy Stuart Ritchie",
                "盖·里奇",
                "گای ریچی"
            ]
        );
        assert_eq!(response.biography, Some(BIOGRAPHY.to_string()));
        assert_eq!(
            response.birthday,
            NaiveDate::parse_from_str("1968-09-10", "%Y-%m-%d").ok()
        );
        assert_eq!(response.deathday, None);
        assert_eq!(response.gender, Gender::Male);
        assert_eq!(response.homepage, None);
        assert_eq!(response.id, PERSON_ID);
        assert_eq!(response.imdb_id, Some("nm0005363".to_string()));
        assert_eq!(response.known_for_department, "Directing");
        assert_eq!(response.name, "Guy Ritchie");
        assert_eq!(
            response.place_of_birth,
            Some("Hatfield, Hertfordshire, England, UK".to_string())
        );
        assert_eq!(response.popularity, 5.5305);
        assert_eq!(
            response.profile_path,
            Some("/9pLUnjMgIEWXi0mlHYzie9cKUTD.jpg".to_string())
        );

        mock.assert();
    }
}
