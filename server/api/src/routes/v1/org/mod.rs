use actix_web::web;
use actix_web::web::ServiceConfig;
use dal::entities::{Org, OrgScope, User};
use crate::error::WebResult;
use crate::routable::Routable;

mod get;
mod list;
mod create;

mod user;
mod remove;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/org")
            .configure(user::Router::configure)
            .route("", web::get().to(get::get))
            .route("/create", web::post().to(create::create))
            .route("/list", web::get().to(list::list))
            .route("/remove", web::post().to(remove::remove))
        );
    }
}

/// Retrieve all OrgScope's the user has for the provided organization in Proto format
fn get_user_org_scopes_proto(user: &User<'_>, org: &Org<'_>) -> WebResult<Vec<proto::OrgScope>> {
    let owned_scopes = org.list_scopes(user)?;
    let org_scopes = OrgScope::variants()
        .into_iter()
        .map(|x| proto::OrgScope {
            name: x.to_string(),
            enabled: owned_scopes.contains(&x)
        })
        .collect::<Vec<_>>();
    Ok(org_scopes)
}