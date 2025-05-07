use crate::Tmdb;
use crate::endpoints::request;
use crate::models::genre_id::GenreId;
use crate::models::genres::{MovieGenre, TvGenre};
use chrono::NaiveDate;
use reqwest::Method;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct CombinedCredits {
    #[serde(default)]
    pub id: u64,
    pub cast: Option<Vec<Cast>>,
    pub crew: Option<Vec<Crew>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "media_type")]
pub enum Cast {
    #[serde(rename = "movie")]
    Movie(MovieCast),
    #[serde(rename = "tv")]
    Tv(TvCast),
}

#[derive(Debug, Deserialize)]
pub struct MovieCast {
    pub id: usize,
    pub title: String,
    pub original_title: String,
    pub character: String,
    #[serde(deserialize_with = "deserialize_movie_genre", flatten)]
    pub genres: Vec<MovieGenre>,
    #[serde(deserialize_with = "deserialize_release_date")]
    pub release_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
}

#[derive(Debug, Deserialize)]
pub struct TvCast {
    pub id: usize,
    pub name: String,
    pub original_name: String,
    pub character: String,
    #[serde(deserialize_with = "deserialize_tv_genre", flatten)]
    pub genres: Vec<TvGenre>,
    #[serde(deserialize_with = "deserialize_release_date")]
    pub first_air_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "media_type")]
pub enum Crew {
    #[serde(rename = "movie")]
    Movie(MovieCrew),

    #[serde(rename = "tv")]
    Tv(TvCrew),
}

#[derive(Debug, Deserialize)]
pub struct MovieCrew {
    pub id: usize,
    pub title: String,
    pub original_title: String,
    pub department: String,
    pub job: String,
    #[serde(deserialize_with = "deserialize_movie_genre", flatten)]
    pub genres: Vec<MovieGenre>,
    #[serde(deserialize_with = "deserialize_release_date")]
    pub release_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
}

#[derive(Debug, Deserialize)]
pub struct TvCrew {
    pub id: usize,
    pub name: String,
    pub original_name: String,
    pub department: String,
    pub job: String,
    #[serde(deserialize_with = "deserialize_tv_genre", flatten)]
    pub genres: Vec<TvGenre>,
    #[serde(deserialize_with = "deserialize_release_date")]
    pub first_air_date: Option<NaiveDate>,
    pub overview: String,
    pub original_language: String,
}

fn deserialize_movie_genre<'de, D>(deserializer: D) -> Result<Vec<MovieGenre>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Data {
        genre_ids: Vec<GenreId>,
    }

    let Data { genre_ids } = Data::deserialize(deserializer)?;

    Ok(genre_ids.into_iter().map(MovieGenre::from).collect())
}

fn deserialize_tv_genre<'de, D>(deserializer: D) -> Result<Vec<TvGenre>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Data {
        genre_ids: Vec<GenreId>,
    }

    let Data { genre_ids } = Data::deserialize(deserializer)?;

    Ok(genre_ids.into_iter().map(TvGenre::from).collect())
}

fn deserialize_release_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let date: &str = Deserialize::deserialize(deserializer)?;

    if date.is_empty() {
        return Ok(None);
    }

    Ok(NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
}

/// [GET: Combined Credits](https://developer.themoviedb.org/v3/reference/person-combined-credits)
///
/// Performs a get request on the `person/{person_id}/combined_credits` endpoint.
///
/// ## NOTE
/// The CombinedCredits struct is not an exhaustive representation of the data provided by
/// the api.
pub async fn get(tmdb: &Tmdb, person_id: &str) -> Result<CombinedCredits, reqwest::Error> {
    let path = format!("person/{person_id}/combined_credits");

    let response = request(tmdb, path, Method::GET).await?;

    response.json::<CombinedCredits>().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    fn init() -> Tmdb {
        Tmdb::new(Client::new(), "NO_TOKEN_REQUIRED".into())
    }

    #[tokio::test]
    async fn test_get_19498_cast() {
        const PERSON_ID: &str = "19498";

        let tmdb = init();

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id.to_string(), PERSON_ID);

        let cast = response.cast.unwrap();
        assert_eq!(cast.len(), 75);

        let movie = match &cast[0] {
            Cast::Movie(cast) => cast,
            Cast::Tv(_) => {
                panic!("first cast entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 1852);
        assert_eq!(movie.title, "World Trade Center");
        assert_eq!(movie.original_title, "World Trade Center");
        assert_eq!(movie.character, "Christopher Amoroso");
        assert_eq!(movie.genres, [MovieGenre::Drama, MovieGenre::History, MovieGenre::Thriller]);
        assert_eq!(movie.release_date, Some(NaiveDate::parse_from_str("2006-08-09", "%Y-%m-%d").unwrap()));
        assert_eq!(movie.overview, "Two police officers struggle to survive when they become trapped beneath the rubble of the World Trade Center on September 11, 2001.");
        assert_eq!(movie.original_language, "en");

        let tv = match &cast[48] {
            Cast::Tv(cast) => cast,
            Cast::Movie(_) => {
                panic!("first cast entry should be a tv show, was a movie");
            }
        };

        assert_eq!(tv.id, 1100);
        assert_eq!(tv.name, "How I Met Your Mother");
        assert_eq!(tv.original_name, "How I Met Your Mother");
        assert_eq!(tv.character, "Carlos");
        assert_eq!(tv.genres, [TvGenre::Comedy]);
        assert_eq!(tv.first_air_date, Some(NaiveDate::parse_from_str("2005-09-19", "%Y-%m-%d").unwrap()));
        assert_eq!(tv.overview, "A father recounts to his children - through a series of flashbacks - the journey he and his four best friends took leading up to him meeting their mother.");
        assert_eq!(tv.original_language, "en");
    }

    #[tokio::test]
    async fn test_get_956_crew() {
        const PERSON_ID: &str = "956";

        let tmdb = init();

        let response = get(&tmdb, PERSON_ID).await.unwrap();
        assert_eq!(response.id.to_string(), PERSON_ID);

        let crew = response.crew.unwrap();
        assert_eq!(crew.len(), 66);

        let movie = match &crew[7] {
            Crew::Movie(crew) => crew,
            Crew::Tv(_) => {
                panic!("first crew entry should be a movie, was a tv show");
            }
        };

        assert_eq!(movie.id, 10528);
        assert_eq!(movie.title, "Sherlock Holmes");
        assert_eq!(movie.original_title, "Sherlock Holmes");
        assert_eq!(movie.department, "Directing");
        assert_eq!(movie.job, "Director");
        assert_eq!(movie.genres, [MovieGenre::Action, MovieGenre::Adventure, MovieGenre::Crime, MovieGenre::Mystery]);
        assert_eq!(movie.release_date, Some(NaiveDate::parse_from_str("2009-12-23", "%Y-%m-%d").unwrap()));
        assert_eq!(movie.overview, "Eccentric consulting detective Sherlock Holmes and Doctor John Watson battle to bring down a new nemesis and unravel a deadly plot that could destroy England.");
        assert_eq!(movie.original_language, "en");

        let tv = match &crew[59] {
            Crew::Tv(crew) => crew,
            Crew::Movie(_) => {
                panic!("first crew entry should be a tv show, was a movie");
            }
        };

        assert_eq!(tv.id, 236235);
        assert_eq!(tv.name, "The Gentlemen");
        assert_eq!(tv.original_name, "The Gentlemen");
        assert_eq!(movie.department, "Directing");
        assert_eq!(movie.job, "Director");
        assert_eq!(tv.genres, [TvGenre::Comedy, TvGenre::Drama, TvGenre::Crime]);
        assert_eq!(tv.first_air_date, Some(NaiveDate::parse_from_str("2024-03-07", "%Y-%m-%d").unwrap()));
        assert_eq!(tv.overview, "When aristocratic Eddie inherits the family estate, he discovers that it's home to an enormous weed empire — and its proprietors aren't going anywhere.");
        assert_eq!(tv.original_language, "en");
    }
}
