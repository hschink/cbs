use chrono::prelude::NaiveDateTime;

use serde::{Deserialize,Serialize};

use uuid::Uuid;

use crate::schema::*;

#[derive(Queryable,Serialize)]
#[derive(Debug)]
pub struct Bike {
    pub id: i32
}

#[derive(Deserialize,Queryable,Serialize)]
#[derive(Debug)]
pub struct BikeTranslatable {
    pub id: i32,
    pub bike_id: i32,
    pub locale: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>
}

#[derive(Insertable,Deserialize)]
#[diesel(table_name = bike_translatables)]
#[derive(Debug)]
pub struct InsertBikeTranslatable {
    pub bike_id: i32,
    pub locale: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>
}

#[derive(Queryable,Identifiable,Serialize,Deserialize)]
#[derive(Debug)]
pub struct Rent {
    pub id: i32,
    pub token_id: i32,
    pub bike_id: i32,
    pub created_at: NaiveDateTime,
    pub start_timestamp: NaiveDateTime,
    pub end_timestamp: NaiveDateTime,
    pub revocation_timestamp: Option<NaiveDateTime>
}

#[derive(Queryable,Identifiable,Serialize)]
#[diesel(table_name = tokens)]
#[derive(Debug)]
pub struct Token {
    pub id: i32,
    pub uuid: Uuid,
    pub created_at: NaiveDateTime
}

#[derive(Serialize,Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct Booking {
    pub token: Uuid,
    pub bike_id: i32,
    pub start_timestamp: NaiveDateTime,
    pub end_timestamp: NaiveDateTime,
    pub encrypted_details: String,
    pub short_token: String,
    pub email: Option<String>
}

#[derive(Queryable,Serialize)]
#[derive(Debug)]
pub struct TokenChallenge {
    pub id: i32
}

#[derive(Queryable,Identifiable,Serialize)]
#[derive(Debug)]
pub struct TokenChallengeTranslatable {
    pub id: i32,
    pub token_challenge_id: i32,
    pub locale: String,
    pub question: String,
    pub answer_hash: String,
    pub url: Option<String>
}

#[derive(Insertable,Deserialize)]
#[diesel(table_name = token_challenge_translatables)]
#[derive(Debug)]
pub struct InsertTokenChallengeTranslatable {
    pub token_challenge_id: i32,
    pub locale: String,
    pub question: String,
    pub answer_hash: String,
    pub url: Option<String>
}

#[derive(Deserialize)]
#[derive(Debug)]
#[derive(Clone)]
pub struct ChallengeResponse {
    pub token_challenge_id: i32,
    pub answer_hash: String
}

#[derive(Insertable,Deserialize)]
#[diesel(table_name = rents)]
#[derive(Debug)]
pub struct InsertRent {
    pub token_id: i32,
    pub bike_id: i32,
    pub start_timestamp: NaiveDateTime,
    pub end_timestamp: NaiveDateTime
}

#[derive(Insertable,Deserialize)]
#[diesel(table_name = rent_details)]
#[derive(Debug)]
pub struct InsertRentDetail {
    pub rent_id: i32,
    pub encrypted_details: String,
}

#[derive(Queryable,Serialize)]
#[derive(Debug)]
pub struct SupporterWithTypeAndTranslatable {
    pub id: i32,
    pub supporter_type_title: String,
    pub locale: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub logo_url: Option<String>,
    pub logo_width: Option<i16>,
    pub logo_height: Option<i16>,
    pub logo_alt_text: Option<String>,
}