use mockall_double::double;

use std::sync::Arc;

use rocket::{get,post};
use rocket::serde::json::{Json,Value,json};

use crate::database::DbConn;
use crate::database::models::ChallengeResponse;

#[double]
use crate::database::daos::challenge;

use regex::Regex;

use crate::routes::errors::ChallengeError;

#[get("/challenges/<p_locale>/random")]
pub async fn get_random_challenge(db: DbConn, p_locale: &str) -> Result<Value,ChallengeError> {
    lazy_static! {
        static ref LOCALE_REGEX: Regex = Regex::new(r"\w{2}-\w{2}").unwrap();
    }

    if !LOCALE_REGEX.is_match(p_locale) {
        return Err(ChallengeError::Parse(String::from("No valid locale passed.")));
    }

    let p_locale = Arc::new(p_locale.to_string());

    let challenge = challenge::get_random_challenge(db, p_locale).await?;

    Ok(json!({
        "token_challenge_id": challenge.token_challenge_id,
        "question": challenge.question,
        "url": challenge.url,
    }))
}

#[post("/challenges/test", data = "<challenge_response>")]
pub async fn test_challenge(db: DbConn, challenge_response: Json<ChallengeResponse>) -> Result<Value,ChallengeError> {
    let challenge_response = Arc::new((*challenge_response).clone());

    let result = challenge::test_challenge(db, challenge_response).await;

    match result {
        Ok(token) =>
            Ok(json!({
                "token": token.uuid
            })),
        Err(error) =>  {
            Err(error)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    use rocket;
    use rocket::routes;
    use rocket::http::Status;

    use rocket::local::blocking::Client;

    use crate::database::DbConn;
    use crate::database::models::{Token,TokenChallengeTranslatable};

    use lazy_static::lazy_static;
    use std::sync::{Mutex, MutexGuard};

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    // When a test panics, it will poison the Mutex. Since we don't actually
    // care about the state of the data we ignore that it is poisoned and grab
    // the lock regardless. If you just do `let _m = &MTX.lock().unwrap()`, one
    // test panicking will cause all other tests that try and acquire a lock on
    // that Mutex to also panic.
    fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()> {
        match m.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[test]
    fn test_get_random_challenge() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let random_challenge_ctx = challenge::get_random_challenge_context();

        random_challenge_ctx.expect()
            .returning(|_, p_locale| {
                assert_eq!(*p_locale, "de-DE");

                Ok(TokenChallengeTranslatable {
                    id: 1,
                    token_challenge_id: 1,
                    locale: "de-DE".to_string(),
                    question: "The question".to_string(),
                    answer_hash: "cryptic hash here".to_string(),
                    url: None,
                })
            });

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_random_challenge]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/challenges/de-DE/random").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("{\"question\":\"The question\",\"token_challenge_id\":1,\"url\":null}".to_string()));
    }

    #[test]
    fn test_test_challenge() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let test_challenge_ctx = challenge::test_challenge_context();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";
        let uuid = Uuid::parse_str(uuid).unwrap();
        let date = NaiveDate::from_ymd_opt(2021, 4, 18)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap();

        test_challenge_ctx.expect()
            .returning(move |_, _| {
                Ok(Token {
                    id: 1,
                    uuid: uuid,
                    created_at: date,
                })
            });

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::test_challenge]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.post("/challenges/test")
            .body("{\"answer_hash\":\"cryptic hash here\",\"token_challenge_id\":1}")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some(format!("{{\"token\":\"{}\"}}", "00a791f1-68b8-457c-82d9-a060f48efbae")));
    }
}