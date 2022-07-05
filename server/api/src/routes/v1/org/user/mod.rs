use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod add;
mod list;
mod remove;

mod scope;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/user")
            .configure(scope::Router::configure)
            .route("/add", web::post().to(add::add))
            .route("/list", web::get().to(list::list))
            .route("/remove", web::post().to(remove::remove))
        );
    }
}