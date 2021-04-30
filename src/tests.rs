pub mod database; 

use crate::database::DbConn;

pub fn get_database_connection() -> DbConn {
    let rocket = rocket::ignite().attach(DbConn::fairing());

    DbConn::get_one(&rocket).unwrap()
}