#[macro_use]
extern crate rocket;
extern crate diesel;
extern crate dotenv;

use serial_test::serial;

use dotenv::dotenv;
use std::env;

use chrono::prelude::*;

use serde_json::json;

use diesel::{PgConnection, RunQueryDsl, QueryDsl, Connection};
use diesel::{delete, insert_into};

use rocket::fairing::AdHoc;
use rocket::local::blocking::Client;

use rocket::http::Status;

use cargobike_share_backend::database::DbConn;
use cargobike_share_backend::routes::rent;
use cargobike_share_backend::database::models::{Rent, Bike, Token, InsertRent, Booking};
use cargobike_share_backend::schema::bikes::dsl::*;
use cargobike_share_backend::schema::tokens::dsl::*;
use cargobike_share_backend::schema::rent_details::dsl::*;
use cargobike_share_backend::schema::rents::dsl::*;

fn get_database_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url.to_string()))
}

fn setup_rents(connection: &mut PgConnection) -> Vec<Token> {
    let _ = delete(rent_details).execute(connection);
    let _ = delete(rents).execute(connection);
    let _ = delete(tokens).execute(connection);
    let _ = delete(bikes).execute(connection);

    let bike1 = insert_into(bikes).default_values().get_result::<Bike>(connection).unwrap();
    let token1 = insert_into(tokens).default_values().get_result::<Token>(connection).unwrap();
    let token2 = insert_into(tokens).default_values().get_result::<Token>(connection).unwrap();

    let rent1 = InsertRent {
        token_id: token1.id,
        bike_id: bike1.id,
        start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-01T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-02T00:00:00.000Z".to_string()).unwrap().naive_utc(),
    };
    let rent2 = InsertRent {
        token_id: token2.id,
        bike_id: bike1.id,
        start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-03T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-04T00:00:00.000Z".to_string()).unwrap().naive_utc(),
    };

    insert_into(rents).values(rent1).execute(connection).unwrap();
    insert_into(rents).values(rent2).execute(connection).unwrap();

    vec![token1, token2]
}

fn setup_rocket_client() -> Client {
    let ignite = AdHoc::on_ignite("test_test_challenge_succeeds", |rocket| async {
        rocket.attach(DbConn::fairing())
            .mount("/", routes![
                rent::get_rents,
                rent::book,
                rent::revoke_booking
            ])
    });

    Client::tracked(rocket::build().attach(ignite)).unwrap()
}

#[test]
#[serial(bike)]
pub fn test_get_rents_with_epoch() {
    let mut connection = get_database_connection();
    setup_rents(&mut connection);

    let client = setup_rocket_client();

    let expected: usize = 2;
    let actual: usize = client.get("/rents?as_of=1970-01-01T00:00:00.000Z")
        .dispatch()
        .into_json::<Vec<Rent>>()
        .unwrap()
        .len();

    assert_eq!(actual, expected);
}

#[test]
#[serial(bike)]
pub fn test_get_rents_with_timestamp_after_first_rent() {
    let mut connection = get_database_connection();
    setup_rents(&mut connection);

    let client = setup_rocket_client();

    let expected: usize = 1;
    let actual: usize = client.get("/rents?as_of=1970-01-03T00:00:00.000Z")
        .dispatch()
        .into_json::<Vec<Rent>>()
        .unwrap()
        .len();

    assert_eq!(actual, expected);
}

#[test]
#[serial(bike)]
pub fn test_insert_booking_succeeds() {
    let mut connection = get_database_connection();
    setup_rents(&mut connection);

    let bike1 = bikes.order_by(cargobike_share_backend::schema::bikes::dsl::id)
        .limit(1)
        .get_result::<Bike>(&mut connection)
        .unwrap();
    let token3 = insert_into(tokens)
        .default_values()
        .get_result::<Token>(&mut connection)
        .unwrap();

    let booking = Booking {
        token: token3.uuid,
        bike_id: bike1.id,
        start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-05T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-06T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        encrypted_details: "".to_string(),
        short_token: "".to_string(),
        email: None,
    };

    let client = setup_rocket_client();

    let response = client.post("/rents")
        .body(json!(booking).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
#[serial(bike)]
pub fn test_insert_booking_fails() {
    let mut connection = get_database_connection();
    setup_rents(&mut connection);

    let bike1 = bikes.order_by(cargobike_share_backend::schema::bikes::dsl::id)
        .limit(1)
        .get_result::<Bike>(&mut connection)
        .unwrap();
    let token3 = insert_into(tokens)
        .default_values()
        .get_result::<Token>(&mut connection)
        .unwrap();

    let booking = Booking {
        token: token3.uuid,
        bike_id: bike1.id,
        start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-01T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-06T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        encrypted_details: "".to_string(),
        short_token: "".to_string(),
        email: None,
    };

    let client = setup_rocket_client();

    let response = client.post("/rents")
        .body(json!(booking).to_string())
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}

#[test]
#[serial(bike)]
pub fn test_revoke_booking_succeeds() {
    let mut connection = get_database_connection();
    let test_tokens = setup_rents(&mut connection);

    let client = setup_rocket_client();

    env::set_var("SENT_RENT_MAIL", "false");

    let response = client.post(format!("/rents/{}/revoke", test_tokens[0].uuid.to_string()))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
#[serial(bike)]
pub fn test_revoke_booking_fails() {
    
    let mut connection = get_database_connection();
    let test_tokens = setup_rents(&mut connection);

    let client = setup_rocket_client();

    env::set_var("SENT_RENT_MAIL", "false");

    let response = client.post(format!("/rents/{}/revoke", test_tokens[0].uuid.to_string()))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    let response = client.post(format!("/rents/{}/revoke", test_tokens[0].uuid.to_string()))
        .dispatch();

    assert_eq!(response.status(), Status::BadRequest);
}