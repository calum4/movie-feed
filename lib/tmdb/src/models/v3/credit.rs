use crate::SITE_URL;
use crate::models::v3::cast::Cast;
use crate::models::v3::crew::Crew;
use crate::models::v3::genres::Genre;
use crate::models::v3::media_type::MediaType;
use chrono::NaiveDate;
use tmdb_macros::IsCredit;
use url::Url;

#[derive(Debug, Hash, IsCredit)]
pub enum Credit {
    Cast(Cast),
    Crew(Crew),
}

impl From<Cast> for Credit {
    fn from(cast: Cast) -> Self {
        Self::Cast(cast)
    }
}

impl From<Crew> for Credit {
    fn from(crew: Crew) -> Self {
        Self::Crew(crew)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CreditType {
    Cast,
    Crew,
}

pub trait IsCredit {
    // Common Fields
    fn id(&self) -> usize;
    fn title(&self) -> &str;
    fn original_title(&self) -> &str;
    // TODO - Avoid collecting into vec?
    fn genres(&self) -> Vec<&dyn Genre>;
    fn release_date(&self) -> Option<&NaiveDate>;
    fn original_language(&self) -> &str;
    fn overview(&self) -> Option<&String>;
    fn credit_id(&self) -> &str;

    // Other
    fn media_type(&self) -> MediaType;
    fn credit_type(&self) -> CreditType;

    #[inline]
    fn tmdb_media_url(&self) -> Url {
        let media_url_prefix = self.media_type().tmdb_url_prefix().expect(
            "Self::MEDIA_TYPE is const and is guaranteed by tests to always return Some(_)",
        );

        SITE_URL
            .join(format!("{media_url_prefix}/{}", self.id()).as_str())
            .expect("url guaranteed to be valid")
    }

    #[inline]
    fn overview_len(&self) -> Option<usize> {
        self.overview().map(|overview| overview.len())
    }
}
