use std::fmt;

use rocket::Responder;

use crate::mailer::errors::MailerError;

// http://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/error-handling.html#error-handling-with-a-custom-type

#[derive(Responder)]
#[derive(Debug)]
pub enum RentError {
    #[response(status = 400)]
    Parse(String),
    #[response(status = 400)]
    Database(String),
    #[response(status = 400)]
    Validation(String),
    #[response(status = 400)]
    MailError(String),
}

impl fmt::Display for RentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RentError::Parse(ref err) => write!(f, "{}", err),
            RentError::Database(ref err) => write!(f, "{}", err),
            RentError::Validation(ref err) => write!(f, "{}", err),
            RentError::MailError(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<chrono::ParseError> for RentError {
    fn from(err: chrono::ParseError) -> RentError {
        RentError::Parse(err.to_string())
    }
}

impl From<diesel::result::Error> for RentError {
    fn from(err: diesel::result::Error) -> RentError {
        RentError::Database(err.to_string())
    }
}

impl From<uuid::ParseError> for RentError {
    fn from(err: uuid::ParseError) -> RentError {
        RentError::Database(err.to_string())
    }
}

impl From<MailerError> for RentError {
    fn from(err: MailerError) -> RentError {
        RentError::MailError(err.to_string())
    }
}

#[derive(Responder)]
#[derive(Debug)]
pub enum ChallengeError {
    #[response(status = 400)]
    Parse(String),
    #[response(status = 400)]
    Database(String),
    #[response(status = 400)]
    Validation(String),
}

impl fmt::Display for ChallengeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ChallengeError::Parse(ref err) => write!(f, "{}", err),
            ChallengeError::Database(ref err) => write!(f, "{}", err),
            ChallengeError::Validation(ref err) => write!(f, "{}", err),
        }
    }
}

impl From<diesel::result::Error> for ChallengeError {
    fn from(err: diesel::result::Error) -> ChallengeError {
        ChallengeError::Database(err.to_string())
    }
}