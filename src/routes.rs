pub mod error;

use chrono::prelude::{DateTime,Utc};

use rocket::{get,post};
use rocket::http::RawStr;
use rocket_contrib::json;
use rocket_contrib::json::{Json,JsonValue};

use diesel::{Connection,RunQueryDsl,QueryDsl,BoolExpressionMethods,ExpressionMethods};
use diesel::{insert_into,update};

use regex::Regex;

use crate::database::DbConn;
use crate::database::models::*;
use crate::schema::bike_translatables::dsl::*;
use crate::schema::rents::dsl::*;
use crate::schema::rent_details::dsl::*;
use crate::schema::tokens::dsl::*;
use crate::schema::token_challenge_translatables::dsl::*;

use crate::routes::error::{RentError,ChallengeError};

#[get("/")]
pub fn index() -> JsonValue {
    json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION")
    })
}

#[get("/bikes")]
pub fn get_bikes(db: DbConn) -> Result<Json<Vec<BikeTranslatable>>,diesel::result::Error> {
    let data = bike_translatables
        .load::<BikeTranslatable>(&*db);

    match data {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(e),
    }
}

#[get("/rents?<as_of>")]
pub fn get_rents(db: DbConn, as_of: Option<String>) -> Result<Json<Vec<Rent>>,RentError> {
    let as_of = as_of.unwrap_or("1970-01-01T00:00:00.000Z".to_string());
    let as_of = DateTime::parse_from_rfc3339(&as_of)?;
    let as_of = as_of.naive_utc();

    let data = rents
        .filter(end_timestamp.ge(&as_of))
        .filter(revocation_timestamp.is_null())
        .get_results::<Rent>(&*db)?;

    Ok(Json(data))
}

#[post("/rents", data = "<booking>")]
pub fn book(db: DbConn, booking: Json<Booking>) -> Result<JsonValue,RentError> {
    let booking = &*booking;
    // TODO: Turing test

    (&*db).transaction(|| {
        let overlapping_rent_count = rents
                .filter(revocation_timestamp.is_null()
                    .and(start_timestamp.between(booking.start_timestamp, booking.end_timestamp)
                        .or(start_timestamp.between(booking.start_timestamp, booking.end_timestamp))
                    )
                )
                .count()
                .get_result::<i64>(&*db)?;

        if overlapping_rent_count > 0 {
            return Err(RentError::Validation(String::from("There is already a rent at the same period.")));
        }

        let token = tokens
            .filter(uuid.eq(booking.token))
            .get_result::<Token>(&*db)?;

        let rent = InsertRent {
            token_id: token.id,
            bike_id: booking.bike_id,
            start_timestamp: booking.start_timestamp,
            end_timestamp: booking.end_timestamp,
        };

        let inserted_rent = insert_into(rents)
            .values(&rent)
            .get_result::<Rent>(&*db)?;

        let rent_detail = InsertRentDetail {
            rent_id: inserted_rent.id,
            encrypted_details: booking.encrypted_details.clone(),
        };

        insert_into(rent_details)
            .values(&rent_detail)
            .execute(&*db)?;

        Ok(json!({
            "token": booking.token
        }))
    })
}

#[post("/rents/<token>/revoke")]
pub fn revoke_booking(db: DbConn, token: &RawStr) -> Result<(),RentError> {
    let parsed_token = ::uuid::Uuid::parse_str(token)?;

    (&*db).transaction(|| {
        let token = tokens
            .filter(uuid.eq(&parsed_token))
            .get_result::<Token>(&*db)?;

        let booking = rents
            .filter(revocation_timestamp.is_null())
            .filter(token_id.eq(&token.id))
            .get_result::<Rent>(&*db)?;

        update(&booking)
            .set(revocation_timestamp.eq(Utc::now().naive_utc()))
            .execute(&*db)?;

        Ok(())
    })
}

no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");

#[get("/challenges/<p_locale>/random")]
pub fn get_random_challenge(db: DbConn, p_locale: &RawStr) -> Result<JsonValue,ChallengeError> {
    lazy_static! {
        static ref LOCALE_REGEX: Regex = Regex::new(r"\w{2}-\w{2}").unwrap();
    }

    if !LOCALE_REGEX.is_match(p_locale) {
        return Err(ChallengeError::Parse(String::from("No valid locale passed.")));
    }

    let challenge = token_challenge_translatables
        .filter(crate::schema::token_challenge_translatables::dsl::locale.eq(&p_locale.to_string()))
        .order(RANDOM)
        .limit(1)
        .get_result::<TokenChallengeTranslatable>(&*db)?;

    Ok(json!({
        "token_challenge_id": challenge.token_challenge_id,
        "question": challenge.question,
        "url": challenge.url,
    }))
}

#[post("/challenges/test", data = "<challenge_response>")]
pub fn test_challenge(db: DbConn, challenge_response: Json<ChallengeResponse>) -> Result<JsonValue,ChallengeError> {

    token_challenge_translatables
        .filter(token_challenge_id.eq(challenge_response.token_challenge_id).and(answer_hash.eq(&challenge_response.answer_hash)))
        .get_result::<TokenChallengeTranslatable>(&*db)?;

    let token = insert_into(tokens)
        .default_values()
        .get_result::<Token>(&*db)?;

    Ok(json!({
        "token": token.uuid
    }))
}