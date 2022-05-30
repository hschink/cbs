use mockall_double::double;

use rocket::get;
use rocket::serde::json::Json;

use crate::database::DbConn;
use crate::database::models::BikeTranslatable;

#[double]
use crate::database::daos::bike;

use crate::routes::errors::BikeError;

#[get("/bikes")]
pub async fn get_bikes(db: DbConn) -> Result<Json<Vec<BikeTranslatable>>,BikeError> {
    let data = bike::get_bikes(db).await?;

    Ok(Json(data))
}

#[cfg(test)]
mod test {
    use super::*;

    use rocket;
    use rocket::routes;
    use rocket::http::Status;

    use rocket::local::blocking::Client;

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

        let bike_ctx = bike::get_bikes_context();

        bike_ctx.expect()
            .returning(|_| Ok(vec![
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
            ]));

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_bikes]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/bikes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("[{\"id\":1,\"bike_id\":1,\"locale\":\"de-DE\",\"title\":\"Test\",\"description\":null,\"url\":null},{\"id\":2,\"bike_id\":1,\"locale\":\"de-DE\",\"title\":\"Test 2\",\"description\":\"Test description\",\"url\":\"https://bikes.test.rs/2\"}]".to_string()));
    }

    #[test]
    fn test_get_bikes_with_empty_result() {
        let _m = get_lock(&MTX);

        crate::database::test::setup();

        let bike_ctx = bike::get_bikes_context();

        bike_ctx.expect()
            .returning(|_| Ok(vec![]));

        let rocket = rocket::build()
            .attach(DbConn::fairing())
            .mount("/", routes![super::get_bikes]);
        let client = Client::tracked(rocket).expect("valid rocket instance");

        let response = client.get("/bikes").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("[]".to_string()));
    }
}