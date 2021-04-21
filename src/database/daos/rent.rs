#[cfg(test)]
use mocktopus::macros::mockable;

use ::uuid::Uuid;
use chrono::{NaiveDateTime,Utc};

use diesel::{Connection,RunQueryDsl,QueryDsl,BoolExpressionMethods,ExpressionMethods};
use diesel::{insert_into,update};

use crate::database::DbConn;
use crate::database::models::{Booking,Rent,InsertRent,InsertRentDetail,Token};
use crate::schema::rents::dsl::*;
use crate::schema::rent_details::dsl::*;
use crate::schema::tokens::dsl::*;

use crate::routes::errors::RentError;

#[cfg_attr(test, mockable)]
pub fn get_rents(db: DbConn, as_of: &NaiveDateTime) -> Result<Vec<Rent>, diesel::result::Error> {
    rents.filter(end_timestamp.ge(&as_of))
        .filter(revocation_timestamp.is_null())
        .get_results::<Rent>(&*db)
}

#[cfg_attr(test, mockable)]
pub fn insert_booking(db: DbConn, booking: &Booking) -> Result<(), RentError> {
    (*db).transaction(|| {
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

        Ok(())
    })
}

#[cfg_attr(test, mockable)]
pub fn revoke_booking(db: DbConn, token: &Uuid) -> Result<(),RentError> {
    (&*db).transaction(|| {
        let token = tokens
            .filter(uuid.eq(token))
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