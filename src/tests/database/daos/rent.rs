use chrono::prelude::*;

use diesel::{RunQueryDsl,QueryDsl,Connection};
use diesel::{insert_into};

use crate::database::DbConn;

use crate::database::models::{Bike,Token,InsertRent,Booking};
use crate::schema::bikes::dsl::*;
use crate::schema::tokens::dsl::*;
use crate::schema::rents::dsl::*;

fn setup_database(db: &DbConn) -> Vec<Token> {
    let bike1 = insert_into(bikes).default_values().get_result::<Bike>(&**db).unwrap();
    let token1 = insert_into(tokens).default_values().get_result::<Token>(&**db).unwrap();
    let token2 = insert_into(tokens).default_values().get_result::<Token>(&**db).unwrap();

    let rent1 = InsertRent {
        token_id: token1.id,
        bike_id: bike1.id,
        start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-01T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-02T00:00:00.000Z".to_string()).unwrap().naive_utc(),
    };
    let rent2 = InsertRent {
        token_id: token2.id,
        bike_id: bike1.id,
        start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-03T00:00:00.000Z".to_string()).unwrap().naive_utc(),
        end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-04T00:00:00.000Z".to_string()).unwrap().naive_utc(),
    };

    insert_into(rents).values(rent1).execute(&**db).unwrap();
    insert_into(rents).values(rent2).execute(&**db).unwrap();

    vec![token1, token2]
}

#[test]
pub fn test_get_rents_with_epoch() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
        setup_database(&db);

        let expected: usize = 2;
        let actual: usize = crate::database::daos::rent::get_rents(&db, &DateTime::parse_from_rfc3339(&"1970-01-01T00:00:00.000Z".to_string()).unwrap().naive_utc())
            .unwrap()
            .len();

        assert_eq!(actual, expected);

        Ok(())
    });
}

#[test]
pub fn test_get_rents_with_timestamp_after_first_rent() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
        setup_database(&db);

        let expected: usize = 1;
        let actual: usize = crate::database::daos::rent::get_rents(&db, &DateTime::parse_from_rfc3339(&"1970-01-03T00:00:00.000Z".to_string()).unwrap().naive_utc())
            .unwrap()
            .len();

        assert_eq!(actual, expected);

        Ok(())
    });
}

#[test]
pub fn test_insert_booking_succeeds() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
        setup_database(&db);

        let bike1 = bikes.order_by(crate::schema::bikes::dsl::id)
            .limit(1)
            .get_result::<Bike>(&*db)
            .unwrap();
        let token3 = insert_into(tokens)
            .default_values()
            .get_result::<Token>(&*db)
            .unwrap();
        let booking = Booking {
            token: token3.uuid,
            bike_id: bike1.id,
            start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-05T00:00:00.000Z".to_string()).unwrap().naive_utc(),
            end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-06T00:00:00.000Z".to_string()).unwrap().naive_utc(),
            encrypted_details: "".to_string(),
            short_token: "".to_string(),
            email: None,
        };

        let actual = crate::database::daos::rent::insert_booking(&db, &booking);

        assert_eq!(actual.is_ok(), true);

        Ok(())
    });
}

#[test]
pub fn test_insert_booking_fails() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
        setup_database(&db);

        let bike1 = bikes.order_by(crate::schema::bikes::dsl::id)
            .limit(1)
            .get_result::<Bike>(&*db)
            .unwrap();
        let token3 = insert_into(tokens)
            .default_values()
            .get_result::<Token>(&*db)
            .unwrap();
        let booking = Booking {
            token: token3.uuid,
            bike_id: bike1.id,
            start_timestamp: DateTime::parse_from_rfc3339(&"1970-01-01T00:00:00.000Z".to_string()).unwrap().naive_utc(),
            end_timestamp: DateTime::parse_from_rfc3339(&"1970-01-06T00:00:00.000Z".to_string()).unwrap().naive_utc(),
            encrypted_details: "".to_string(),
            short_token: "".to_string(),
            email: None,
        };

        let actual = crate::database::daos::rent::insert_booking(&db, &booking);

        assert_eq!(actual.is_err(), true);

        Ok(())
    });
}

#[test]
pub fn test_revoke_booking_succeeds() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
        let test_tokens = setup_database(&db);

        let actual = crate::database::daos::rent::revoke_booking(&db, &test_tokens[0].uuid);

        assert_eq!(actual.is_ok(), true);

        Ok(())
    });
}

#[test]
pub fn test_revoke_booking_fails() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
        let test_tokens = setup_database(&db);

        assert_eq!(crate::database::daos::rent::revoke_booking(&db, &test_tokens[0].uuid).is_ok(), true);

        let actual = crate::database::daos::rent::revoke_booking(&db, &test_tokens[0].uuid);

        assert_eq!(actual.is_err(), true);

        Ok(())
    });
}