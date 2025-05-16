use std::fmt::{Debug, Display, Formatter};
use tmdb_macros::make_genre;

make_genre!(
    MovieGenre,
    r#"{
      "genres": [
        {
          "id": 28,
          "name": "Action"
        },
        {
          "id": 12,
          "name": "Adventure"
        },
        {
          "id": 16,
          "name": "Animation"
        },
        {
          "id": 35,
          "name": "Comedy"
        },
        {
          "id": 80,
          "name": "Crime"
        },
        {
          "id": 99,
          "name": "Documentary"
        },
        {
          "id": 18,
          "name": "Drama"
        },
        {
          "id": 10751,
          "name": "Family"
        },
        {
          "id": 14,
          "name": "Fantasy"
        },
        {
          "id": 36,
          "name": "History"
        },
        {
          "id": 27,
          "name": "Horror"
        },
        {
          "id": 10402,
          "name": "Music"
        },
        {
          "id": 9648,
          "name": "Mystery"
        },
        {
          "id": 10749,
          "name": "Romance"
        },
        {
          "id": 878,
          "name": "Science Fiction"
        },
        {
          "id": 10770,
          "name": "TV Movie"
        },
        {
          "id": 53,
          "name": "Thriller"
        },
        {
          "id": 10752,
          "name": "War"
        },
        {
          "id": 37,
          "name": "Western"
        }
      ]
    }"#
);

make_genre!(
    TvGenre,
    r#"{
      "genres": [
        {
          "id": 10759,
          "name": "Action & Adventure"
        },
        {
          "id": 16,
          "name": "Animation"
        },
        {
          "id": 35,
          "name": "Comedy"
        },
        {
          "id": 80,
          "name": "Crime"
        },
        {
          "id": 99,
          "name": "Documentary"
        },
        {
          "id": 18,
          "name": "Drama"
        },
        {
          "id": 10751,
          "name": "Family"
        },
        {
          "id": 10762,
          "name": "Kids"
        },
        {
          "id": 9648,
          "name": "Mystery"
        },
        {
          "id": 10763,
          "name": "News"
        },
        {
          "id": 10764,
          "name": "Reality"
        },
        {
          "id": 10765,
          "name": "Sci-Fi & Fantasy"
        },
        {
          "id": 10766,
          "name": "Soap"
        },
        {
          "id": 10767,
          "name": "Talk"
        },
        {
          "id": 10768,
          "name": "War & Politics"
        },
        {
          "id": 37,
          "name": "Western"
        }
      ]
    }"#
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::v3::genre_id::GenreId;

    #[test]
    fn test_movie_genre_id() {
        let genre = MovieGenre::from(GenreId::from(28));

        assert_eq!(genre.id(), 28.into());
    }

    #[test]
    fn test_movie_genre_name() {
        let genre = MovieGenre::from(GenreId::from(10751));

        assert_eq!(genre.name(), "Family");
    }

    #[test]
    fn test_tv_genre() {
        let genre = TvGenre::SciFiAndFantasy;

        assert_eq!(genre.id(), 10765.into());
        assert_eq!(genre.name(), "Sci-Fi & Fantasy");
    }
}
