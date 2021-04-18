pub mod models;

pub mod daos;

use rocket_contrib::database;
use rocket_contrib::databases::diesel;

#[database("cbs")]
pub struct DbConn(diesel::PgConnection);

#[cfg(test)]
pub mod test {
    use dotenv::dotenv;

    pub fn setup() {
        dotenv().ok();
    }
}