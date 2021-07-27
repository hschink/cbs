#[cfg(test)]
use mocktopus::macros::mockable;

use diesel::{RunQueryDsl,QueryDsl,NullableExpressionMethods};

use crate::database::DbConn;
use crate::database::models::SupporterWithTypeAndTranslatable;
use crate::schema::supporters::dsl::*;
use crate::schema::supporter_types::dsl::*;
use crate::schema::supporter_translatables::dsl::*;

#[cfg_attr(test, mockable)]
pub fn get_supporters(db: &DbConn) -> Result<Vec<SupporterWithTypeAndTranslatable>, diesel::result::Error> {
    supporters
        .inner_join(supporter_types)
        .inner_join(supporter_translatables)
        .select((
            crate::schema::supporters::dsl::id,
            crate::schema::supporter_types::dsl::title,
            crate::schema::supporter_translatables::dsl::locale,
            crate::schema::supporter_translatables::dsl::title,
            crate::schema::supporter_translatables::dsl::description.nullable(),
            crate::schema::supporter_translatables::dsl::url.nullable(),
            crate::schema::supporter_translatables::dsl::logo_url.nullable(),
            crate::schema::supporter_translatables::dsl::logo_width.nullable(),
            crate::schema::supporter_translatables::dsl::logo_height.nullable(),
        ))
        .load::<SupporterWithTypeAndTranslatable>(&**db)
}