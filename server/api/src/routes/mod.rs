use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod v1;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.configure(v1::Router::configure);
    }
}