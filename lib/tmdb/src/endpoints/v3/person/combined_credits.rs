use crate::Tmdb;
use crate::endpoints::{RequestError, request};
use crate::models::v3::cast::Cast;
use crate::models::v3::crew::Crew;
use crate::models::v3::tmdb_error::TmdbError;
#[cfg(all(feature = "cached", not(test)))]
use cached::proc_macro::cached;
use http::StatusCode;
use reqwest::Method;
use serde::Deserialize;

#[cfg_attr(feature = "serde_serialize", derive(serde::Serialize))]
#[derive(Debug, Deserialize, Clone)]
pub struct CombinedCredits {
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub cast: Vec<Cast>,
    #[serde(default)]
    pub crew: Vec<Crew>,
}

/// [GET: Combined Credits](https://developer.themoviedb.org/v3/reference/person-combined-credits)
///
/// Performs a get request on the `person/{person_id}/combined_credits` endpoint.
///
/// ## NOTE
/// The CombinedCredits struct is not an exhaustive representation of the data provided by
/// the api.
#[cfg_attr(all(feature = "cached", not(test)), cached(
    time = 3600, // 1 hour
    time_refresh = false,
    sync_writes = "by_key",
    key = "String",
    convert = r##"{ person_id.to_string() }"##,
    result = true
))]
pub async fn get(tmdb: &Tmdb, person_id: &str) -> Result<CombinedCredits, RequestError> {
    let path = format!("person/{person_id}/combined_credits");

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
        .json::<CombinedCredits>()
        .await
        .map_err(RequestError::Reqwest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::v3::genres::{MovieGenre, TvGenre};
    use chrono::NaiveDate;
    use tmdb_test_utils::api::v3::person::combined_credits::mock_get_person_combined_credits;
    use tmdb_test_utils::mockito::{Mock, ServerGuard};
    use tmdb_test_utils::start_mock_tmdb_api;

    async fn init(person_id: &str) -> (Tmdb, ServerGuard, Mock) {
        let mut server = start_mock_tmdb_api().await;
        let mock = mock_get_person_combined_credits(&mut server, person_id).await;

        let mut tmdb = Tmdb::default();
        tmdb.override_api_url(server.url().as_str()).unwrap();

        (tmdb, server, mock)
    }

    #[tokio::test]
    async fn test_get_19498_cast() {
        const PERSON_ID: &str = "19498";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(19498));

        let cast = response.cast;
        assert_eq!(cast.len(), 79);

        let movie = match &cast[0] {
            Cast::Movie(cast) => cast,
            Cast::Tv(_) => {
                panic!("first cast entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 1852);
        assert_eq!(movie.title, "World Trade Center");
        assert_eq!(movie.original_title, "World Trade Center");
        assert_eq!(movie.character.as_ref().unwrap(), "Christopher Amoroso");
        assert_eq!(
            movie.genres,
            [MovieGenre::Drama, MovieGenre::History, MovieGenre::Thriller]
        );
        assert_eq!(
            movie.release_date,
            Some(NaiveDate::parse_from_str("2006-08-09", "%Y-%m-%d").unwrap())
        );
        assert_eq!(
            movie.overview,
            Some(
                include_str!(
                    "../../../../tests/assets/api/person/combined_credits/1852_overview.txt"
                )
                .to_string()
            )
        );
        assert_eq!(movie.original_language, "en");
        assert_eq!(movie.credit_id, "52fe431bc3a36847f803a9db");

        let tv = match &cast[50] {
            Cast::Tv(cast) => cast,
            Cast::Movie(_) => {
                panic!("48th cast entry should be a tv show, was a movie");
            }
        };

        assert_eq!(tv.id, 1100);
        assert_eq!(tv.name, "How I Met Your Mother");
        assert_eq!(tv.original_name, "How I Met Your Mother");
        assert_eq!(tv.character.as_ref().unwrap(), "Carlos");
        assert_eq!(tv.genres, [TvGenre::Comedy]);
        assert_eq!(
            tv.first_air_date,
            Some(NaiveDate::parse_from_str("2005-09-19", "%Y-%m-%d").unwrap())
        );
        assert_eq!(
            tv.overview,
            Some(
                include_str!(
                    "../../../../tests/assets/api/person/combined_credits/1100_overview.txt"
                )
                .to_string()
            )
        );
        assert_eq!(tv.original_language, "en");
        assert_eq!(tv.credit_id, "5256c6e119c2956ff602e49c");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_19498_no_cast_credits() {
        const PERSON_ID: &str = "19498-no-cast-credits";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(19498));

        assert_eq!(response.cast.len(), 0);
        assert_eq!(response.crew.len(), 3);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_956_crew() {
        const PERSON_ID: &str = "956";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(956));

        let crew = response.crew;
        assert_eq!(crew.len(), 66);

        let movie = match &crew[7] {
            Crew::Movie(crew) => crew,
            Crew::Tv(_) => {
                panic!("7th crew entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 10528);
        assert_eq!(movie.title, "Sherlock Holmes");
        assert_eq!(movie.original_title, "Sherlock Holmes");
        assert_eq!(movie.department, "Directing");
        assert_eq!(movie.job, "Director");
        assert_eq!(
            movie.genres,
            [
                MovieGenre::Action,
                MovieGenre::Adventure,
                MovieGenre::Crime,
                MovieGenre::Mystery
            ]
        );
        assert_eq!(
            movie.release_date,
            Some(NaiveDate::parse_from_str("2009-12-23", "%Y-%m-%d").unwrap())
        );
        assert_eq!(
            movie.overview,
            Some(
                include_str!(
                    "../../../../tests/assets/api/person/combined_credits/10528_overview.txt"
                )
                .to_string()
            )
        );
        assert_eq!(movie.original_language, "en");
        assert_eq!(movie.credit_id, "52fe43809251416c75012e71");

        let tv = match &crew[59] {
            Crew::Tv(crew) => crew,
            Crew::Movie(_) => {
                panic!("59th crew entry should be a tv show, was a movie");
            }
        };

        assert_eq!(tv.id, 236235);
        assert_eq!(tv.name, "The Gentlemen");
        assert_eq!(tv.original_name, "The Gentlemen");
        assert_eq!(movie.department, "Directing");
        assert_eq!(movie.job, "Director");
        assert_eq!(tv.genres, [TvGenre::Comedy, TvGenre::Drama, TvGenre::Crime]);
        assert_eq!(
            tv.first_air_date,
            Some(NaiveDate::parse_from_str("2024-03-07", "%Y-%m-%d").unwrap())
        );
        assert_eq!(
            tv.overview,
            Some(
                include_str!(
                    "../../../../tests/assets/api/person/combined_credits/236235_overview.txt"
                )
                .to_string()
            )
        );
        assert_eq!(tv.original_language, "en");
        assert_eq!(tv.credit_id, "65df4747b76cbb017dd8ff39");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_956_no_crew_credits() {
        const PERSON_ID: &str = "956-no-crew-credits";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(956));

        assert_eq!(response.cast.len(), 13);
        assert_eq!(response.crew.len(), 0);

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_5_cast() {
        const PERSON_ID: &str = "5";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(5));

        let cast = response.cast;
        assert_eq!(cast.len(), 139);

        let movie = match &cast[12] {
            Cast::Movie(cast) => cast,
            Cast::Tv(_) => {
                panic!("12th cast entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 11868);
        assert_eq!(movie.title, "Dracula");
        assert_eq!(movie.original_title, "Dracula");
        assert_eq!(movie.character, Some("Doctor Van Helsing".to_string()));
        assert_eq!(movie.genres, [MovieGenre::Horror]);
        assert_eq!(
            movie.release_date,
            Some(NaiveDate::parse_from_str("1958-04-21", "%Y-%m-%d").unwrap())
        );
        assert_eq!(
            movie.overview,
            Some(
                include_str!(
                    "../../../../tests/assets/api/person/combined_credits/11868_overview.txt"
                )
                .to_string()
            )
        );
        assert_eq!(movie.original_language, "en");
        assert_eq!(movie.credit_id, "52fe44969251416c7503a173");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_48000_cast() {
        const PERSON_ID: &str = "48000";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(48000));

        let cast = response.cast;
        assert_eq!(cast.len(), 19);

        let movie = match &cast[0] {
            Cast::Movie(cast) => cast,
            Cast::Tv(_) => {
                panic!("1st cast entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 6117);
        assert_eq!(movie.title, "Love, Dance, and 1000 Songs");
        assert_eq!(movie.original_title, "Liebe, Tanz und 1000 Schlager");
        assert_eq!(movie.character, Some("Orchesterleiter".to_string()));
        assert_eq!(
            movie.genres,
            [MovieGenre::Romance, MovieGenre::Comedy, MovieGenre::Music]
        );
        assert_eq!(
            movie.release_date,
            Some(NaiveDate::parse_from_str("1955-10-13", "%Y-%m-%d").unwrap())
        );
        assert_eq!(movie.overview, None);
        assert_eq!(movie.original_language, "de");
        assert_eq!(movie.credit_id, "52fe443fc3a36847f808abff");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_48000_crew() {
        const PERSON_ID: &str = "48000";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(48000));

        let crew = response.crew;
        assert_eq!(crew.len(), 3);

        let movie = match &crew[1] {
            Crew::Movie(crew) => crew,
            Crew::Tv(_) => {
                panic!("2nd crew entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 6525);
        assert_eq!(movie.title, "Kriminaltango");
        assert_eq!(movie.original_title, "Kriminaltango");
        assert_eq!(movie.department, "Sound");
        assert_eq!(movie.job, "Original Music Composer");
        assert_eq!(movie.genres, [MovieGenre::Comedy]);
        assert_eq!(
            movie.release_date,
            Some(NaiveDate::parse_from_str("1960-07-30", "%Y-%m-%d").unwrap())
        );
        assert_eq!(movie.overview, None);
        assert_eq!(movie.original_language, "de");
        assert_eq!(movie.credit_id, "52fe4458c3a36847f8090951");

        mock.assert();
    }

    #[tokio::test]
    async fn test_get_240990() {
        const PERSON_ID: &str = "240990";
        let (tmdb, _server, mock) = init(PERSON_ID).await;

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id, Some(240990));

        let cast = response.cast;
        assert_eq!(cast.len(), 1);
        assert_eq!(response.crew.len(), 0);

        let movie = match &cast[0] {
            Cast::Movie(cast) => cast,
            Cast::Tv(_) => {
                panic!("1st cast entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 65369);
        assert_eq!(movie.title, "Do-Nut");
        assert_eq!(movie.original_title, "โด๋-นัท");
        assert_eq!(movie.character, Some("Jane".to_string()));
        assert_eq!(movie.genres, [MovieGenre::Romance, MovieGenre::Comedy]);
        assert_eq!(
            movie.release_date,
            Some(NaiveDate::parse_from_str("2011-05-26", "%Y-%m-%d").unwrap())
        );
        assert_eq!(
            movie.overview,
            Some(
                include_str!(
                    "../../../../tests/assets/api/person/combined_credits/65369_overview.txt"
                )
                .to_string()
            )
        );
        assert_eq!(movie.original_language, "th");
        assert_eq!(movie.credit_id, "52fe4707c3a368484e0b1447");

        mock.assert();
    }
}
