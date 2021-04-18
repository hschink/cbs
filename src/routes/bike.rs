use rocket::{get};
use rocket_contrib::json::Json;

use crate::database::DbConn;
use crate::database::models::BikeTranslatable;
use crate::database::daos::bike;

#[get("/bikes")]
pub fn get_bikes(db: DbConn) -> Result<Json<Vec<BikeTranslatable>>,diesel::result::Error> {
    let data = bike::get_bikes(db);

    match data {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod test {
    use mocktopus::mocking::Mockable;
    use mocktopus::mocking::MockResult;

    use rocket;
    use rocket::routes;
    use rocket::local::Client;
    use rocket::http::Status;

    use crate::database::DbConn;
    use crate::database::daos::bike;
    use crate::database::models::BikeTranslatable;

    #[test]
    fn test_get_bikes() {
        crate::database::test::setup();

        bike::get_bikes.mock_safe(|_| MockResult::Return(Ok(vec![
            BikeTranslatable { id: 1,
                bike_id: 1,
                locale: "de-DE".to_string(),
                title: "Test".to_string(),
                description: None,
                url: None
            },
            BikeTranslatable { id: 2,
                bike_id: 1,
                locale: "de-DE".to_string(),
                title: "Test 2".to_string(),
                description: Some("Test description".to_string()),
                url: Some("https://bikes.test.rs/2".to_string())
            },
        ])));

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_bikes]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/bikes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("[{\"id\":1,\"bike_id\":1,\"locale\":\"de-DE\",\"title\":\"Test\",\"description\":null,\"url\":null},{\"id\":2,\"bike_id\":1,\"locale\":\"de-DE\",\"title\":\"Test 2\",\"description\":\"Test description\",\"url\":\"https://bikes.test.rs/2\"}]".to_string()));
    }

    #[test]
    fn test_get_bikes_with_empty_result() {
        crate::database::test::setup();

        bike::get_bikes.mock_safe(|_| MockResult::Return(Ok(vec![])));

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_bikes]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/bikes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("[]".to_string()));
    }
}