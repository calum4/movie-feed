mod query_args;
mod release_status;
mod size;
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
    use crate::api::routes::person::combined_credits::query_args::QueryArgs;
    use crate::api::rss::Rss;
    use ammonia::Builder;
    use axum::Extension;
    use axum::extract::{Path, Query};
    use axum::response::{IntoResponse, Response};
    use chrono::Utc;
    #[cfg(test)]
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};
    use itertools::Itertools;
    use rss::{Category, ChannelBuilder, Guid, GuidBuilder, Item, ItemBuilder};
    use std::collections::HashSet;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::sync::Arc;
    use std::time::Duration;
    use tmdb::endpoints::v3::person::combined_credits::get as get_combined_credits;
    use tmdb::endpoints::v3::person::get as get_person_details;
    use tmdb::models::v3::credit::{Credit, IsCredit};
    use tracing::warn;

    const TTL: Duration = Duration::from_secs(60 * 60); // 60 minutes

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

        let mut items: Vec<Item> = Vec::with_capacity(query.size.get());

        for credit in credits.into_iter().take(query.size.get()) {
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
        use crate::api::routes::person::combined_credits::release_status::ReleaseStatus;
        use crate::api::routes::person::combined_credits::size::Size;
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
            assert_eq!(credits.matches("<item>").count(), Size::default().get());
        }

        #[tokio::test]
        async fn test_size_5() {
            const PERSON_ID: i32 = 19498;

            let query_args = QueryArgs {
                size: Size::try_from(5).unwrap(),
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
                size: Size::try_from(30).unwrap(),
                ..QueryArgs::default()
            };
            let bytes = combined_credits(PERSON_ID, query_args).await;

            let credits = String::from_utf8_lossy(bytes.as_ref());
            assert_eq!(credits.matches("<item>").count(), 30);
        }
    }
}
