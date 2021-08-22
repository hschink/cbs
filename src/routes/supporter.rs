use rocket::get;
use rocket_contrib::json::Json;

use crate::database::DbConn;
use crate::database::models::SupporterWithTypeAndTranslatable;
use crate::database::daos::supporter;

use crate::routes::errors::RentError;

#[get("/supporters")]
pub fn get_supporters(db: DbConn) -> Result<Json<Vec<SupporterWithTypeAndTranslatable>>,RentError> {
    let data = supporter::get_supporters(&db)?;

    Ok(Json(data))
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
    use crate::database::daos::supporter;
    use crate::database::models::SupporterWithTypeAndTranslatable;

    #[test]
    fn test_get_bikes() {
        crate::database::test::setup();

        supporter::get_supporters.mock_safe(|_| MockResult::Return(Ok(vec![
            SupporterWithTypeAndTranslatable { id: 1,
                supporter_type_title: "Hardware".to_string(),
                locale: "de-DE".to_string(),
                title: "Test".to_string(),
                description: None,
                url: None,
                logo_url: None,
                logo_width: None,
                logo_height: None,
                logo_alt_text: None,
            },
            SupporterWithTypeAndTranslatable { id: 2,
                supporter_type_title: "Software".to_string(),
                locale: "de-DE".to_string(),
                title: "Test 2".to_string(),
                description: Some("Test description".to_string()),
                url: Some("https://test.rs/2".to_string()),
                logo_url: Some("https//test.rs/test.png".to_string()),
                logo_width: Some(500),
                logo_height: Some(300),
                logo_alt_text: Some("alt".to_string()),
            },
        ])));

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_supporters]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/supporters").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("[{\"id\":1,\"supporter_type_title\":\"Hardware\",\"locale\":\"de-DE\",\"title\":\"Test\",\"description\":null,\"url\":null,\"logo_url\":null,\"logo_width\":null,\"logo_height\":null,\"logo_alt_text\":null},{\"id\":2,\"supporter_type_title\":\"Software\",\"locale\":\"de-DE\",\"title\":\"Test 2\",\"description\":\"Test description\",\"url\":\"https://test.rs/2\",\"logo_url\":\"https//test.rs/test.png\",\"logo_width\":500,\"logo_height\":300,\"logo_alt_text\":\"alt\"}]".to_string()));
    }

    #[test]
    fn test_get_bikes_with_empty_result() {
        crate::database::test::setup();

        supporter::get_supporters.mock_safe(|_| MockResult::Return(Ok(vec![])));

        let rocket = rocket::ignite()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_supporters]);
        let client = Client::new(rocket).expect("valid rocket instance");

        let mut response = client.get("/supporters").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("[]".to_string()));
    }
}