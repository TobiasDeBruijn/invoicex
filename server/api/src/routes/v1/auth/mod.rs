use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod login;
mod session;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/auth")
            .configure(login::Router::configure)
            .configure(session::Router::configure)
        );
    }
}