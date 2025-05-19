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
    use axum::Extension;
    use axum::extract::Path;
    use axum::response::{IntoResponse, Response};
    use chrono::{NaiveDate, Utc};
    use itertools::Itertools;
    use rss::{ChannelBuilder, Guid, GuidBuilder, Item, ItemBuilder};
    use std::cmp::Ordering;
    use std::sync::Arc;
    use std::time::Duration;
    use tmdb::endpoints::v3::person::combined_credits::get as get_combined_credits;
    use tmdb::endpoints::v3::person::get as get_person_details;
    use tmdb::models::v3::cast::{Cast, IsCredit, MediaPageUrl, MediaTypeDefinition};
    use tmdb::models::v3::crew::Crew;
    use tracing::warn;

    const TTL: Duration = Duration::from_secs(60 * 60); // 60 minutes
    const SIZE: usize = 20;

    #[derive(Debug)]
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
    }

    fn sort_release_date_descending(a: &Option<NaiveDate>, b: &Option<NaiveDate>) -> Ordering {
        match (a, b) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => b.cmp(a),
        }
    }

    #[inline]
    fn guid<T: IsCredit + MediaTypeDefinition>(
        credit_id: usize,
        job: &str,
        credit: &T,
    ) -> Option<Guid> {
        let tmdb_url_prefix = credit
            .media_type()
            .tmdb_url_prefix()
            .expect("guaranteed by tests to be Some(_)");

        Some(
            GuidBuilder::default()
                .value(format!("{tmdb_url_prefix}-{job}-{credit_id}"))
                .permalink(true)
                .build(),
        )
    }

    pub(super) async fn combined_credits(
        Path(person_id): Path<i32>,
        api_state: Extension<Arc<ApiState>>,
    ) -> Response {
        let details = match get_person_details(&api_state.tmdb, person_id).await {
            Ok(details) => details,
            Err(error) => {
                warn!("{error}");

                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        let credits = match get_combined_credits(&api_state.tmdb, &person_id.to_string()).await {
            Ok(credits) => credits,
            Err(error) => {
                warn!("{error}");

                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

        let cast_iter = credits.cast.into_iter().map(Credit::Cast);
        let crew_iter = credits.crew.into_iter().map(Credit::Crew);

        let mut credits = cast_iter.merge_by(crew_iter, |_a, _b| true).collect_vec();
        credits.sort_by(|a, b| sort_release_date_descending(a.release_date(), b.release_date()));

        let mut items: Vec<Item> = Vec::with_capacity(SIZE);

        for credit in credits.into_iter().take(SIZE) {
            let mut item = ItemBuilder::default();

            match credit {
                Credit::Cast(Cast::Movie(credit)) => {
                    let description = format!(
                        "Character: {}\nGenres: {}\nLanguage: {}\n\n{}",
                        credit.character.as_ref().map_or("TBA", |c| c.as_str()),
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit.overview
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .guid(guid(credit.id, "Actor", &credit))
                        .title(credit.title)
                        .description(description);
                }
                Credit::Cast(Cast::Tv(credit)) => {
                    let description = format!(
                        "Character: {}\nGenres: {}\nLanguage: {}\n\n{}",
                        credit.character.as_ref().map_or("TBA", |c| c.as_str()),
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit.overview
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .guid(guid(credit.id, "Actor", &credit))
                        .title(credit.name)
                        .description(description);
                }
                Credit::Crew(Crew::Movie(credit)) => {
                    let description = format!(
                        "Department: {}\nJob: {}\nGenres: {}\nLanguage: {}\n\n{}",
                        credit.department,
                        credit.job,
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit.overview
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .guid(guid(credit.id, credit.job.as_str(), &credit))
                        .title(credit.title)
                        .description(description);
                }
                Credit::Crew(Crew::Tv(credit)) => {
                    let description = format!(
                        "Department: {}\nJob: {}\nGenres: {}\nLanguage: {}\n\n{}",
                        credit.department,
                        credit.job,
                        credit.genres.iter().map(|genre| genre.name()).join(", "),
                        credit.original_language,
                        credit.overview
                    );

                    item.link(credit.tmdb_media_url().to_string())
                        .guid(guid(credit.id, credit.job.as_str(), &credit))
                        .title(credit.name)
                        .description(description);
                }
            };

            items.push(item.build());
        }

        let mut channel = ChannelBuilder::default();

        channel
            .title(format!("Combined Credits - {}", details.name))
            .link(details.tmdb_url())
            .last_build_date(Utc::now().format("%a, %d %b %Y %H:%M %Z").to_string())
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

        #[tokio::test]
        async fn test_get() {
            let api_state = ApiState {
                tmdb: Tmdb::default(),
            };

            let a = combined_credits(Path(19498), Extension(Arc::new(api_state))).await;
            let b = a.into_body();

            let size = {
                let size = b.size_hint();
                usize::try_from(size.exact().unwrap_or(size.lower()))
            }
            .unwrap();

            let bytes = axum::body::to_bytes(b, size).await.unwrap();

            let test = String::from_utf8_lossy(bytes.as_ref());

            dbg!(test);
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
