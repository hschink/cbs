pub mod errors;
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

#[cfg(test)]
mod test {
    use rocket;
    use rocket::routes;
    use rocket::local::Client;
    use rocket::http::Status;

    #[test]
    fn test_index() {
        let rocket = rocket::ignite().mount("/", routes![super::index]);
        let client = Client::new(rocket).expect("valid rocket instance");
        let mut response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some(format!("{{\"name\":\"{}\",\"version\":\"{}\"}}"
            , env!("CARGO_PKG_NAME")
            , env!("CARGO_PKG_VERSION")
        )));
    }
}