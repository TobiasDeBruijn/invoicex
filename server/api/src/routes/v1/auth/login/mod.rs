use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod login;
mod register;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/login")
            .route("", web::post().to(login::login))
            .route("/register", web::post().to(register::register))
        );
    }
}