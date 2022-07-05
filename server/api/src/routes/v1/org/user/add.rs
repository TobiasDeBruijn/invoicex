use actix_multiresponse::Payload;
use dal::entities::{OrgScope, User};
use proto::OrgUserAddRequest;
use crate::empty::Empty;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn add(data: WebData, session: Session, payload: Payload<OrgUserAddRequest>) -> WebResult<Empty> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &payload.org_id, OrgScope::OrgUserManagment)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    let user = User::get_by_email(&data.driver, &payload.user_email)?.ok_or(Error::NotFound("User not found".to_string()))?;

    let mut org = access.org;
    org.add_user(&user, payload.is_org_admin)?;

    Ok(Empty)
}
