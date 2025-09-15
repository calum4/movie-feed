use crate::api::routes::person::combined_credits::release_status::ReleaseStatus;
use crate::api::routes::person::combined_credits::release_status::deserialize_release_status;
use crate::api::routes::person::combined_credits::size::Size;
use crate::api::routes::person::combined_credits::sort_order::SortReleaseDates;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug, Eq, PartialEq)]
pub(super) struct QueryArgs {
    #[serde(default)]
    /// Number of credits to return
    pub(super) size: Size,
    #[serde(flatten, deserialize_with = "deserialize_release_status")]
    /// The release status of the credits to return
    pub(super) release_status: ReleaseStatus,
    #[serde(default)]
    pub(super) sort_order: SortReleaseDates,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::routes::person::combined_credits::release_status::ReleaseStatusError;
    use axum::extract::Query;
    use axum::http::Uri;
    use chrono::TimeDelta;
    use std::str::FromStr;

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
                size: Size::try_from(10).unwrap(),
                ..Default::default()
            }
        );

        let uri = Uri::from_static(r##"https://example.com?size=-5"##);
        let query = Query::<QueryArgs>::try_from_uri(&uri);
        assert!(query.is_err());
        assert_eq!(
            query.err().unwrap().to_string(),
            "Failed to deserialize query string: size: invalid digit found in string"
        );

        let uri = Uri::from_str(
            format!("https://example.com?size={}", Size::MAX_SIZE.get() + 1).as_str(),
        )
        .unwrap();
        let query = Query::<QueryArgs>::try_from_uri(&uri);
        assert!(query.is_err());
        assert_eq!(
            query.err().unwrap().to_string(),
            "Failed to deserialize query string: size: size must not exceed the max size"
        );

        let uri = Uri::from_static("https://example.com?size=0");
        let query = Query::<QueryArgs>::try_from_uri(&uri);
        assert!(query.is_err());
        assert_eq!(
            query.err().unwrap().to_string(),
            "Failed to deserialize query string: size: invalid value: integer `0`, expected a nonzero usize"
        );
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
