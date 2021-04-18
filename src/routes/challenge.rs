use rocket::{get,post};
use rocket::http::RawStr;
use rocket_contrib::json;
use rocket_contrib::json::{Json,JsonValue};

use crate::database::DbConn;
use crate::database::models::ChallengeResponse;
use crate::database::daos::challenge;

use regex::Regex;

use crate::routes::errors::ChallengeError;

#[get("/challenges/<p_locale>/random")]
pub fn get_random_challenge(db: DbConn, p_locale: &RawStr) -> Result<JsonValue,ChallengeError> {
    lazy_static! {
        static ref LOCALE_REGEX: Regex = Regex::new(r"\w{2}-\w{2}").unwrap();
    }

    if !LOCALE_REGEX.is_match(p_locale) {
        return Err(ChallengeError::Parse(String::from("No valid locale passed.")));
    }

    let challenge = challenge::get_random_challenge(db, &p_locale.to_string())?;

    Ok(json!({
        "token_challenge_id": challenge.token_challenge_id,
        "question": challenge.question,
        "url": challenge.url,
    }))
}

#[post("/challenges/test", data = "<challenge_response>")]
pub fn test_challenge(db: DbConn, challenge_response: Json<ChallengeResponse>) -> Result<JsonValue,ChallengeError> {

    let token = challenge::test_challenge(db, &challenge_response)?;

    Ok(json!({
        "token": token.uuid
    }))
}

#[cfg(test)]
mod test {
    use mocktopus::mocking::Mockable;
    use mocktopus::mocking::MockResult;

    use chrono::NaiveDate;
    use uuid::Uuid;

    use rocket;
    use rocket::routes;
    use rocket::local::Client;
    use rocket::http::Status;

    use crate::database::DbConn;
    use crate::database::daos::challenge;
    use crate::database::models::{Token,TokenChallengeTranslatable};

    #[test]
    fn test_get_random_challenge() {
        crate::database::test::setup();

        challenge::get_random_challenge.mock_safe(|_, locale| {
            assert_eq!(locale, "de-DE");
            MockResult::Return(Ok(TokenChallengeTranslatable {
                id: 1,
                token_challenge_id: 1,
                locale: "de-DE".to_string(),
                question: "The question".to_string(),
                answer_hash: "cryptic hash here".to_string(),
                url: None,
            }))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_random_challenge]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/challenges/de-DE/random").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("{\"question\":\"The question\",\"token_challenge_id\":1,\"url\":null}".to_string()));
    }

    #[test]
    fn test_test_challenge() {
        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";
        let uuid = Uuid::parse_str(uuid).unwrap();

        challenge::test_challenge.mock_safe(move |_, _| {
            MockResult::Return(Ok(Token {
                id: 1,
                uuid: uuid,
                created_at: NaiveDate::from_ymd(2021, 4, 18).and_hms(0, 0, 0),
            }))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::test_challenge]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.post("/challenges/test")
            .body("{\"answer_hash\":\"cryptic hash here\",\"token_challenge_id\":1}")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(format!("{{\"token\":\"{}\"}}", "00a791f1-68b8-457c-82d9-a060f48efbae")));
    }
}