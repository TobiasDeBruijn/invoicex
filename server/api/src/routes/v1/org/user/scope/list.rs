use actix_multiresponse::Payload;
use actix_web::web;
use serde::Deserialize;
use dal::entities::{OrgScope, User, Entity};
use proto::OrgUserScopeListResponse;
use crate::error::{Error, WebResult};
use crate::routes::v1::org::get_user_org_scopes_proto;
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

#[derive(Deserialize)]
pub struct Query {
    org_id: String,
    user_id: Option<String>,
}

pub async fn list(data: WebData, session: Session, query: web::Query<Query>) -> WebResult<Payload<OrgUserScopeListResponse>> {
    let user = session.user(&data.driver)?;
    let access = can_access(&data.driver, &user, &query.org_id, OrgScope::GetOrg)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    let target_user = if let Some(user_id) = &query.user_id {
        User::get(&data.driver, user_id.clone())?.ok_or(Error::NotFound("User not found".to_string()))?
    } else {
        user
    };

    Ok(Payload(OrgUserScopeListResponse {
       org_scopes: get_user_org_scopes_proto(&target_user, &access.org)?
    }))
}