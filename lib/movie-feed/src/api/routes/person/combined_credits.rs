mod release_status;
mod sort_order;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

pub(super) const PATH: &str = "/{person_id}/combined_credits";

pub(super) fn router() -> Router {
    Router::new().route("/", get(get::combined_credits))
}

mod get {
    use super::*;
    use crate::api::ApiState;
    use crate::api::process_result::{ProcessedResponse, process_response};
    use crate::api::routes::person::combined_credits::sort_order::SortReleaseDates;
    use crate::api::rss::Rss;
    use ammonia::Builder;
    use axum::Extension;
    use axum::extract::{Path, Query};
    use axum::response::{IntoResponse, Response};
    use chrono::Utc;
    #[cfg(test)]
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
    use itertools::Itertools;
    use release_status::ReleaseStatus;
    use rss::{Category, ChannelBuilder, Guid, GuidBuilder, Item, ItemBuilder};
    use serde::{Deserialize, Deserializer};
    use std::collections::HashSet;
    use std::fmt::{Display, Formatter};
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::sync::Arc;
    use std::time::Duration;
    use thiserror::Error;
    use tmdb::endpoints::v3::person::combined_credits::get as get_combined_credits;
    use tmdb::endpoints::v3::person::get as get_person_details;
    use tmdb::models::v3::credit::{Credit, IsCredit};
    use tracing::warn;
    use utils::const_assert;

    const TTL: Duration = Duration::from_secs(60 * 60); // 60 minutes

    const DEFAULT_SIZE: usize = 20;
    const MAX_SIZE: usize = 50;
    const_assert!(
        DEFAULT_SIZE <= MAX_SIZE,
        "MAX_SIZE must not exceed DEFAULT_SIZE"
    );

    #[cfg(test)]
    const TEST_BUILD_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 5, 21).unwrap();
    #[cfg(test)]
    const TEST_BUILD_TIME: NaiveTime = NaiveTime::from_hms_opt(18, 20, 44).unwrap();

    /// A hash of the following fields of a Credit
    /// - ID
    /// - Title
    /// - Release Date
    /// - Media Type
    /// - Credit Type
    fn credit_guid(credit: &impl IsCredit) -> Guid {
        let mut hasher = DefaultHasher::default();

        credit.id().hash(&mut hasher);
        credit.title().hash(&mut hasher);
        credit.release_date().hash(&mut hasher);
        credit.media_type().hash(&mut hasher);
        credit.credit_type().hash(&mut hasher);

        GuidBuilder::default()
            .value(hasher.finish().to_string())
            .permalink(false)
            .build()
    }

    #[inline]
    fn sanitise_text<S: AsRef<str>>(text: S) -> String {
        let mut s = text.as_ref().replace('\n', "<br>");

        let mut tags = HashSet::new();
        tags.insert("br");
        tags.insert("p");

        s = Builder::new().tags(tags).clean(s.as_ref()).to_string();

        s
    }

    #[derive(Deserialize, Debug, Eq, PartialEq)]
    pub(super) struct QueryArgs {
        #[serde(default = "serde_utils::defaults::default_usize::<DEFAULT_SIZE>")]
        /// Number of credits to return
        size: usize,
        #[serde(flatten, deserialize_with = "deserialize_release_status")]
        /// The release status of the credits to return
        release_status: ReleaseStatus,
        #[serde(default)]
        sort_order: SortReleaseDates,
    }

    impl Default for QueryArgs {
        fn default() -> Self {
            Self {
                size: DEFAULT_SIZE,
                release_status: Default::default(),
                sort_order: Default::default(),
            }
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Error)]
    enum ReleaseStatusError {
        MaxAgeSmaller,
    }

    impl ReleaseStatusError {
        fn text(&self) -> &'static str {
            match self {
                ReleaseStatusError::MaxAgeSmaller => "max_age must be larger than min_age",
            }
        }
    }

    impl Display for ReleaseStatusError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.text())
        }
    }

    fn deserialize_release_status<'de, D>(deserializer: D) -> Result<ReleaseStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        let release = ReleaseStatus::deserialize(deserializer).unwrap_or_default();

        if let ReleaseStatus::Released { max_age, min_age } = &release
            && max_age < min_age
        {
            return Err(serde::de::Error::custom(ReleaseStatusError::MaxAgeSmaller));
        }

        Ok(release)
    }

    pub(super) async fn combined_credits(
        Path(person_id): Path<i32>,
        api_state: Extension<Arc<ApiState>>,
        query: Query<QueryArgs>,
    ) -> Response {
        let details = match process_response(get_person_details(&api_state.tmdb, person_id).await) {
            ProcessedResponse::Ok(details) => details,
            ProcessedResponse::Err(error) => {
                warn!("{error}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            ProcessedResponse::Response(response) => return response,
        };

        let credits = match process_response(
            get_combined_credits(&api_state.tmdb, &person_id.to_string()).await,
        ) {
            ProcessedResponse::Ok(credits) => credits,
            ProcessedResponse::Err(error) => {
                warn!("{error}");

                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            ProcessedResponse::Response(response) => return response,
        };

        let cast_iter = credits.cast.into_iter().map(Credit::Cast);
        let crew_iter = credits.crew.into_iter().map(Credit::Crew);

        let mut credits = cast_iter
            .merge_by(crew_iter, |_a, _b| true)
            .filter(|credit| query.release_status.check(credit.release_date()))
            .collect_vec();

        credits.sort_by(|a, b| {
            query
                .sort_order
                .sort_release_date(a.release_date(), b.release_date())
        });

        let items_size = query.size.min(MAX_SIZE);
        let mut items: Vec<Item> = Vec::with_capacity(items_size);

        for credit in credits.into_iter().take(items_size) {
            let mut item = ItemBuilder::default();

            item.guid(Some(credit_guid(&credit)));
            item.category(Category::from(sanitise_text(
                credit.media_type().to_string(),
            )));

            let mut description = credit
                .overview_len()
                .map(String::with_capacity)
                .unwrap_or(String::new());

            description.push_str("<p>");

            match &credit {
                Credit::Cast(cast) => {
                    description.push_str("Character: ");

                    match cast.character() {
                        None => {
                            description.push_str("TBA");
                        }
                        Some(character) => {
                            description.push_str(character.as_str());
                            item.category(Category::from(sanitise_text(character)));
                        }
                    }
                }
                Credit::Crew(crew) => {
                    description.push_str("Department: ");
                    description.push_str(crew.department());

                    description.push_str("<br>Job: ");
                    description.push_str(crew.job());
                }
            };

            description.push_str("<br>Genres: ");
            credit.genres().iter().enumerate().for_each(|(i, genre)| {
                if i > 0 {
                    description.push_str(", ");
                }

                description.push_str(genre.name());
                item.category(Category::from(genre.name()));
            });

            description.push_str("<br>Language: ");
            description.push_str(credit.original_language());

            description.push_str("<br>Release Date: ");
            match credit.release_date() {
                None => description.push_str("TBA"),
                Some(date) => {
                    let date = date.format("%d-%b-%Y").to_string();
                    description.push_str(date.as_str());
                }
            }

            if let Some(overview) = credit.overview() {
                description.push_str("</p><p>");
                description.push_str(overview.as_str());
                description.push_str("</p>");
            }

            item.link(credit.tmdb_media_url().to_string())
                .title(Some(credit.title().to_string()))
                .description(sanitise_text(description));

            items.push(item.build());
        }

        #[cfg(test)]
        let build_date: DateTime<Utc> = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(TEST_BUILD_DATE, TEST_BUILD_TIME),
            Utc,
        );
        #[cfg(not(test))]
        let build_date = Utc::now();

        let mut channel = ChannelBuilder::default();

        channel
            .title(sanitise_text(format!(
                "{} - Combined Credits",
                details.name
            )))
            .link(details.tmdb_url())
            .last_build_date(build_date.format("%a, %d %b %Y %H:%M %Z").to_string())
            .generator(Some(
                "Movie Feed <https://github.com/calum4/movie-feed/>".to_string(),
            ))
            .docs(Some(
                "https://www.rssboard.org/rss-specification".to_string(),
            ))
            .ttl(Some((TTL.as_secs() * 60).to_string()))
            .items(items);

        if let Some(bio) = details.biography {
            channel.description(sanitise_text(bio));
        }

        let rss = Rss::new(channel.build());
        rss.into_response()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use axum::body::HttpBody;
        use axum::http::Uri;
        use chrono::TimeDelta;
        use tmdb::Tmdb;
        use tmdb_test_utils::api::v3::person::combined_credits::mock_get_person_combined_credits;
        use tmdb_test_utils::api::v3::person::mock_get_person_details;
        use tmdb_test_utils::mockito::{Mock, ServerGuard};
        use tmdb_test_utils::start_mock_tmdb_api;

        async fn init(person_id: i32) -> (Tmdb, ServerGuard, (Mock, Mock)) {
            let mut server = start_mock_tmdb_api().await;

            let details_mock = mock_get_person_details(&mut server, person_id).await;
            let credits_mock =
                mock_get_person_combined_credits(&mut server, person_id.to_string().as_str()).await;

            let mut tmdb = Tmdb::default();
            tmdb.override_api_url(server.url().as_str()).unwrap();

            (tmdb, server, (details_mock, credits_mock))
        }

        async fn combined_credits(person_id: i32, query_args: QueryArgs) -> axum::body::Bytes {
            let (tmdb, _server, _) = init(person_id).await;

            let api_state = ApiState { tmdb };

            let body = super::combined_credits(
                Path(person_id),
                Extension(Arc::new(api_state)),
                Query(query_args),
            )
            .await
            .into_body();

            let size = {
                let size = body.size_hint();
                usize::try_from(size.exact().unwrap_or(size.lower()))
            }
            .unwrap();

            axum::body::to_bytes(body, size).await.unwrap()
        }

        #[tokio::test]
        async fn test_get_release_status_all() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs {
                release_status: ReleaseStatus::All,
                ..QueryArgs::default()
            };
            let bytes = combined_credits(PERSON_ID, query_args).await;

            assert_eq!(
                String::from_utf8_lossy(bytes.as_ref()),
                include_str!("../../../../tests/assets/api/person/get_combined_credits_19498.xml")
            );
        }

        #[tokio::test]
        async fn test_get_release_status_released() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs {
                release_status: ReleaseStatus::Released {
                    max_age: Default::default(),
                    min_age: Default::default(),
                },
                ..QueryArgs::default()
            };
            let bytes = combined_credits(PERSON_ID, query_args).await;

            assert_eq!(
                String::from_utf8_lossy(bytes.as_ref()),
                include_str!(
                    "../../../../tests/assets/api/person/get_combined_credits_19498_released.xml"
                )
            );
        }

        #[tokio::test]
        async fn test_size_default() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs::default();
            let bytes = combined_credits(PERSON_ID, query_args).await;

            let credits = String::from_utf8_lossy(bytes.as_ref());
            assert_eq!(credits.matches("<item>").count(), DEFAULT_SIZE);
        }

        #[tokio::test]
        async fn test_size_5() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs {
                size: 5,
                ..QueryArgs::default()
            };
            let bytes = combined_credits(PERSON_ID, query_args).await;

            let credits = String::from_utf8_lossy(bytes.as_ref());
            assert_eq!(credits.matches("<item>").count(), 5);
        }

        #[tokio::test]
        async fn test_size_30() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs {
                size: 30,
                ..QueryArgs::default()
            };
            let bytes = combined_credits(PERSON_ID, query_args).await;

            let credits = String::from_utf8_lossy(bytes.as_ref());
            assert_eq!(credits.matches("<item>").count(), 30);
        }

        #[tokio::test]
        async fn test_size_max() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs {
                size: usize::MAX,
                ..QueryArgs::default()
            };
            let bytes = combined_credits(PERSON_ID, query_args).await;

            let credits = String::from_utf8_lossy(bytes.as_ref());
            assert_eq!(credits.matches("<item>").count(), MAX_SIZE);
        }

        #[test]
        fn test_query_args_default_deserialisation() {
            let uri = Uri::from_static(r##"https://example.com"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            assert_eq!(query.unwrap().0, QueryArgs::default());
        }

        #[test]
        fn test_query_args_size_deserialisation() {
            let uri = Uri::from_static(r##"https://example.com?size=10"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    size: 10,
                    ..Default::default()
                }
            );

            let uri = Uri::from_static(r##"https://example.com?size=-5"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            assert!(query.is_err());
        }

        #[test]
        fn test_query_args_release_status_deserialisation() {
            // Unreleased
            let uri = Uri::from_static(r##"https://example.com?release_status=Unreleased"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::Unreleased {
                max_time_until_release: None,
            };
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );

            let uri = Uri::from_static(
                r##"https://example.com?release_status=Unreleased&max_time_until_release=5m"##,
            );
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::Unreleased {
                max_time_until_release: Some(TimeDelta::minutes(5)),
            };
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );

            // Released
            let uri = Uri::from_static(r##"https://example.com?release_status=Released"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::Released {
                max_age: None,
                min_age: None,
            };
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );

            let uri = Uri::from_static(
                r##"https://example.com?release_status=Released&max_age=5h&min_age=5m"##,
            );
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::Released {
                max_age: Some(TimeDelta::hours(5)),
                min_age: Some(TimeDelta::minutes(5)),
            };
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );

            let uri = Uri::from_static(
                r##"https://example.com?release_status=Released&max_age=1s&min_age=5m"##,
            );
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            assert_eq!(
                query.unwrap_err().to_string(),
                format!(
                    "Failed to deserialize query string: {}",
                    ReleaseStatusError::MaxAgeSmaller.text()
                )
            );

            // HasReleaseDate
            let uri = Uri::from_static(r##"https://example.com?release_status=HasReleaseDate"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::HasReleaseDate {
                max_time_until_release: None,
                max_age: None,
            };
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );

            let uri = Uri::from_static(
                r##"https://example.com?release_status=HasReleaseDate&max_time_until_release=52w&max_age=5h"##,
            );
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::HasReleaseDate {
                max_time_until_release: Some(TimeDelta::weeks(52)),
                max_age: Some(TimeDelta::hours(5)),
            };
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );

            // NoReleaseDate
            let uri = Uri::from_static(r##"https://example.com?release_status=NoReleaseDate"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::NoReleaseDate;
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );

            // All
            let uri = Uri::from_static(r##"https://example.com?release_status=All"##);
            let query = Query::<QueryArgs>::try_from_uri(&uri);
            let release_status = ReleaseStatus::All;
            assert_eq!(
                query.unwrap().0,
                QueryArgs {
                    release_status,
                    ..Default::default()
                }
            );
        }
    }
}
