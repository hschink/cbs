#[cfg(test)]
use mocktopus::macros::mockable;

use diesel::{RunQueryDsl,QueryDsl,BoolExpressionMethods,ExpressionMethods};
use diesel::{insert_into};

use crate::database::DbConn;
use crate::database::models::{TokenChallengeTranslatable, ChallengeResponse, Token};
use crate::schema::tokens::dsl::*;
use crate::schema::token_challenge_translatables::dsl::*;

no_arg_sql_function!(RANDOM, (), "Represents the sql RANDOM() function");

#[cfg_attr(test, mockable)]
pub fn get_random_challenge(db: DbConn, p_locale: &String) -> Result<TokenChallengeTranslatable, diesel::result::Error> {
    token_challenge_translatables
        .filter(crate::schema::token_challenge_translatables::dsl::locale.eq(p_locale))
        .order(RANDOM)
        .limit(1)
        .get_result::<TokenChallengeTranslatable>(&*db)
}

#[cfg_attr(test, mockable)]
pub fn test_challenge(db: DbConn, challenge_response: &ChallengeResponse) -> Result<Token, diesel::result::Error> {
    token_challenge_translatables
        .filter(token_challenge_id.eq(challenge_response.token_challenge_id).and(answer_hash.eq(challenge_response.answer_hash.to_string())))
        .get_result::<TokenChallengeTranslatable>(&*db)?;

    insert_into(tokens)
        .default_values()
        .get_result::<Token>(&*db)
}