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
pub fn get_random_challenge(db: &DbConn, p_locale: &String) -> Result<TokenChallengeTranslatable, diesel::result::Error> {
    token_challenge_translatables
        .filter(crate::schema::token_challenge_translatables::dsl::locale.eq(p_locale))
        .order(RANDOM)
        .limit(1)
        .get_result::<TokenChallengeTranslatable>(&**db)
}

#[cfg_attr(test, mockable)]
pub fn test_challenge(db: &DbConn, challenge_response: &ChallengeResponse) -> Result<Token, diesel::result::Error> {
    token_challenge_translatables
        .filter(token_challenge_id.eq(challenge_response.token_challenge_id).and(answer_hash.eq(challenge_response.answer_hash.to_string())))
        .get_result::<TokenChallengeTranslatable>(&**db)?;

    insert_into(tokens)
        .default_values()
        .get_result::<Token>(&**db)
}

#[cfg(test)]
mod test {

    use diesel::{RunQueryDsl,Connection};
    use diesel::{insert_into};

    use crate::database::DbConn;

    use crate::database::models::{TokenChallenge,InsertTokenChallengeTranslatable,ChallengeResponse};
    use crate::schema::token_challenges::dsl::*;
    use crate::schema::token_challenge_translatables::dsl::*;

    #[test]
    pub fn test_test_challenge_succeeds() {
        let rocket = rocket::ignite().attach(DbConn::fairing());
        let db = DbConn::get_one(&rocket).unwrap();

        (&*db).test_transaction::<_, diesel::result::Error, _>(|| {
            let answer_hash_value = "123".to_string();
            let token_challenge = insert_into(token_challenges).default_values().get_result::<TokenChallenge>(&*db).unwrap();
            let token_challenge_translatable = InsertTokenChallengeTranslatable {
                token_challenge_id: token_challenge.id,
                locale: "de-DE".to_string(),
                question: "".to_string(),
                answer_hash: answer_hash_value.to_string(),
                url: None,
            };

            insert_into(token_challenge_translatables)
                .values(token_challenge_translatable)
                .execute(&*db)
                .expect("Could not insert token challenge translatables.");

            let input = ChallengeResponse {
                token_challenge_id: token_challenge.id,
                answer_hash: answer_hash_value.to_string(),
            };

            let actual = super::test_challenge(&db, &input);

            assert_eq!(actual.is_ok(), true);

            Ok(())
        });
    }

    #[test]
    pub fn test_test_challenge_fails() {
        let rocket = rocket::ignite().attach(DbConn::fairing());
        let db = DbConn::get_one(&rocket).unwrap();

        (&*db).test_transaction::<_, diesel::result::Error, _>(|| {
            let answer_hash_value = "123".to_string();
            let token_challenge = insert_into(token_challenges).default_values().get_result::<TokenChallenge>(&*db).unwrap();
            let token_challenge_translatable = InsertTokenChallengeTranslatable {
                token_challenge_id: token_challenge.id,
                locale: "de-DE".to_string(),
                question: "".to_string(),
                answer_hash: "wrong".to_string(),
                url: None,
            };

            insert_into(token_challenge_translatables)
                .values(token_challenge_translatable)
                .execute(&*db)
                .expect("Could not insert token challenge translatables.");

            let input = ChallengeResponse {
                token_challenge_id: token_challenge.id,
                answer_hash: answer_hash_value.to_string(),
            };

            let actual = super::test_challenge(&db, &input);

            assert_eq!(actual.is_ok(), false);

            Ok(())
        });
    }
}