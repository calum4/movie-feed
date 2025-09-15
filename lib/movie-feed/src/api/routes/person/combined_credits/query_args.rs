use crate::api::routes::person::combined_credits::release_status::ReleaseStatus;
use crate::api::routes::person::combined_credits::release_status::deserialize_release_status;
use crate::api::routes::person::combined_credits::sort_order::SortReleaseDates;
use serde::Deserialize;
use utils::const_assert;

pub(super) const DEFAULT_SIZE: usize = 20;
pub(super) const MAX_SIZE: usize = 50;
const_assert!(
    DEFAULT_SIZE <= MAX_SIZE,
    "MAX_SIZE must not exceed DEFAULT_SIZE"
);

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub(super) struct QueryArgs {
    #[serde(default = "serde_utils::defaults::default_usize::<DEFAULT_SIZE>")]
    /// Number of credits to return
    pub(super) size: usize,
    #[serde(flatten, deserialize_with = "deserialize_release_status")]
    /// The release status of the credits to return
    pub(super) release_status: ReleaseStatus,
    #[serde(default)]
    pub(super) sort_order: SortReleaseDates,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::routes::person::combined_credits::release_status::ReleaseStatusError;
    use axum::extract::Query;
    use axum::http::Uri;
    use chrono::TimeDelta;

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
