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
    use ammonia::Builder;
    use axum::Extension;
    use axum::extract::{Path, Query};
    use axum::response::{IntoResponse, Response};
    #[cfg(test)]
    use chrono::{DateTime, NaiveDateTime, NaiveTime};
    use chrono::{Datelike, NaiveDate, TimeDelta, Utc};
    use itertools::Itertools;
    use rss::{Category, ChannelBuilder, Guid, GuidBuilder, Item, ItemBuilder};
    use serde::{Deserialize, Deserializer};
    use std::cmp::Ordering;
    use std::collections::HashSet;
    use std::fmt::{Display, Formatter};
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use thiserror::Error;
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

        let mut tags = HashSet::new();
        tags.insert("br");
        tags.insert("p");

        s = Builder::new().tags(tags).clean(s.as_ref()).to_string();

        s
    }

    #[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
    #[serde(tag = "release_status")]
    enum ReleaseStatus {
        Unreleased {
            #[serde(default, deserialize_with = "deserialize_time_delta")]
            max_time_until_release: Option<TimeDelta>,
        },
        Released {
            #[serde(default, deserialize_with = "deserialize_time_delta")]
            max_age: Option<TimeDelta>,
            #[serde(default, deserialize_with = "deserialize_time_delta")]
            min_age: Option<TimeDelta>,
        },
        HasReleaseDate {
            #[serde(default, deserialize_with = "deserialize_time_delta")]
            max_time_until_release: Option<TimeDelta>,
            #[serde(default, deserialize_with = "deserialize_time_delta")]
            max_age: Option<TimeDelta>,
        },
        NoReleaseDate,
        All,
    }

    pub(super) fn deserialize_time_delta<'de, D>(
        deserializer: D,
    ) -> Result<Option<TimeDelta>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str: &str = Deserialize::deserialize(deserializer)?;
        let duration = humantime::Duration::from_str(str).map_err(serde::de::Error::custom)?;

        TimeDelta::from_std(*duration)
            .map(Into::into)
            .map_err(serde::de::Error::custom)
    }

    impl Default for ReleaseStatus {
        fn default() -> Self {
            Self::HasReleaseDate {
                max_time_until_release: None,
                max_age: None,
            }
        }
    }

    impl ReleaseStatus {
        fn check_max_time_until_release(
            now: &NaiveDate,
            release_date: &NaiveDate,
            max_time_until_release: &Option<TimeDelta>,
        ) -> bool {
            if let Some(max_time_until_release) = max_time_until_release {
                let Some(max_release_date) = now.checked_add_signed(*max_time_until_release) else {
                    return false;
                };

                return release_date.lt(&max_release_date);
            }

            true
        }

        fn check_max_age(
            now: &NaiveDate,
            release_date: &NaiveDate,
            max_age: &Option<TimeDelta>,
        ) -> bool {
            if let Some(max_age) = max_age {
                let Some(max_age_date) = now.checked_sub_signed(*max_age) else {
                    return false;
                };

                if release_date.lt(&max_age_date) {
                    return false;
                }
            }

            true
        }

        fn check_min_age(
            now: &NaiveDate,
            release_date: &NaiveDate,
            min_age: &Option<TimeDelta>,
        ) -> bool {
            if let Some(min_age) = min_age {
                let Some(min_age_date) = now.checked_sub_signed(*min_age) else {
                    return false;
                };

                if release_date.gt(&min_age_date) {
                    return false;
                }
            }

            true
        }

        fn check(&self, release_date: Option<&NaiveDate>) -> bool {
            match self {
                Self::Unreleased {
                    max_time_until_release,
                } => {
                    let Some(date) = release_date else {
                        return true;
                    };

                    let now = Utc::now();
                    let now = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                        .expect("constructed from Utc::now(), should always be valid");

                    date.gt(&now)
                        && ReleaseStatus::check_max_time_until_release(
                            &now,
                            date,
                            max_time_until_release,
                        )
                }
                Self::Released { max_age, min_age } => {
                    let Some(date) = release_date else {
                        return false;
                    };

                    let now = Utc::now();
                    let now = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                        .expect("constructed from Utc::now(), should always be valid");

                    date.le(&now)
                        && ReleaseStatus::check_max_age(&now, date, max_age)
                        && ReleaseStatus::check_min_age(&now, date, min_age)
                }
                Self::HasReleaseDate {
                    max_time_until_release,
                    max_age,
                } => {
                    let Some(date) = release_date else {
                        return false;
                    };

                    let now = Utc::now();
                    let now = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                        .expect("constructed from Utc::now(), should always be valid");

                    ReleaseStatus::check_max_time_until_release(&now, date, max_time_until_release)
                        && ReleaseStatus::check_max_age(&now, date, max_age)
                }
                Self::NoReleaseDate => release_date.is_none(),
                Self::All => true,
            }
        }
    }

    #[derive(Deserialize, Default, Debug, Eq, PartialEq)]
    pub(super) struct QueryArgs {
        #[serde(default)]
        size: Option<usize>,
        #[serde(flatten, deserialize_with = "deserialize_release_status")]
        release_status: ReleaseStatus,
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

        if let ReleaseStatus::Released { max_age, min_age } = &release {
            if max_age < min_age {
                return Err(serde::de::Error::custom(ReleaseStatusError::MaxAgeSmaller));
            }
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

        credits.sort_by(|a, b| sort_release_date_descending(a.release_date(), b.release_date()));

        let items_size = query.size.unwrap_or(SIZE);
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
        use chrono::{Days, Months};
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

        struct ReleaseStatusInit {
            now: NaiveDate,

            past_one_week: NaiveDate,
            future_one_week: NaiveDate,

            past_one_month: NaiveDate,
            future_one_month: NaiveDate,

            past_one_year: NaiveDate,
            future_one_year: NaiveDate,
        }

        fn init_release_status() -> ReleaseStatusInit {
            let now = Utc::now();
            let now = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
                .expect("constructed from Utc::now(), should always be valid");

            ReleaseStatusInit {
                now,
                past_one_week: now.checked_sub_days(Days::new(7)).unwrap(),
                future_one_week: now.checked_add_days(Days::new(7)).unwrap(),
                past_one_month: now.checked_sub_months(Months::new(1)).unwrap(),
                future_one_month: now.checked_add_months(Months::new(1)).unwrap(),
                past_one_year: now.checked_sub_months(Months::new(12)).unwrap(),
                future_one_year: now.checked_add_months(Months::new(12)).unwrap(),
            }
        }

        #[test]
        fn test_release_status_unreleased() {
            let data = init_release_status();
            let now = data.now;

            let unreleased = ReleaseStatus::Unreleased {
                max_time_until_release: None,
            };

            assert!(unreleased.check(None));
            assert!(!unreleased.check(Some(&NaiveDate::MIN)));
            assert!(!unreleased.check(Some(&now)));
            assert!(unreleased.check(Some(&NaiveDate::MAX)));

            let unreleased = ReleaseStatus::Unreleased {
                max_time_until_release: Some(TimeDelta::weeks(6)),
            };

            assert!(unreleased.check(None));
            assert!(!unreleased.check(Some(&NaiveDate::MIN)));
            assert!(!unreleased.check(Some(&now)));
            assert!(!unreleased.check(Some(&NaiveDate::MAX)));

            assert!(!unreleased.check(Some(&data.past_one_week)));
            assert!(unreleased.check(Some(&data.future_one_week)));
            assert!(!unreleased.check(Some(&data.past_one_month)));
            assert!(unreleased.check(Some(&data.future_one_month)));
            assert!(!unreleased.check(Some(&data.past_one_year)));
            assert!(!unreleased.check(Some(&data.future_one_year)));
        }

        #[test]
        fn test_release_status_released() {
            let data = init_release_status();
            let now = data.now;

            let released = ReleaseStatus::Released {
                max_age: None,
                min_age: None,
            };

            assert!(!released.check(None));
            assert!(released.check(Some(&NaiveDate::MIN)));
            assert!(released.check(Some(&now)));
            assert!(!released.check(Some(&NaiveDate::MAX)));

            let released = ReleaseStatus::Released {
                max_age: Some(TimeDelta::weeks(6)),
                min_age: Some(TimeDelta::weeks(2)),
            };

            assert!(!released.check(None));
            assert!(!released.check(Some(&NaiveDate::MIN)));
            assert!(!released.check(Some(&now)));
            assert!(!released.check(Some(&NaiveDate::MAX)));

            assert!(!released.check(Some(&data.past_one_week)));
            assert!(!released.check(Some(&data.future_one_week)));
            assert!(released.check(Some(&data.past_one_month)));
            assert!(!released.check(Some(&data.future_one_month)));
            assert!(!released.check(Some(&data.past_one_year)));
            assert!(!released.check(Some(&data.future_one_year)));
        }

        #[test]
        fn test_release_status_has_release_date() {
            let data = init_release_status();
            let now = data.now;

            let has_release_date = ReleaseStatus::HasReleaseDate {
                max_time_until_release: None,
                max_age: None,
            };

            assert!(!has_release_date.check(None));
            assert!(has_release_date.check(Some(&NaiveDate::MIN)));
            assert!(has_release_date.check(Some(&now)));
            assert!(has_release_date.check(Some(&NaiveDate::MAX)));

            let has_release_date = ReleaseStatus::HasReleaseDate {
                max_time_until_release: Some(TimeDelta::weeks(2)),
                max_age: Some(TimeDelta::weeks(6)),
            };

            assert!(!has_release_date.check(None));
            assert!(!has_release_date.check(Some(&NaiveDate::MIN)));
            assert!(has_release_date.check(Some(&now)));
            assert!(!has_release_date.check(Some(&NaiveDate::MAX)));

            assert!(has_release_date.check(Some(&data.past_one_week)));
            assert!(has_release_date.check(Some(&data.future_one_week)));
            assert!(has_release_date.check(Some(&data.past_one_month)));
            assert!(!has_release_date.check(Some(&data.future_one_month)));
            assert!(!has_release_date.check(Some(&data.past_one_year)));
            assert!(!has_release_date.check(Some(&data.future_one_year)));
        }

        #[test]
        fn test_release_status_no_release_date() {
            let data = init_release_status();
            let now = data.now;

            assert!(ReleaseStatus::NoReleaseDate.check(None));
            assert!(!ReleaseStatus::NoReleaseDate.check(Some(&NaiveDate::MIN)));
            assert!(!ReleaseStatus::NoReleaseDate.check(Some(&now)));
            assert!(!ReleaseStatus::NoReleaseDate.check(Some(&NaiveDate::MAX)));
        }

        #[test]
        fn test_release_status_all() {
            let data = init_release_status();
            let now = data.now;

            assert!(ReleaseStatus::All.check(None));
            assert!(ReleaseStatus::All.check(Some(&NaiveDate::MIN)));
            assert!(ReleaseStatus::All.check(Some(&now)));
            assert!(ReleaseStatus::All.check(Some(&NaiveDate::MAX)));
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
                    size: Some(10),
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
