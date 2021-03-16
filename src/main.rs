#![feature(proc_macro_hygiene, decl_macro)]

use log::error;

use rocket;
use rocket::fairing::AdHoc;
use rocket::routes;

use cargobike_share_backend::database::{DbConn,init};
use cargobike_share_backend::routes;

fn main() {

    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_attach("Database initializiation", |rocket| {
            let db = DbConn::get_one(&rocket).expect("Database connection");

            match init(db) {
                Ok(()) => Ok(rocket),
                Err(e) => {
                    error!("Failed to run database migrations: {:?}", e);
                    Err(rocket)
                }
            }
        }))
        .mount("/", routes![routes::index,
            routes::get_bikes,
            routes::get_rents,
            routes::book,
            routes::revoke_booking,
            routes::get_random_challenge,
            routes::test_challenge
        ])
        .launch();
}
