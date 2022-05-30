use mockall::automock;

#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;

pub mod routes;

pub mod schema;

pub mod database;

pub mod mailer_errors;

#[automock()]
pub mod mailer {

    use std::env;

    use lettre::transport::smtp::Error;
    use lettre::transport::smtp::response::Response;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    use crate::database::models::Booking;
    use crate::mailer_errors::MailerError;

    fn send_mail_from_backend(to: Vec<String>, subject: String, body: String) -> Result<Response, Error> {
        let mut message_builder = Message::builder();

        message_builder = message_builder.from(env::var("EMAIL_FROM").unwrap().parse().unwrap())
            .reply_to(env::var("EMAIL_FROM").unwrap().parse().unwrap())
            .to(to[0].parse().unwrap())
            .subject(format!("[{}] {}", env::var("EMAIL_SUBJECT_PREFIX").unwrap(), subject));

        for x in &to[1..] {
            message_builder = message_builder.bcc(x.parse().unwrap());
        }

        let email = message_builder.body(body).unwrap();

        let creds = Credentials::new(env::var("SMTP_USER").unwrap(), env::var("SMTP_PASSWORD").unwrap());

        let mailer = SmtpTransport::relay(&env::var("SMTP_SERVER").unwrap())
            .unwrap()
            .credentials(creds)
            .build();

        mailer.send(&email)
    }

    pub fn is_mail_config_available() -> bool {
        env::var("SMTP_USER").is_ok()
            && env::var("SMTP_PASSWORD").is_ok()
            && env::var("SMTP_SERVER").is_ok()
            && env::var("EMAIL_FROM").is_ok()
            && env::var("EMAIL_TO").is_ok()
    }

    pub fn send_startup_mail() -> Result<Response, MailerError> {
        let email_to = env::var("EMAIL_TO").unwrap();
        let to = email_to.split(",")
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let result = send_mail_from_backend(to,
            "Launch".to_string(),
            "Cargobike share backend is about to launch!".to_string()
        );

        match result {
            Ok(response) => Ok(response),
            Err(err) => Err(From::from(err))
        }
    }

    pub fn send_rent_mail(booking: &Booking) -> Result<Response, MailerError> {
        let start_date = booking.start_timestamp.format("%Y-%m-%d");
        let end_date = booking.end_timestamp.format("%Y-%m-%d");
        let subject = format!("New rent from {} to {}", start_date, end_date);
        let email_to = env::var("EMAIL_TO").unwrap();
        let mut to = email_to.split(",")
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        if booking.email.is_some() {
            to.push(booking.email.as_ref().unwrap().clone());
        }

        let body = format!("{}", booking.short_token);

        let result = send_mail_from_backend(to, subject, body);

        match result {
            Ok(response) => Ok(response),
            Err(err) => Err(From::from(err))
        }
    }
}