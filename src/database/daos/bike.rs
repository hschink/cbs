#[cfg(test)]
use mocktopus::macros::mockable;

use diesel::RunQueryDsl;

use crate::database::DbConn;
use crate::database::models::BikeTranslatable;
use crate::schema::bike_translatables::dsl::{bike_translatables};

#[cfg_attr(test, mockable)]
pub fn get_bikes(db: DbConn) -> Result<Vec<BikeTranslatable>, diesel::result::Error> {
    bike_translatables.load::<BikeTranslatable>(&*db)
}