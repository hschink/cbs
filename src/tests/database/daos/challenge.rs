use diesel::{RunQueryDsl,Connection};
use diesel::{insert_into};

use crate::database::models::{TokenChallenge,InsertTokenChallengeTranslatable,ChallengeResponse};
use crate::schema::token_challenges::dsl::*;
use crate::schema::token_challenge_translatables::dsl::*;

#[test]
pub fn test_test_challenge_succeeds() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
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

        let actual = crate::database::daos::challenge::test_challenge(&db, &input);

        assert_eq!(actual.is_ok(), true);

        Ok(())
    });
}

#[test]
pub fn test_test_challenge_fails() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
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

        let actual = crate::database::daos::challenge::test_challenge(&db, &input);

        assert_eq!(actual.is_ok(), false);

        Ok(())
    });
}