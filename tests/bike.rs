#[macro_use]
extern crate rocket;

use serial_test::serial;

use rocket::fairing::AdHoc;
use rocket::local::blocking::Client;

use rocket::{Rocket, Build};

use diesel::{RunQueryDsl, insert_into, delete};

use cargobike_share_backend::database::DbConn;
use cargobike_share_backend::routes::bike;
use cargobike_share_backend::database::models::{Bike, BikeTranslatable, InsertBikeTranslatable};
use cargobike_share_backend::schema::bikes::dsl::bikes;
use cargobike_share_backend::schema::bike_translatables::dsl::bike_translatables;

#[test]
#[serial(bike)]
pub fn test_get_bikes_without_bikes_in_database() {

    let ignite = AdHoc::on_ignite("test_get_bikes_without_bikes_in_database", |rocket| async {
            rocket.attach(DbConn::fairing())
                .attach(AdHoc::on_ignite("DB Setup", test_get_bikes_without_bikes_in_database_setup))
                .mount("/", routes![bike::get_bikes])
        });
    let client = Client::tracked(rocket::build().attach(ignite)).unwrap();

    let expected: usize = 0;
    let actual: usize = client.get("/bikes")
        .dispatch()
        .into_json::<Vec<BikeTranslatable>>()
        .unwrap()
        .len();

    assert_eq!(actual, expected);
}

async fn test_get_bikes_without_bikes_in_database_setup(rocket: Rocket<Build>) -> Rocket<Build> {

    let conn = DbConn::get_one(&rocket).await.expect("database connection");

    conn.run( |c| {
        let _ = delete(bike_translatables).execute(c);
        let _ = delete(bikes).execute(c);
    }).await;

    rocket
}

#[test]
#[serial(bike)]
pub fn test_get_bikes_with_bikes_in_database() {

    let ignite = AdHoc::on_ignite("test_get_bikes_with_bikes_in_database", |rocket| async {
        rocket.attach(DbConn::fairing())
            .attach(AdHoc::on_ignite("DB Setup", test_get_bikes_with_bikes_in_database_setup))
            .mount("/", routes![bike::get_bikes])
    });
    let client = Client::tracked(rocket::build()
            .attach(ignite)
        ).unwrap();

    let expected: usize = 2;
    let actual: usize = client.get("/bikes")
        .dispatch()
        .into_json::<Vec<BikeTranslatable>>()
        .unwrap()
        .len();

    assert_eq!(actual, expected);
}

async fn test_get_bikes_with_bikes_in_database_setup(rocket: Rocket<Build>) -> Rocket<Build> {

    let conn = DbConn::get_one(&rocket).await.expect("database connection");

    conn.run( |c| {
        let _ = delete(bike_translatables).execute(c);
        let _ = delete(bikes).execute(c);

        let bike1 = insert_into(bikes).default_values().get_result::<Bike>(c).unwrap();
        let bike2 = insert_into(bikes).default_values().get_result::<Bike>(c).unwrap();
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

        insert_into(bike_translatables).values(&values).execute(c).expect("Could not insert bike translatables.");
    }).await;

    rocket
}