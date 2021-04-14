pub mod errors;

use std::env;

use lettre::transport::smtp::Error;
use lettre::transport::smtp::response::Response;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::database::models::Booking;
use crate::mailer::errors::MailerError;

fn send_mail_from_backend(to: Vec<&String>, subject: String, body: String) -> Result<Response, Error> {
    let mut message_builder = Message::builder();

    message_builder = message_builder.from(env::var("EMAIL_FROM").unwrap().parse().unwrap())
        .subject(format!("[{}] {}", env::var("EMAIL_SUBJECT_PREFIX").unwrap(), subject));

    for x in to {
        message_builder = message_builder.to(x.parse().unwrap());
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
    let result = send_mail_from_backend(vec![&env::var("EMAIL_TO").unwrap()],
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
    let initial_to = env::var("EMAIL_TO").unwrap();
    let mut to = vec![&initial_to];

    if booking.email.is_some() {
        to.push(booking.email.as_ref().unwrap());
    }

    let body = format!("{}", booking.short_token);

    let result = send_mail_from_backend(to, subject, body);

    match result {
        Ok(response) => Ok(response),
        Err(err) => Err(From::from(err))
    }
}