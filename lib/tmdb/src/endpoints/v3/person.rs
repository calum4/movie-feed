use crate::Tmdb;
use crate::endpoints::{RequestError, request};
use crate::models::v3::person_details::PersonDetails;
use crate::models::v3::tmdb_error::TmdbError;
#[cfg(all(feature = "cached", not(test)))]
use cached::proc_macro::cached;
use http::StatusCode;
use reqwest::Method;
use tracing::{instrument, trace};

pub mod combined_credits;

/// [GET: Person Details](https://developer.themoviedb.org/reference/person-details)
///
/// Performs a get request on the `person/{person_id}` endpoint.
#[cfg_attr(all(feature = "cached", not(test)), cached(
    time = 3600, // 1 hour
    time_refresh = false,
    sync_writes = "by_key",
    key = "i32",
    convert = r##"{ person_id }"##,
    result = true
))]
#[instrument(level = "trace", name = "person::get", skip(tmdb))]
pub async fn get(tmdb: &Tmdb, person_id: i32) -> Result<PersonDetails, RequestError> {
    #[cfg(feature = "cached")]
    trace!("cache miss");

    let path = format!("person/{person_id}");

    let response = request(tmdb, path, Method::GET).await?;

    match response.status() {
        StatusCode::OK => (),
        _ => {
            return Err(match TmdbError::try_from_response(response).await {
                Ok(error) => error.into(),
                Err(error) => error.into(),
            });
        }
    }

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
        assert_eq!(response.popularity, 11.5636);
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
        assert_eq!(response.popularity, 6.5548);
        assert_eq!(
            response.profile_path,
            Some("/9pLUnjMgIEWXi0mlHYzie9cKUTD.jpg".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_5() {
        const PERSON_ID: i32 = 5;
        const BIOGRAPHY: &str = include_str!("../../../tests/assets/api/person/5_biography.txt");

        let (tmdb, _server, mock) = init(PERSON_ID).await;
        let response = get(&tmdb, PERSON_ID).await.unwrap();

        assert!(!response.adult);
        assert_eq!(
            response.also_known_as,
            vec![
                "Peter Wilton Cushing",
                "彼得·庫辛",
                "Питер Кушинг",
                "Питер Уилтон Кушинг",
                "پیتر کوشینگ"
            ]
        );
        assert_eq!(response.biography, Some(BIOGRAPHY.to_string()));
        assert_eq!(
            response.birthday,
            NaiveDate::parse_from_str("1913-05-26", "%Y-%m-%d").ok()
        );
        assert_eq!(
            response.deathday,
            NaiveDate::parse_from_str("1994-08-11", "%Y-%m-%d").ok()
        );
        assert_eq!(response.gender, Gender::Male);
        assert_eq!(response.homepage, None);
        assert_eq!(response.id, PERSON_ID);
        assert_eq!(response.imdb_id, Some("nm0001088".to_string()));
        assert_eq!(response.known_for_department, "Acting");
        assert_eq!(response.name, "Peter Cushing");
        assert_eq!(
            response.place_of_birth,
            Some("Kenley, Surrey, England, UK".to_string())
        );
        assert_eq!(response.popularity, 4.34);
        assert_eq!(
            response.profile_path,
            Some("/if5g03wn6uvHx7F6FxXHLebKc0q.jpg".to_string())
        );

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_48000() {
        const PERSON_ID: i32 = 48000;

        let (tmdb, _server, mock) = init(PERSON_ID).await;
        let response = get(&tmdb, PERSON_ID).await.unwrap();

        assert!(!response.adult);
        assert!(response.also_known_as.is_empty());
        assert_eq!(response.biography, None);
        assert_eq!(
            response.birthday,
            NaiveDate::parse_from_str("1922-02-18", "%Y-%m-%d").ok()
        );
        assert_eq!(
            response.deathday,
            NaiveDate::parse_from_str("2012-02-26", "%Y-%m-%d").ok()
        );
        assert_eq!(response.gender, Gender::Male);
        assert_eq!(response.homepage, None);
        assert_eq!(response.id, PERSON_ID);
        assert_eq!(response.imdb_id, Some("nm0652411".to_string()));
        assert_eq!(response.known_for_department, "Acting");
        assert_eq!(response.name, "Hazy Osterwald");
        assert_eq!(
            response.place_of_birth,
            Some("Bern, Switzerland".to_string())
        );
        assert_eq!(response.popularity, 0.1471);
        assert_eq!(response.profile_path, None);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_240990() {
        const PERSON_ID: i32 = 240990;

        let (tmdb, _server, mock) = init(PERSON_ID).await;
        let response = get(&tmdb, PERSON_ID).await.unwrap();

        assert!(!response.adult);
        assert!(response.also_known_as.is_empty());
        assert_eq!(response.biography, None);
        assert_eq!(response.birthday, None);
        assert_eq!(response.deathday, None);
        assert_eq!(response.gender, Gender::NotSpecified);
        assert_eq!(response.homepage, None);
        assert_eq!(response.id, PERSON_ID);
        assert_eq!(response.imdb_id, None);
        assert_eq!(response.known_for_department, "Acting");
        assert_eq!(response.name, "Phulada Luechatham");
        assert_eq!(response.place_of_birth, None);
        assert_eq!(response.popularity, 0.0552);
        assert_eq!(response.profile_path, None);

        mock.assert();
    }
}
