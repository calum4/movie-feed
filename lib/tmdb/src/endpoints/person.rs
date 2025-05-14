use crate::Tmdb;
use crate::endpoints::request;
use reqwest::Method;
use crate::models::person_details::PersonDetails;

pub mod combined_credits;

/// [GET: Person Details](https://developer.themoviedb.org/reference/person-details)
///
/// Performs a get request on the `person/{person_id}` endpoint.
pub async fn get(tmdb: &Tmdb, person_id: i32) -> Result<PersonDetails, reqwest::Error> {
    let path = format!("person/{person_id}");

    let response: reqwest::Response = request(tmdb, path, Method::GET).await?;

    response.json::<PersonDetails>().await
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use super::*;
    use reqwest::Client;
    use crate::models::gender::Gender;

    fn init() -> Tmdb {
        Tmdb::new(Client::new(), "NO_TOKEN_REQUIRED".into())
    }

    #[tokio::test]
    async fn test_get_19498() {
        const PERSON_ID: i32 = 19498;
        const BIOGRAPHY: &str = r#"Jonathan Edward Bernthal is an American actor. Beginning his career in the early 2000s, he came to prominence for portraying Shane Walsh on the AMC horror drama series The Walking Dead (2010–2012; 2018), where he was a starring cast member in the first two seasons. Bernthal achieved further recognition as Frank Castle / The Punisher in the Marvel Cinematic Universe, appearing in the second season of Daredevil (2016), the spin-off series The Punisher (2017–2019), and the first season of Daredevil: Born Again (2025). For his recurring guest role as drug addict Michael Berzatto in the series The Bear (2022–present), Bernthal won a Primetime Emmy Award.

His film roles include Snitch (2013), The Wolf of Wall Street (2013), Fury (2014), Sicario (2015), The Accountant (2016), Baby Driver (2017), Wind River (2017), Widows (2018), Ford v Ferrari (2019), Those Who Wish Me Dead (2021), King Richard (2021), The Many Saints of Newark (2021), Origin (2023), and The Accountant 2 (2025)."#;

        let tmdb = init();

        let response = get(&tmdb, PERSON_ID).await.unwrap();

        assert!(!response.adult);
        assert_eq!(
            response.also_known_as,
            vec![
                "乔·本恩瑟",
                "جان برانتال",
                "존 번탈",
                "Jonathan E. Bernthal",
                "جان برنتال",
            ]
        );
        assert_eq!(response.biography, Some(BIOGRAPHY.to_string()));
        assert_eq!(
            response.birthday,
            NaiveDate::parse_from_str("1976-09-20", "%Y-%m-%d").ok()
        );
        assert_eq!(response.deathday, None);
        assert_eq!(response.gender, Gender::Male);
        assert_eq!(response.homepage, None);
        assert_eq!(response.id, PERSON_ID);
        assert_eq!(response.imdb_id, Some("nm1256532".to_string()));
        assert_eq!(response.known_for_department, "Acting");
        assert_eq!(response.name, "Jon Bernthal");
        assert_eq!(
            response.place_of_birth,
            Some("Washington, D.C., USA".to_string())
        );
        assert_eq!(response.popularity, 10.3331);
        assert_eq!(
            response.profile_path,
            Some("/o0t6EVkJOrFAjESDilZUlf46IbQ.jpg".to_string())
        );
    }

    #[tokio::test]
    async fn test_get_956() {
        const PERSON_ID: i32 = 956;
        const BIOGRAPHY: &str = r#"Guy Stuart Ritchie (born 10 September 1968) is an English film director, producer and screenwriter. His work includes British gangster films and the Sherlock Holmes films starring Robert Downey Jr.

Ritchie left school at the age of 15. He worked in entry-level jobs in the film industry before directing television commercials. In 1995, he directed a short film, The Hard Case, followed by the crime comedy Lock, Stock and Two Smoking Barrels(1998), his feature-length directorial debut. He gained recognition with his second film, Snatch (2000), which was found to be critical and commercially successful. Following Snatch, Ritchie directed Swept Away (2002), a critically panned box-office bomb starring Madonna, to whom Ritchie was married between 2000 and 2008. He went on to direct Revolver (2005) and RocknRolla (2008), which were less successful and received mixed reviews. In 2009 and 2011, he directed the box-office hits Sherlock Holmes and its sequel, Sherlock Holmes: A Game of Shadows. The former was nominated for Academy Awards in Best Original Score and Best Art Direction.

His other directed films include The Man from U.N.C.L.E. (2015), based on the 1960s television series King Arthur: Legend of the Sword (2017), and Aladdin (2019), Disney's live-action adaptation of their 1992 animated film which grossed over $1 billion worldwide, becoming one of the highest-grossing films in 2019 and the highest-grossing film of Ritchie's career. In 2019, he returned to crime comedy with The Gentlemen (2019), which was mainly well-received and a commercial success. He subsequently reteamed with Jason Statham on the action films Wrath of Man (2021) and Operation Fortune: Ruse de Guerre (2023). His second film of 2023, The Covenant, received generally positive reviews.

Description above from the Wikipedia article Guy Ritchie, licensed under CC-BY-SA, full list of contributors on Wikipedia."#;

        let tmdb = init();

        let response = get(&tmdb, PERSON_ID).await.unwrap();

        assert!(!response.adult);
        assert_eq!(
            response.also_known_as,
            vec![
                "佳·烈治",
                "กาย ริตชี",
                "ガイ・リッチー",
                "가이 리치",
                "غاي ريتشي",
                "Γκάι Ρίτσι",
                "Guy Stuart Ritchie",
                "盖·里奇",
                "گای ریچی"
            ]
        );
        assert_eq!(response.biography, Some(BIOGRAPHY.to_string()));
        assert_eq!(
            response.birthday,
            NaiveDate::parse_from_str("1968-09-10", "%Y-%m-%d").ok()
        );
        assert_eq!(response.deathday, None);
        assert_eq!(response.gender, Gender::Male);
        assert_eq!(response.homepage, None);
        assert_eq!(response.id, PERSON_ID);
        assert_eq!(response.imdb_id, Some("nm0005363".to_string()));
        assert_eq!(response.known_for_department, "Directing");
        assert_eq!(response.name, "Guy Ritchie");
        assert_eq!(
            response.place_of_birth,
            Some("Hatfield, Hertfordshire, England, UK".to_string())
        );
        assert_eq!(response.popularity, 5.5305);
        assert_eq!(
            response.profile_path,
            Some("/9pLUnjMgIEWXi0mlHYzie9cKUTD.jpg".to_string())
        );
    }
}
