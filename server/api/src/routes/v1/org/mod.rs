use actix_web::web;
use actix_web::web::ServiceConfig;
use dal::Driver;
use dal::entities::{Entity, Org, OrgScope, User};
use crate::error::{Error, WebResult};
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
            .route("", web::get().to(get::get))
            .route("/create", web::post().to(create::create))
            .route("/list", web::get().to(list::list))
            .route("/remove", web::post().to(remove::remove))

        );
    }
}

struct AccessResult<'a> {
    pub accessible: bool,
    pub org: Org<'a>
}

/// Checks if a user can access the requested scope in the requested organization
fn can_access<'a>(driver: &'a Driver,user: &User<'_>, org_id: &str, scope: OrgScope) -> WebResult<AccessResult<'a>> {
    let org = Org::get(&driver, org_id.to_string())?.ok_or(Error::Unauthorized("The requested organization does not exist or the user has no access".to_string()))?;
    let scopes = org.list_scopes(user)?;

    Ok(AccessResult {
        accessible: scopes.contains(&scope),
        org
    })
}