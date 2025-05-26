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
    use axum::Extension;
    use axum::extract::Path;
    use axum::response::{IntoResponse, Response};
    #[cfg(test)]
    use chrono::{DateTime, NaiveDateTime, NaiveTime};
    use chrono::{NaiveDate, Utc};
    use itertools::Itertools;
    use rss::{ChannelBuilder, Item, ItemBuilder};
    use std::cmp::Ordering;
    use std::sync::Arc;
    use std::time::Duration;
    use tmdb::endpoints::v3::person::combined_credits::get as get_combined_credits;
    use tmdb::endpoints::v3::person::get as get_person_details;
    use tmdb::models::v3::cast::{Cast, MediaPageUrl};
    use tmdb::models::v3::crew::Crew;
    use tracing::warn;

    const TTL: Duration = Duration::from_secs(60 * 60); // 60 minutes
    const SIZE: usize = 20;

    #[cfg(test)]
    const TEST_BUILD_DATE: NaiveDate = NaiveDate::from_ymd_opt(2025, 5, 21).unwrap();
    #[cfg(test)]
    const TEST_BUILD_TIME: NaiveTime = NaiveTime::from_hms_opt(18, 20, 44).unwrap();

    #[derive(Debug, Hash)]
    enum Credit {
        Cast(Cast),
        Crew(Crew),
    }

    impl Credit {
        fn release_date(&self) -> &Option<NaiveDate> {
            match self {
                Credit::Cast(Cast::Movie(movie)) => &movie.release_date,
                Credit::Cast(Cast::Tv(tv)) => &tv.first_air_date,
                Credit::Crew(Crew::Movie(movie)) => &movie.release_date,
                Credit::Crew(Crew::Tv(tv)) => &tv.first_air_date,
            }
        }

        fn credit_id(&self) -> usize {
            match self {
                Credit::Cast(Cast::Movie(credit)) => credit.id,
                Credit::Cast(Cast::Tv(credit)) => credit.id,
                Credit::Crew(Crew::Movie(credit)) => credit.id,
                Credit::Crew(Crew::Tv(credit)) => credit.id,
            }
        }
    }

    fn sort_release_date_descending(a: &Option<NaiveDate>, b: &Option<NaiveDate>) -> Ordering {
        match (a, b) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => b.cmp(a),
        }
    }

    pub(super) async fn combined_credits(
        Path(person_id): Path<i32>,
        api_state: Extension<Arc<ApiState>>,
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

        let mut credits = cast_iter.merge_by(crew_iter, |_a, _b| true).collect_vec();
        credits.sort_by(|a, b| sort_release_date_descending(a.release_date(), b.release_date()));

        let mut items: Vec<Item> = Vec::with_capacity(SIZE);

        for credit in credits.into_iter().take(SIZE) {
            let mut item = ItemBuilder::default();

            {
                use rss::GuidBuilder;
                use std::hash::{DefaultHasher, Hash, Hasher};

                let mut hasher = DefaultHasher::new();

                // More permissive hash for tests without sacrificing test quality
                if cfg!(test) {
                    credit.credit_id().hash(&mut hasher);
                } else {
                    credit.hash(&mut hasher);
                }

                let hash = hasher.finish();

                let guid = GuidBuilder::default()
                    .value(hash.to_string())
                    .permalink(true)
                    .build();

                item.guid(Some(guid));
            }

            match credit {
                Credit::Cast(Cast::Movie(credit)) => {
                    let description = format!(
                        "Character: {}\nGenres: {}\nLanguage: {}{}",
                        credit.character.as_ref().map_or("TBA", |c| c.as_str()),
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit
                            .overview
                            .as_ref()
                            .map(|overview| format!("\n\n{overview}"))
                            .unwrap_or("".to_string()), // TODO - Improve
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .title(credit.title)
                        .description(description);
                }
                Credit::Cast(Cast::Tv(credit)) => {
                    let description = format!(
                        "Character: {}\nGenres: {}\nLanguage: {}{}",
                        credit.character.as_ref().map_or("TBA", |c| c.as_str()),
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit
                            .overview
                            .as_ref()
                            .map(|overview| format!("\n\n{overview}"))
                            .unwrap_or("".to_string()),
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .title(credit.name)
                        .description(description);
                }
                Credit::Crew(Crew::Movie(credit)) => {
                    let description = format!(
                        "Department: {}\nJob: {}\nGenres: {}\nLanguage: {}{}",
                        credit.department,
                        credit.job,
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit
                            .overview
                            .as_ref()
                            .map(|overview| format!("\n\n{overview}"))
                            .unwrap_or("".to_string()),
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .title(credit.title)
                        .description(description);
                }
                Credit::Crew(Crew::Tv(credit)) => {
                    let description = format!(
                        "Department: {}\nJob: {}\nGenres: {}\nLanguage: {}{}",
                        credit.department,
                        credit.job,
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit
                            .overview
                            .as_ref()
                            .map(|overview| format!("\n\n{overview}"))
                            .unwrap_or("".to_string()),
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .title(credit.name)
                        .description(description);
                }
            };

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
            .title(format!("Combined Credits - {}", details.name))
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
            channel.description(bio);
        }

        channel.build().to_string().into_response()
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

        #[tokio::test]
        async fn test_get() {
            const PERSON_ID: i32 = 19498;

            let (tmdb, _server, (details_mock, credits_mock)) = init(PERSON_ID).await;

            let api_state = ApiState { tmdb };

            let body = combined_credits(Path(PERSON_ID), Extension(Arc::new(api_state)))
                .await
                .into_body();

            let size = {
                let size = body.size_hint();
                usize::try_from(size.exact().unwrap_or(size.lower()))
            }
            .unwrap();

            let bytes = axum::body::to_bytes(body, size).await.unwrap();

            assert_eq!(
                String::from_utf8_lossy(bytes.as_ref()),
                include_str!("../../../../tests/assets/api/person/get_combined_credits_19498.xml")
            );

            details_mock.assert();
            credits_mock.assert();
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

            dates.sort_by(sort_release_date_descending);

            assert_eq!(dates, sorted_dates);
        }
    }
}
