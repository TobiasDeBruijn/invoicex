use actix_web::web;
use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod list;
mod set;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/scope")
            .route("/list", web::get().to(list::list))
            .route("/set", web::post().to(set::set))
        );
    }
}