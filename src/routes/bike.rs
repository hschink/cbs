use rocket::{get};
use rocket_contrib::json::Json;

use diesel::RunQueryDsl;

use crate::database::DbConn;
use crate::database::models::BikeTranslatable;
use crate::schema::bike_translatables::dsl::{bike_translatables};

#[get("/bikes")]
pub fn get_bikes(db: DbConn) -> Result<Json<Vec<BikeTranslatable>>,diesel::result::Error> {
    let data = bike_translatables
        .load::<BikeTranslatable>(&*db);

    match data {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(e),
    }
}