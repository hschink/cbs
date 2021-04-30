#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;

pub mod routes;

pub mod schema;

pub mod database;

pub mod mailer;

#[cfg(test)]
pub mod tests;