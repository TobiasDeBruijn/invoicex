use actix_multiresponse::Payload;
use dal::entities::{OrgScope, User, Entity};
use proto::OrgUserRemoveRequest;
use crate::empty::Empty;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn remove(data: WebData, session: Session, payload: Payload<OrgUserRemoveRequest>) -> WebResult<Empty> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &payload.org_id, OrgScope::OrgUserManagment)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    let target_user = User::get(&data.driver, payload.user_id.clone())?.ok_or(Error::NotFound("User not found".to_string()))?;
    let mut org = access.org;
    org.remove_user(&target_user)?;

    Ok(Empty)
}