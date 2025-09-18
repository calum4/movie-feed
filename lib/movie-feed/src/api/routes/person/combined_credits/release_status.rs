use chrono::{Datelike, NaiveDate, TimeDelta, Utc};
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
#[serde(tag = "release_status")]
/// Release date constraints used for selecting credits which fall within the defined range
pub(super) enum ReleaseStatus {
    /// Unreleased Credits
    ///
    /// # Default
    /// `Unreleased { max_time_until_release: None }`
    ///
    /// # Examples
    /// - `Unreleased { max_time_until_release: None }` - All unreleased credits
    /// - `Unreleased { max_time_until_release: Some("2 months") }` - All unreleased credits
    ///   that are no older than 2 months
    Unreleased {
        #[serde(default, deserialize_with = "deserialize_time_delta")]
        max_time_until_release: Option<TimeDelta>,
    },
    /// Released Credits
    ///
    /// # Default
    /// `Released { max_age: None, min_age: None}`
    ///
    /// # Examples
    /// - `Released { max_age: None, min_age: None}` - All released credits
    /// - `Released { max_age: Some("4 months"), min_age: None}` - Credits which released less
    ///   than 4 months ago
    /// - `Released { max_age: None, min_age: Some("2 months")}` - Credits which released more
    ///   than 2 months ago
    /// - `Released { max_age: Some("4 months"), min_age: Some("2 months")}` - Credits which
    ///   released more than 2 months ago and less than 4 months ago
    Released {
        #[serde(default, deserialize_with = "deserialize_time_delta")]
        max_age: Option<TimeDelta>,
        #[serde(default, deserialize_with = "deserialize_time_delta")]
        min_age: Option<TimeDelta>,
    },
    /// Credits with a set release date
    ///
    /// # Default
    /// `HasReleaseDate { max_time_until_release: None, max_age: None}`
    ///
    /// # Examples
    /// - `HasReleaseDate { max_time_until_release: None, max_age: None}` - All released or
    ///   unreleased credits which have a set release date
    /// - `HasReleaseDate { max_time_until_release: Some("2 months"), max_age: None}` - All
    ///   released credits and unreleased credits which release in no more than 2 months
    /// - `HasReleaseDate { max_time_until_release: None, max_age: Some("4 months")}` - All
    ///   unreleased credits and released credits which released no more than 4 months ago
    /// - `HasReleaseDate { max_time_until_release: Some("2 months"), max_age: Some("4 months")}`
    ///   \- All released credits which released no more than 4 months ago, and all unreleased
    ///   credits which release in no more than 2 months
    HasReleaseDate {
        #[serde(default, deserialize_with = "deserialize_time_delta")]
        max_time_until_release: Option<TimeDelta>,
        #[serde(default, deserialize_with = "deserialize_time_delta")]
        max_age: Option<TimeDelta>,
    },
    /// All credits without a release date
    NoReleaseDate,
    /// All credits
    All,
}

pub(super) fn deserialize_time_delta<'de, D>(deserializer: D) -> Result<Option<TimeDelta>, D::Error>
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

    #[cfg(not(test))]
    #[inline]
    fn date_now() -> NaiveDate {
        let now = Utc::now();

        NaiveDate::from_ymd_opt(now.year(), now.month(), now.day())
            .expect("constructed from Utc::now(), should always be valid")
    }

    #[cfg(test)]
    #[inline]
    // Hardcoded date for tests
    fn date_now() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 9, 18).expect("hardcoded")
    }

    pub(super) fn check(&self, release_date: Option<&NaiveDate>) -> bool {
        match self {
            Self::Unreleased {
                max_time_until_release,
            } => {
                let Some(date) = release_date else {
                    return true;
                };

                let now = Self::date_now();

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

                let now = Self::date_now();

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

                let now = Self::date_now();

                ReleaseStatus::check_max_time_until_release(&now, date, max_time_until_release)
                    && ReleaseStatus::check_max_age(&now, date, max_age)
            }
            Self::NoReleaseDate => release_date.is_none(),
            Self::All => true,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Error)]
pub(super) enum ReleaseStatusError {
    MaxAgeSmaller,
}

impl ReleaseStatusError {
    pub(super) fn text(&self) -> &'static str {
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

pub(super) fn deserialize_release_status<'de, D>(deserializer: D) -> Result<ReleaseStatus, D::Error>
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Days, Months};

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
}
