use rocket::{get,post};
use rocket::http::RawStr;
use rocket_contrib::json;
use rocket_contrib::json::{Json,JsonValue};

use chrono::prelude::DateTime;

use crate::database::DbConn;
use crate::database::models::*;
use crate::database::daos::rent;
use crate::mailer;

use crate::routes::errors::RentError;

#[get("/rents?<as_of>")]
pub fn get_rents(db: DbConn, as_of: Option<String>) -> Result<Json<Vec<Rent>>,RentError> {
    let as_of = as_of.unwrap_or("1970-01-01T00:00:00.000Z".to_string());
    let as_of = DateTime::parse_from_rfc3339(&as_of)?;
    let as_of = as_of.naive_utc();

    let data = rent::get_rents(&db, &as_of)?;

    Ok(Json(data))
}

#[post("/rents", data = "<booking>")]
pub fn book(db: DbConn, booking: Json<Booking>) -> Result<JsonValue,RentError> {
    let booking = &*booking;

    let result = rent::insert_booking(&db, booking);

    if result.is_ok() {
        if booking.email.is_some() {
            mailer::send_rent_mail(booking)?;
        }

        Ok(json!({
            "token": booking.token
        }))
    } else {
        Err(result.err().unwrap())
    }
}

#[post("/rents/<token>/revoke")]
pub fn revoke_booking(db: DbConn, token: &RawStr) -> Result<(),RentError> {
    let parsed_token = ::uuid::Uuid::parse_str(token)?;

    rent::revoke_booking(&db, &parsed_token)
}

#[cfg(test)]
mod test {
    use mocktopus::mocking::Mockable;
    use mocktopus::mocking::MockResult;

    use chrono::prelude::DateTime;
    use lettre::transport::smtp::response::{Category,Code,Detail,Response,Severity};

    use rocket;
    use rocket::routes;
    use rocket::local::Client;
    use rocket::http::Status;

    use crate::database::DbConn;
    use crate::database::daos::rent;

    use crate::mailer;

    use crate::routes::errors::RentError;

    #[test]
    fn test_get_rents_without_timestamp() {
        crate::database::test::setup();

        rent::get_rents.mock_safe(|_, as_of| {
            let expected_as_of = DateTime::parse_from_rfc3339(&"1970-01-01T00:00:00.000Z".to_string()).unwrap();
            let expected_as_of = expected_as_of.naive_utc();

            assert_eq!(*as_of, expected_as_of);
            MockResult::Return(Ok(vec![]))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_rents]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/rents").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("[]".to_string()));
    }

    #[test]
    fn test_get_rents_with_timestamp() {
        crate::database::test::setup();

        rent::get_rents.mock_safe(|_, as_of| {
            let expected_as_of = DateTime::parse_from_rfc3339(&"2021-04-21T00:00:00.000Z".to_string()).unwrap();
            let expected_as_of = expected_as_of.naive_utc();

            assert_eq!(*as_of, expected_as_of);
            MockResult::Return(Ok(vec![]))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_rents]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/rents?as_of=2021-04-21T00:00:00.000Z").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("[]".to_string()));
    }

    #[test]
    fn test_book_with_successful_database_insert_without_email() {
        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        rent::insert_booking.mock_safe(|_, _| {
            MockResult::Return(Ok(()))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::book]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.post("/rents")
            .body(format!(r#"{{"token":"{}","bike_id": 1,"start_timestamp": "2021-04-19T00:00:00.000","end_timestamp": "2021-04-19T00:00:00.000","encrypted_details": "","short_token": "","email": null}}"#, uuid))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(format!("{{\"token\":\"{}\"}}", uuid).to_string()));
    }

    #[test]
    fn test_book_with_successful_database_insert_with_email() {
        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        rent::insert_booking.mock_safe(|_, _| {
            MockResult::Return(Ok(()))
        });

        mailer::send_rent_mail.mock_safe(|b| {
            assert_eq!(b.email.as_ref().unwrap(), "someone@somewhere.near");
            MockResult::Return(Ok(Response {
                code: Code {
                    category: Category::Information,
                    detail: Detail::Zero,
                    severity: Severity::PositiveCompletion,
                },
                message: vec![],
            }))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::book]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.post("/rents")
            .body(format!(r#"{{"token":"{}","bike_id": 1,"start_timestamp": "2021-04-19T00:00:00.000","end_timestamp": "2021-04-19T00:00:00.000","encrypted_details": "","short_token": "","email": "someone@somewhere.near"}}"#, uuid))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(format!("{{\"token\":\"{}\"}}", uuid).to_string()));
    }

    #[test]
    fn test_book_with_failed_database_insert() {
        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        rent::insert_booking.mock_safe(|_, _| {
            MockResult::Return(Err(RentError::Validation("Bätsch".to_string())))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::book]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.post("/rents")
            .body(format!(r#"{{"token":"{}","bike_id": 1,"start_timestamp": "2021-04-19T00:00:00.000","end_timestamp": "2021-04-19T00:00:00.000","encrypted_details": "","short_token": "","email": null}}"#, uuid))
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.body_string(), Some("Bätsch".to_string()));
    }

    #[test]
    fn test_revoke_booking() {
        crate::database::test::setup();

        let uuid = "00a791f1-68b8-457c-82d9-a060f48efbae";

        rent::revoke_booking.mock_safe(move |_, token| {
            assert_eq!(uuid, token.to_string());
            MockResult::Return(Ok(()))
        });

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::revoke_booking]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let response = client.post(format!("/rents/{}/revoke", uuid)).dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}