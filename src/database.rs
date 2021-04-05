pub mod models;

use rocket_contrib::database;
use rocket_contrib::databases::diesel;

#[database("cbs")]
pub struct DbConn(diesel::PgConnection);