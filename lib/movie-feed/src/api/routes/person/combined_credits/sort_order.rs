use chrono::NaiveDate;
use serde::Deserialize;
use std::cmp::Ordering;

#[derive(Deserialize, Default, Debug, Copy, Clone, Eq, PartialEq)]
pub(super) enum SortReleaseDates {
    #[default]
    Descending,
    Ascending,
}

impl SortReleaseDates {
    pub(super) fn sort_release_date(
        &self,
        a: Option<&NaiveDate>,
        b: Option<&NaiveDate>,
    ) -> Ordering {
        match self {
            SortReleaseDates::Descending => match (a, b) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(a), Some(b)) => b.cmp(a),
            },
            SortReleaseDates::Ascending => match (a, b) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => a.cmp(b),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() -> Vec<Option<NaiveDate>> {
        vec![
            None,
            NaiveDate::from_ymd_opt(2025, 4, 10),
            None,
            NaiveDate::from_ymd_opt(2025, 5, 14),
            NaiveDate::from_ymd_opt(2026, 1, 1),
            None,
            NaiveDate::from_ymd_opt(1990, 6, 1),
        ]
    }

    #[test]
    fn sort_descending() {
        const SORT_ORDER: SortReleaseDates = SortReleaseDates::Descending;

        let mut unsorted_dates = init();

        let sorted_dates = vec![
            None,
            None,
            None,
            NaiveDate::from_ymd_opt(2026, 1, 1),
            NaiveDate::from_ymd_opt(2025, 5, 14),
            NaiveDate::from_ymd_opt(2025, 4, 10),
            NaiveDate::from_ymd_opt(1990, 6, 1),
        ];

        unsorted_dates.sort_by(|a, b| SORT_ORDER.sort_release_date(a.as_ref(), b.as_ref()));

        assert_eq!(unsorted_dates, sorted_dates);
    }

    #[test]
    fn sort_ascending() {
        const SORT_ORDER: SortReleaseDates = SortReleaseDates::Ascending;

        let mut unsorted_dates = init();

        let sorted_dates = vec![
            NaiveDate::from_ymd_opt(1990, 6, 1),
            NaiveDate::from_ymd_opt(2025, 4, 10),
            NaiveDate::from_ymd_opt(2025, 5, 14),
            NaiveDate::from_ymd_opt(2026, 1, 1),
            None,
            None,
            None,
        ];

        unsorted_dates.sort_by(|a, b| SORT_ORDER.sort_release_date(a.as_ref(), b.as_ref()));

        assert_eq!(unsorted_dates, sorted_dates);
    }
}
