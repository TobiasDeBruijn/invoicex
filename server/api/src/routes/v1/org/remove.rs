use actix_multiresponse::Payload;
use dal::entities::{Entity, OrgScope};
use proto::RemoveOrgRequest;
use crate::empty::Empty;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn remove(data: WebData, session: Session, payload: Payload<RemoveOrgRequest>) -> WebResult<Empty> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &payload.org_id, OrgScope::RemoveOrg)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    access.org.remove()?;
    Ok(Empty)
}
