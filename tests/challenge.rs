#[macro_use]
extern crate rocket;

use serial_test::serial;

use rocket::fairing::AdHoc;
use rocket::local::blocking::Client;

use rocket::{Rocket, Build};
use rocket::http::Status;

use serde::{Deserialize};

use diesel::{RunQueryDsl, insert_into, delete};

use cargobike_share_backend::database::DbConn;
use cargobike_share_backend::routes::challenge;
use cargobike_share_backend::database::models::{TokenChallenge, InsertTokenChallengeTranslatable};
use cargobike_share_backend::schema::token_challenges::dsl::token_challenges;
use cargobike_share_backend::schema::token_challenge_translatables::dsl::token_challenge_translatables;

#[derive(Deserialize)]
struct TokenChallengeResponse {
    token_challenge_id: i32
}

#[test]
#[serial(challenge)]
pub fn test_test_challenge_succeeds() {

    let ignite = AdHoc::on_ignite("test_test_challenge_succeeds", |rocket| async {
        rocket.attach(DbConn::fairing())
            .attach(AdHoc::on_ignite("DB Setup", test_test_challenge_succeeds_setup))
            .mount("/", routes![
                challenge::test_challenge,
                challenge::get_random_challenge
            ])
    });
    let client = Client::tracked(rocket::build().attach(ignite)).unwrap();

    let token_challenge_translatable = client.get("/challenges/de-DE/random")
        .dispatch()
        .into_json::<TokenChallengeResponse>()
        .unwrap();

    let response = client.post("/challenges/test")
        .body(format!("{{ \"token_challenge_id\": {}, \"answer_hash\": \"123\" }}", token_challenge_translatable.token_challenge_id))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

async fn test_test_challenge_succeeds_setup(rocket: Rocket<Build>) -> Rocket<Build> {

    let conn = DbConn::get_one(&rocket).await.expect("database connection");

    conn.run( |c| {
        let _ = delete(token_challenge_translatables).execute(c);
        let _ = delete(token_challenges).execute(c);

        let answer_hash_value = "123".to_string();
        let token_challenge = insert_into(token_challenges).default_values().get_result::<TokenChallenge>(c).unwrap();

        let token_challenge_translatable = InsertTokenChallengeTranslatable {
            token_challenge_id: token_challenge.id,
            locale: "de-DE".to_string(),
            question: "".to_string(),
            answer_hash: answer_hash_value.to_string(),
            url: None,
        };

        insert_into(token_challenge_translatables)
            .values(token_challenge_translatable)
            .execute(c)
            .expect("Could not insert token challenge translatables.");
    }).await;

    rocket
}

#[test]
#[serial(challenge)]
pub fn test_test_challenge_fails() {

    let ignite = AdHoc::on_ignite("test_test_challenge_fails", |rocket| async {
        rocket.attach(DbConn::fairing())
            .attach(AdHoc::on_ignite("DB Setup", test_test_challenge_fails_setup))
            .mount("/", routes![
                challenge::test_challenge,
                challenge::get_random_challenge
            ])
    });
    let client = Client::tracked(rocket::build().attach(ignite)).unwrap();

    let token_challenge_translatable = client.get("/challenges/de-DE/random")
        .dispatch()
        .into_json::<TokenChallengeResponse>()
        .unwrap();
    let response = client.post("/challenges/test")
        .body(format!("{{ \"token_challenge_id\": {}, \"answer_hash\": \"123\" }}", token_challenge_translatable.token_challenge_id))
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

async fn test_test_challenge_fails_setup(rocket: Rocket<Build>) -> Rocket<Build> {

    let conn = DbConn::get_one(&rocket).await.expect("database connection");

    conn.run( |c| {
        let _ = delete(token_challenge_translatables).execute(c);
        let _ = delete(token_challenges).execute(c);

        let token_challenge = insert_into(token_challenges).default_values().get_result::<TokenChallenge>(c).unwrap();

        let token_challenge_translatable = InsertTokenChallengeTranslatable {
            token_challenge_id: token_challenge.id,
            locale: "de-DE".to_string(),
            question: "".to_string(),
            answer_hash: "wrong".to_string(),
            url: None,
        };

        insert_into(token_challenge_translatables)
            .values(token_challenge_translatable)
            .execute(c)
            .expect("Could not insert token challenge translatables.");
    }).await;

    rocket
}