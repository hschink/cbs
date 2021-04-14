use std::fmt;

use rocket::Responder;

use lettre::transport::smtp::Error;

#[derive(Responder)]
#[derive(Debug)]
pub enum MailerError {
    MissingConfig(String),
    TransportError(String)
}

impl fmt::Display for MailerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MailerError::MissingConfig(ref err) => write!(f, "{}", err),
            MailerError::TransportError(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<Error> for MailerError {
    fn from(err: Error) -> MailerError {
        MailerError::TransportError(err.to_string())
    }
}