use rocket::{get,post};
use rocket::http::RawStr;
use rocket_contrib::json;
use rocket_contrib::json::{Json,JsonValue};

use diesel::{RunQueryDsl,QueryDsl,BoolExpressionMethods,ExpressionMethods};
use diesel::{insert_into};

use regex::Regex;

use crate::database::DbConn;
use crate::database::models::*;
use crate::schema::tokens::dsl::*;
use crate::schema::token_challenge_translatables::dsl::*;

use crate::routes::error::ChallengeError;

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