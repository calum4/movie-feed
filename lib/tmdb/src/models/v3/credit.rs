use crate::SITE_URL;
use crate::models::v3::cast::Cast;
use crate::models::v3::crew::Crew;
use crate::models::v3::genres::Genre;
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use url::Url;

pub trait IsCredit {
    const MEDIA_TYPE: MediaType;

    // Common Fields
    fn id(&self) -> usize;
    fn title(&self) -> &str;
    fn original_title(&self) -> &str;
    fn genres(&self) -> &[impl Genre];
    fn release_date(&self) -> Option<&NaiveDate>;
    fn original_language(&self) -> &str;
    fn overview(&self) -> Option<&String>;
    fn credit_id(&self) -> &str;

    #[inline]
    fn media_type(&self) -> MediaType {
        Self::MEDIA_TYPE
    }

    fn tmdb_media_url(&self) -> Url {
        let media_url_prefix = Self::MEDIA_TYPE.tmdb_url_prefix().expect(
            "Self::MEDIA_TYPE is const and is guaranteed by tests to always return Some(_)",
        );

        SITE_URL
            .join(format!("{media_url_prefix}/{}", self.id()).as_str())
            .expect("url guaranteed to be valid")
    }
}
