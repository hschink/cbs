use diesel::{RunQueryDsl,Connection};
use diesel::{insert_into};

use crate::database::models::{Bike,InsertBikeTranslatable};
use crate::schema::bikes::dsl::*;
use crate::schema::bike_translatables::dsl::*;

#[test]
pub fn test_get_bikes_without_bikes_in_database() {
    let db = crate::tests::get_database_connection();

    (&*db).test_transaction::<_, diesel::result::Error, _>(|| {
        let expected: usize = 0;
        let actual: usize = crate::database::daos::bike::get_bikes(&db).unwrap().len();

        assert_eq!(actual, expected);

        Ok(())
    });
}

#[test]
pub fn test_get_bikes_with_bikes_in_database() {
    let db = crate::tests::get_database_connection();

    db.test_transaction::<_, diesel::result::Error, _>(|| {
        let bike1 = insert_into(bikes).default_values().get_result::<Bike>(&*db).unwrap();
        let bike2 = insert_into(bikes).default_values().get_result::<Bike>(&*db).unwrap();
        let bike_translatable1 = InsertBikeTranslatable {
            bike_id: bike1.id,
            locale: "de-DE".to_string(),
            title: "Bike1".to_string(),
            description: None,
            url: None,
        };
        let bike_translatable2 = InsertBikeTranslatable {
            bike_id: bike2.id,
            locale: "de-DE".to_string(),
            title: "Bike2".to_string(),
            description: None,
            url: None,
        };
        let values = vec![bike_translatable1, bike_translatable2];

        insert_into(bike_translatables).values(&values).execute(&*db).expect("Could not insert bike translatables.");

        let expected: usize = values.len();
        let actual: usize = crate::database::daos::bike::get_bikes(&db).unwrap().len();

        println!("{} - {}", expected, actual);
        assert_eq!(actual, expected);

        Ok(())
    });
}