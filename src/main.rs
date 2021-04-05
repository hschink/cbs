#![feature(proc_macro_hygiene, decl_macro)]

use rocket;
use rocket::routes;

use cargobike_share_backend::database::{DbConn};
use cargobike_share_backend::routes;

fn main() {

    rocket::ignite()
        .attach(DbConn::fairing())
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
