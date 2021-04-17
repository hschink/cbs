pub mod error;
pub mod bike;
pub mod rent;
pub mod challenge;

use rocket::get;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;

#[get("/")]
pub fn index() -> JsonValue {
    json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION")
    })
}