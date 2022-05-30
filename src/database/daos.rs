use mockall::automock;

sql_function!(fn random() -> Text);

#[automock()]
pub mod bike {
    use diesel::RunQueryDsl;

    use crate::database::DbConn;
    use crate::database::models::BikeTranslatable;
    use crate::schema::bike_translatables::dsl::{bike_translatables};

    pub async fn get_bikes(db: DbConn) -> Result<Vec<BikeTranslatable>, diesel::result::Error> {
        db.run(|c| bike_translatables.load::<BikeTranslatable>(c)).await
    }
}

#[automock()]
pub mod challenge {
    use super::random;

    use std::sync::Arc;

    use diesel::{RunQueryDsl, QueryDsl, BoolExpressionMethods, ExpressionMethods};
    use diesel::{insert_into};

    use crate::routes::errors::ChallengeError;
    use crate::database::DbConn;
    use crate::database::models::{TokenChallengeTranslatable, ChallengeResponse, Token};
    use crate::schema::tokens::dsl::{tokens};
    use crate::schema::token_challenge_translatables::dsl::{answer_hash,token_challenge_id,token_challenge_translatables};

    pub async fn get_random_challenge(db: DbConn, p_locale: Arc<String>) -> Result<TokenChallengeTranslatable, diesel::result::Error> {
        db.run(move |c| token_challenge_translatables
            .filter(crate::schema::token_challenge_translatables::dsl::locale.eq(Arc::try_unwrap(p_locale).unwrap()))
            .order(random())
            .limit(1)
            .get_result::<TokenChallengeTranslatable>(c)
        ).await
    }

    pub async fn test_challenge(db: DbConn, challenge_response: Arc<ChallengeResponse>) -> Result<Token, ChallengeError> {
        db.run(move |c| {
            let challenge_response = Arc::try_unwrap(challenge_response).unwrap();

            let result = token_challenge_translatables
                .filter(token_challenge_id.eq(challenge_response.token_challenge_id).and(answer_hash.eq(challenge_response.answer_hash.to_string())))
                .get_results::<TokenChallengeTranslatable>(c);

            match result {
                Ok(challenges) => {
                    if challenges.len() == 0 {
                        return Err(ChallengeError::Validation("Challenge response is not valid".to_string()))
                    }
                },
                Err(error) => return Err(ChallengeError::Database(error.to_string()))
            };

            let result = insert_into(tokens).default_values().get_result::<Token>(c);

            match result {
                Ok(token) => Ok(token),
                Err(error) => Err(ChallengeError::Database(error.to_string()))
            }
            
        }).await
    }
}

#[automock()]
pub mod rent {
    use std::sync::Arc;

    use ::uuid::Uuid;
    use chrono::{NaiveDateTime,Utc};

    use diesel::{RunQueryDsl, QueryDsl, BoolExpressionMethods, ExpressionMethods};
    use diesel::{insert_into, update};

    use crate::database::DbConn;
    use crate::database::models::{Booking, Rent, InsertRent, InsertRentDetail, Token};
    use crate::schema::rents::dsl::*;
    use crate::schema::rent_details::dsl::*;
    use crate::schema::tokens::dsl::*;

    use crate::routes::errors::RentError;

    pub async fn get_rents(db: DbConn, as_of: Arc<NaiveDateTime>) -> Result<Vec<Rent>, diesel::result::Error> {
        db.run(move |c| 
            rents.filter(end_timestamp.ge(Arc::try_unwrap(as_of).unwrap()))
                .filter(revocation_timestamp.is_null())
                .get_results::<Rent>(c)
        ).await
    }

    pub async fn insert_booking(db: DbConn, booking: Arc<Booking>) -> Result<(), RentError> {
        db.run(move |c|
            c.build_transaction()
                .run(|conn| {
                    let booking = Arc::try_unwrap(booking).unwrap();
                    let overlapping_rent_count = rents
                            .filter(revocation_timestamp.is_null()
                                .and(start_timestamp.between(booking.start_timestamp, booking.end_timestamp)
                                    .or(start_timestamp.between(booking.start_timestamp, booking.end_timestamp))
                                )
                            )
                            .count()
                            .get_result::<i64>(conn)?;
    
                    if overlapping_rent_count > 0 {
                        return Err(RentError::Validation(String::from("There is already a rent at the same period.")));
                    }
    
                    let token = tokens
                        .filter(uuid.eq(booking.token))
                        .get_result::<Token>(conn)?;
    
                    let rent = InsertRent {
                        token_id: token.id,
                        bike_id: booking.bike_id,
                        start_timestamp: booking.start_timestamp,
                        end_timestamp: booking.end_timestamp,
                    };
    
                    let inserted_rent = insert_into(rents)
                        .values(&rent)
                        .get_result::<Rent>(conn)?;
    
                    let rent_detail = InsertRentDetail {
                        rent_id: inserted_rent.id,
                        encrypted_details: booking.encrypted_details.clone(),
                    };
    
                    insert_into(rent_details)
                        .values(&rent_detail)
                        .execute(conn)?;
    
                    Ok(())
                })
        ).await
    }

    pub async fn revoke_booking(db: DbConn, token: Arc<Uuid>) -> Result<(),RentError> {
        db.run(move |c|
            c.build_transaction()
                .run(|conn| {
                    {
                        let token = tokens
                            .filter(uuid.eq(Arc::try_unwrap(token).unwrap()))
                            .get_result::<Token>(conn)?;
        
                        let booking = rents
                            .filter(revocation_timestamp.is_null())
                            .filter(token_id.eq(&token.id))
                            .get_result::<Rent>(conn)?;
        
                        update(&booking)
                            .set(revocation_timestamp.eq(Utc::now().naive_utc()))
                            .execute(conn)?;
        
                        Ok(())
                    }
                })
        ).await
    }
}

#[automock()]
pub mod supporter {
    use diesel::{RunQueryDsl,QueryDsl,NullableExpressionMethods};

    use crate::database::DbConn;
    use crate::database::models::SupporterWithTypeAndTranslatable;
    use crate::schema::supporters::dsl::*;
    use crate::schema::supporter_types::dsl::*;
    use crate::schema::supporter_translatables::dsl::*;

    pub async fn get_supporters(db: DbConn) -> Result<Vec<SupporterWithTypeAndTranslatable>, diesel::result::Error> {
        db.run(|c|
            supporters
                .inner_join(supporter_types)
                .inner_join(supporter_translatables)
                .select((
                    crate::schema::supporters::dsl::id,
                    crate::schema::supporter_types::dsl::title,
                    crate::schema::supporter_translatables::dsl::locale,
                    crate::schema::supporter_translatables::dsl::title,
                    crate::schema::supporter_translatables::dsl::description.nullable(),
                    crate::schema::supporter_translatables::dsl::url.nullable(),
                    crate::schema::supporter_translatables::dsl::logo_url.nullable(),
                    crate::schema::supporter_translatables::dsl::logo_width.nullable(),
                    crate::schema::supporter_translatables::dsl::logo_height.nullable(),
                    crate::schema::supporter_translatables::dsl::logo_alt_text.nullable(),
                ))
                .load::<SupporterWithTypeAndTranslatable>(c)
        ).await
    }
}