use actix_web::web::ServiceConfig;
use crate::routable::Routable;

mod get;
mod remove;

pub struct Router;

impl Routable for Router {
    fn configure(_config: &mut ServiceConfig) {
        todo!()
    }
    
}