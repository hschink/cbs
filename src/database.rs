pub mod models;

use log::{info};

use rocket_contrib::database;
use rocket_contrib::databases::diesel;

embed_migrations!();

#[database("cbs")]
pub struct DbConn(diesel::PgConnection);

pub fn init(db: DbConn) -> Result<(), diesel::migration::RunMigrationsError> {
    info!("Initializing database");

    embedded_migrations::run(&*db)
}