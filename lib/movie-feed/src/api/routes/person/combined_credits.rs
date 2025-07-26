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
    use crate::api::rss::Rss;
    use axum::Extension;
    use axum::extract::{Path, Query};
    use axum::response::{IntoResponse, Response};
    #[cfg(test)]
    use chrono::{DateTime, NaiveDateTime, NaiveTime};
    use chrono::{Datelike, NaiveDate, Utc};
    use itertools::Itertools;
    use rss::{ChannelBuilder, Guid, GuidBuilder, Item, ItemBuilder};
    use serde::Deserialize;
    use std::cmp::Ordering;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::sync::Arc;
    use std::time::Duration;
    use tmdb::endpoints::v3::person::combined_credits::get as get_combined_credits;
    use tmdb::endpoints::v3::person::get as get_person_details;
    use tmdb::models::v3::credit::{Credit, IsCredit};
    use tracing::warn;

    const TTL: Duration = Duration::from_secs(60 * 60); // 60 minutes
    const SIZE: usize = 20;

    #[cfg(test)]
    const TEST_BUILD_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 5, 21).unwrap();
    #[cfg(test)]
    const TEST_BUILD_TIME: NaiveTime = NaiveTime::from_hms_opt(18, 20, 44).unwrap();

    fn sort_release_date_descending(a: Option<&NaiveDate>, b: Option<&NaiveDate>) -> Ordering {
        match (a, b) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => b.cmp(a),
        }
    }

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

        s
    }

    #[derive(Deserialize, Copy, Clone)]
    enum ReleaseStatus {
        Unreleased,
        Released,
        HasReleaseDate,
        NoReleaseDate,
        All,
    }

    impl Default for ReleaseStatus {
        fn default() -> Self {
            Self::HasReleaseDate
        }
    }

    impl ReleaseStatus {
        fn check(&self, release_date: Option<&NaiveDate>) -> bool {
            match self {
                Self::Unreleased => {
                    let Some(date) = release_date else {
                        return true;
                    };

                    let now = Utc::now();
                    let now = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                        .expect("constructed from Utc::now(), should always be valid");

                    date.gt(&now)
                }
                Self::Released => {
                    let Some(date) = release_date else {
                        return false;
                    };

                    let now = Utc::now();
                    let now = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                        .expect("constructed from Utc::now(), should always be valid");

                    date.le(&now)
                }
                Self::NoReleaseDate => release_date.is_none(),
                Self::HasReleaseDate => release_date.is_some(),
                Self::All => true,
            }
        }
    }

    #[derive(Deserialize, Default)]
    pub(super) struct QueryArgs {
        #[serde(default)]
        size: Option<usize>,
        #[serde(default)]
        release_status: ReleaseStatus,
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

        credits.sort_by(|a, b| sort_release_date_descending(a.release_date(), b.release_date()));

        let items_size = query.size.unwrap_or(SIZE);
        let mut items: Vec<Item> = Vec::with_capacity(items_size);

        for credit in credits.into_iter().take(items_size) {
            let mut item = ItemBuilder::default();

            item.guid(Some(credit_guid(&credit)));

            let mut description = credit
                .overview_len()
                .map(String::with_capacity)
                .unwrap_or(String::new());

            description.push_str("<p>");

            match &credit {
                Credit::Cast(cast) => {
                    description.push_str("Character: ");
                    description.push_str(cast.character().map_or("TBA", |c| c.as_str()));
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
            .title(format!("{} - Combined Credits", details.name))
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
            channel.description(sanitise_text(bio.as_str()));
        }

        let rss = Rss::new(channel.build());
        rss.into_response()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use axum::body::HttpBody;
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
                release_status: ReleaseStatus::Released,
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
            assert_eq!(credits.matches("<item>").count(), SIZE);
        }

        #[tokio::test]
        async fn test_size_5() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs {
                size: Some(5),
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
                size: Some(30),
                ..QueryArgs::default()
            };
            let bytes = combined_credits(PERSON_ID, query_args).await;

            let credits = String::from_utf8_lossy(bytes.as_ref());
            assert_eq!(credits.matches("<item>").count(), 30);
        }

        #[test]
        fn test_release_date_sorting() {
            let mut dates = vec![
                None,
                NaiveDate::from_ymd_opt(2025, 4, 10),
                None,
                NaiveDate::from_ymd_opt(2025, 5, 14),
                NaiveDate::from_ymd_opt(2026, 1, 1),
                None,
                NaiveDate::from_ymd_opt(1990, 6, 1),
            ];

            let sorted_dates = vec![
                None,
                None,
                None,
                NaiveDate::from_ymd_opt(2026, 1, 1),
                NaiveDate::from_ymd_opt(2025, 5, 14),
                NaiveDate::from_ymd_opt(2025, 4, 10),
                NaiveDate::from_ymd_opt(1990, 6, 1),
            ];

            dates.sort_by(|a, b| sort_release_date_descending(a.as_ref(), b.as_ref()));

            assert_eq!(dates, sorted_dates);
        }

        #[test]
        fn test_release_status() {
            let now = Utc::now();
            let now = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                .expect("constructed from Utc::now(), should always be valid");

            assert!(ReleaseStatus::Unreleased.check(None));
            assert!(!ReleaseStatus::Unreleased.check(Some(&NaiveDate::MIN)));
            assert!(!ReleaseStatus::Unreleased.check(Some(&now)));
            assert!(ReleaseStatus::Unreleased.check(Some(&NaiveDate::MAX)));

            assert!(!ReleaseStatus::Released.check(None));
            assert!(ReleaseStatus::Released.check(Some(&NaiveDate::MIN)));
            assert!(ReleaseStatus::Released.check(Some(&now)));
            assert!(!ReleaseStatus::Released.check(Some(&NaiveDate::MAX)));

            assert!(!ReleaseStatus::HasReleaseDate.check(None));
            assert!(ReleaseStatus::HasReleaseDate.check(Some(&NaiveDate::MIN)));
            assert!(ReleaseStatus::HasReleaseDate.check(Some(&now)));
            assert!(ReleaseStatus::HasReleaseDate.check(Some(&NaiveDate::MAX)));

            assert!(ReleaseStatus::NoReleaseDate.check(None));
            assert!(!ReleaseStatus::NoReleaseDate.check(Some(&NaiveDate::MIN)));
            assert!(!ReleaseStatus::NoReleaseDate.check(Some(&now)));
            assert!(!ReleaseStatus::NoReleaseDate.check(Some(&NaiveDate::MAX)));

            assert!(ReleaseStatus::All.check(None));
            assert!(ReleaseStatus::All.check(Some(&NaiveDate::MIN)));
            assert!(ReleaseStatus::All.check(Some(&now)));
            assert!(ReleaseStatus::All.check(Some(&NaiveDate::MAX)));
        }
    }
}
