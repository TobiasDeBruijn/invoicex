use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod user;
mod remove;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/session")
            .route("/remove", web::post().to(remove::remove))
            .route("/user", web::get().to(user::user))
        );
    }
}