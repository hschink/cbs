use mockall_double::double;

use std::sync::Arc;

use std::env;

use rocket::{get,post};
use rocket::serde::json::{Json,Value,json};

use chrono::prelude::DateTime;

use crate::database::DbConn;
use crate::database::models::*;

#[double]
use crate::database::daos::rent;

#[double]
use crate::mailer;

use crate::routes::errors::RentError;

#[get("/rents?<as_of>")]
pub async fn get_rents(db: DbConn, as_of: Option<String>) -> Result<Json<Vec<Rent>>,RentError> {
    let as_of = as_of.unwrap_or("1970-01-01T00:00:00.000Z".to_string());
    let as_of = DateTime::parse_from_rfc3339(&as_of)?;
    let as_of = Arc::new(as_of.naive_utc());

    let data = rent::get_rents(db, as_of).await?;

    Ok(Json(data))
}

#[post("/rents", data = "<booking>")]
pub async fn book(db: DbConn, booking: Json<Booking>) -> Result<Value,RentError> {
    let booking_ref = Arc::new((*booking).clone());
    let booking = (*booking).clone();

    let result = rent::insert_booking(db, booking_ref).await;

    if result.is_ok() {
        let is_send_rent_mail_set = env::var("SENT_RENT_MAIL").unwrap_or(true.to_string()).parse::<bool>().unwrap();
        
        if is_send_rent_mail_set {
            mailer::send_rent_mail(&booking)?;
        }

        Ok(json!({
            "token": booking.token
        }))
    } else {
        Err(result.err().unwrap())
    }
}

#[post("/rents/<token>/revoke")]
pub async fn revoke_booking(db: DbConn, token: &str) -> Result<(),RentError> {
    let parsed_token = Arc::new(::uuid::Uuid::parse_str(token)?);

    rent::revoke_booking(db, parsed_token).await
}

#[cfg(test)]
mod test {
    use super::*;

    use lettre::transport::smtp::response::{Category, Code, Detail, Response, Severity};

    use rocket;
    use rocket::routes;
    use rocket::http::Status;

    use rocket::local::blocking::Client;

    use crate::routes::errors::RentError;

    use crate::database::DbConn;

    use lazy_static::lazy_static;
    use std::sync::{Mutex, MutexGuard};

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    // When a test panics, it will poison the Mutex. Since we don't actually
    // care about the state of the data we ignore that it is poisoned and grab
    // the lock regardless.  If you just do `let _m = &MTX.lock().unwrap()`, one
    // test panicking will cause all other tests that try and acquire a lock on
    // that Mutex to also panic.
    fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()> {
        match m.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[test]
    fn test_get_rents_without_timestamp() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let rents_ctx = rent::get_rents_context();

        rents_ctx.expect()
            .returning(|_, as_of| {
                let expected_as_of = DateTime::parse_from_rfc3339(&"1970-01-01T00:00:00.000Z".to_string()).unwrap();
                let expected_as_of = expected_as_of.naive_utc();

                assert_eq!(*as_of, expected_as_of);
                Ok(vec![])
            });

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_rents]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/rents").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("[]".to_string()));
    }

    #[test]
    fn test_get_rents_with_timestamp() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let rents_ctx = rent::get_rents_context();

        rents_ctx.expect()
            .returning(|_, as_of| {
                let expected_as_of = DateTime::parse_from_rfc3339(&"2021-04-21T00:00:00.000Z".to_string()).unwrap();
                let expected_as_of = expected_as_of.naive_utc();

                assert_eq!(*as_of, expected_as_of);
                Ok(vec![])
            });

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_rents]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/rents?as_of=2021-04-21T00:00:00.000Z").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("[]".to_string()));
    }

    #[test]
    fn test_book_with_successful_database_insert_without_email() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        let insert_booking_ctx = rent::insert_booking_context();
        let send_rent_mail_ctx = mailer::send_rent_mail_context();

        insert_booking_ctx.expect()
            .returning(|_, _| Ok(()));

        send_rent_mail_ctx.expect()
            .returning(|_|
                Ok(Response::new(
                    Code {
                        category: Category::Information,
                        detail: Detail::Zero,
                        severity: Severity::PositiveCompletion,
                    },
                    vec![]
                ))
            );

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::book]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.post("/rents")
            .body(format!(r#"{{"token":"{}","bike_id": 1,"start_timestamp": "2021-04-19T00:00:00.000","end_timestamp": "2021-04-19T00:00:00.000","encrypted_details": "","short_token": "","email": null}}"#, uuid))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some(format!("{{\"token\":\"{}\"}}", uuid).to_string()));
    }

    #[test]
    fn test_book_with_successful_database_insert_with_email() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        let insert_booking_ctx = rent::insert_booking_context();
        let send_rent_mail_ctx = mailer::send_rent_mail_context();

        insert_booking_ctx.expect()
            .returning(|_, _| Ok(()));

        send_rent_mail_ctx.expect()
            .returning(|b| {
                assert_eq!(b.email.as_ref().unwrap(), "someone@somewhere.near");

                Ok(Response::new(
                    Code {
                        category: Category::Information,
                        detail: Detail::Zero,
                        severity: Severity::PositiveCompletion,
                    },
                    vec![]
                ))
            });

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::book]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.post("/rents")
            .body(format!(r#"{{"token":"{}","bike_id": 1,"start_timestamp": "2021-04-19T00:00:00.000","end_timestamp": "2021-04-19T00:00:00.000","encrypted_details": "","short_token": "","email": "someone@somewhere.near"}}"#, uuid))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some(format!("{{\"token\":\"{}\"}}", uuid).to_string()));
    }

    #[test]
    fn test_book_with_failed_database_insert() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        let insert_booking_ctx = rent::insert_booking_context();

        insert_booking_ctx.expect()
            .returning(|_, _| Err(RentError::Validation("Bätsch".to_string())));

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::book]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.post("/rents")
            .body(format!(r#"{{"token":"{}","bike_id": 1,"start_timestamp": "2021-04-19T00:00:00.000","end_timestamp": "2021-04-19T00:00:00.000","encrypted_details": "","short_token": "","email": null}}"#, uuid))
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.into_string(), Some("Bätsch".to_string()));
    }

    #[test]
    fn test_revoke_booking() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        let revoke_booking_ctx = rent::revoke_booking_context();

        revoke_booking_ctx.expect()
            .returning(move |_, token| {
                assert_eq!(uuid, token.to_string());

                Ok(())
            });

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::revoke_booking]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.post(format!("/rents/{}/revoke", uuid)).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}