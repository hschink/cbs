pub mod errors;
pub mod bike;
pub mod rent;
pub mod challenge;
pub mod supporter;

use rocket::get;
use rocket::serde::json::{Value,json};

#[get("/")]
pub fn index() -> Value {
    json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION")
    })
}

#[cfg(test)]
mod test {
    use rocket;
    use rocket::routes;
    use rocket::http::Status;

    use rocket::local::blocking::Client;

    #[test]
    fn test_index() {
        let rocket = rocket::build().mount("/", routes![super::index]);

        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/").dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some(format!("{{\"name\":\"{}\",\"version\":\"{}\"}}"
            , env!("CARGO_PKG_NAME")
            , env!("CARGO_PKG_VERSION")
        )));
    }
}