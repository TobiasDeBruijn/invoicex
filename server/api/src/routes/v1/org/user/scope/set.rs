use std::str::FromStr;
use actix_multiresponse::Payload;
use dal::entities::{OrgScope, User, Entity};
use proto::OrgUserScopeSetRequest;
use crate::empty::Empty;
use crate::error::{Error, WebResult};
use crate::routes::v1::org::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn set(data: WebData, session: Session, payload: Payload<OrgUserScopeSetRequest>) -> WebResult<Empty> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &payload.org_id, OrgScope::OrgUserManagment)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    let target_user = User::get(&data.driver, payload.user_id.clone())?.ok_or(Error::NotFound("User does not exist".to_string()))?;
    let mut org = access.org;

    for orgscope in &payload.org_scopes {
        let scope = OrgScope::from_str(&orgscope.name).map_err(|_| Error::BadRequest(format!("Unknown scope '{}'", orgscope.name)))?;
        org.set_scope(&target_user, &scope, orgscope.enabled)?;
    }

    Ok(Empty)
}