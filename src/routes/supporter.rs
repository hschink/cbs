use mockall_double::double;

use rocket::get;
use rocket::serde::json::Json;

use crate::database::DbConn;
use crate::database::models::SupporterWithTypeAndTranslatable;

#[double]
use crate::database::daos::supporter;

use crate::routes::errors::RentError;

#[get("/supporters")]
pub async fn get_supporters(db: DbConn) -> Result<Json<Vec<SupporterWithTypeAndTranslatable>>,RentError> {
    let data = supporter::get_supporters(db).await?;

    Ok(Json(data))
}

#[cfg(test)]
mod test {
    use super::*;

    use rocket;
    use rocket::routes;
    use rocket::http::Status;

    use rocket::local::blocking::Client;

    use crate::database::DbConn;
    use crate::database::models::SupporterWithTypeAndTranslatable;

    use lazy_static::lazy_static;
    use std::sync::{Mutex, MutexGuard};

    lazy_static! {
        static ref MTX: Mutex<()> = Mutex::new(());
    }

    // When a test panics, it will poison the Mutex. Since we don't actually
    // care about the state of the data we ignore that it is poisoned and grab
    // the lock regardless.  If you just do `let _m = &MTX.lock().unwrap()`, one
    // test panicking will cause all other tests that try and acquire a lock on
    // that Mutex to also panic.
    fn get_lock(m: &'static Mutex<()>) -> MutexGuard<'static, ()> {
        match m.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    #[test]
    fn test_get_bikes() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let get_supporters_ctx = supporter::get_supporters_context();

        get_supporters_ctx.expect()
            .returning(|_| Ok(vec![
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
            ]));

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_supporters]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/supporters").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("[{\"id\":1,\"supporter_type_title\":\"Hardware\",\"locale\":\"de-DE\",\"title\":\"Test\",\"description\":null,\"url\":null,\"logo_url\":null,\"logo_width\":null,\"logo_height\":null,\"logo_alt_text\":null},{\"id\":2,\"supporter_type_title\":\"Software\",\"locale\":\"de-DE\",\"title\":\"Test 2\",\"description\":\"Test description\",\"url\":\"https://test.rs/2\",\"logo_url\":\"https//test.rs/test.png\",\"logo_width\":500,\"logo_height\":300,\"logo_alt_text\":\"alt\"}]".to_string()));
    }

    #[test]
    fn test_get_bikes_with_empty_result() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let get_supporters_ctx = supporter::get_supporters_context();

        get_supporters_ctx.expect()
            .returning(|_| Ok(vec![]));

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_supporters]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/supporters").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("[]".to_string()));
    }
}