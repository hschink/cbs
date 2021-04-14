#![feature(proc_macro_hygiene, decl_macro)]

extern crate dotenv;

use dotenv::dotenv;

use rocket;
use rocket::routes;
use rocket::fairing::AdHoc;

use cargobike_share_backend::database::{DbConn};
use cargobike_share_backend::routes;
use cargobike_share_backend::mailer;

fn main() {
    dotenv().ok();

    rocket::ignite()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_launch("Send launch mail", |_| {
            if mailer::is_mail_config_available() == false {
                panic!("Launch failed due to missing mail configuration");
            }

            match mailer::send_startup_mail() {
                Ok(_) => println!("Application is about to launch..."),
                Err(err) => panic!("Launch failed: {:?}", err)
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
