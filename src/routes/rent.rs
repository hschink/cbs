use rocket::{get,post};
use rocket::http::RawStr;
use rocket_contrib::json;
use rocket_contrib::json::{Json,JsonValue};

use chrono::prelude::{DateTime,Utc};

use diesel::{Connection,RunQueryDsl,QueryDsl,BoolExpressionMethods,ExpressionMethods};
use diesel::{insert_into,update};

use crate::database::DbConn;
use crate::database::models::*;
use crate::schema::rents::dsl::*;
use crate::schema::rent_details::dsl::*;
use crate::schema::tokens::dsl::*;
use crate::mailer;

use crate::routes::errors::RentError;

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

        mailer::send_rent_mail(booking)?;

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