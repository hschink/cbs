use chrono::prelude::NaiveDateTime;

use serde::{Deserialize,Serialize};

use uuid::Uuid;

use crate::schema::*;

#[derive(Queryable,Serialize)]
#[derive(Debug)]
pub struct Bike {
    pub id: i32
}

#[derive(Queryable,Serialize)]
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
#[table_name="bike_translatables"]
#[derive(Debug)]
pub struct InsertBikeTranslatable {
    pub bike_id: i32,
    pub locale: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>
}

#[derive(Queryable,Identifiable,Serialize)]
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
#[table_name="tokens"]
#[derive(Debug)]
pub struct Token {
    pub id: i32,
    pub uuid: Uuid,
    pub created_at: NaiveDateTime
}

#[derive(Deserialize)]
pub struct Booking {
    pub token: Uuid,
    pub bike_id: i32,
    pub start_timestamp: NaiveDateTime,
    pub end_timestamp: NaiveDateTime,
    pub encrypted_details: String,
    pub short_token: String,
    pub email: Option<String>
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

#[derive(Deserialize)]
pub struct ChallengeResponse {
    pub token_challenge_id: i32,
    pub answer_hash: String
}

#[derive(Insertable,Deserialize)]
#[table_name="rents"]
#[derive(Debug)]
pub struct InsertRent {
    pub token_id: i32,
    pub bike_id: i32,
    pub start_timestamp: NaiveDateTime,
    pub end_timestamp: NaiveDateTime
}

#[derive(Insertable,Deserialize)]
#[table_name="rent_details"]
#[derive(Debug)]
pub struct InsertRentDetail {
    pub rent_id: i32,
    pub encrypted_details: String,
}