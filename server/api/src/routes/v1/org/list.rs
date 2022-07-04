use actix_multiresponse::Payload;
use dal::entities::{Org, OrgScope};
use proto::ListOrgResponse;
use crate::error::WebResult;
use crate::routes::v1::org::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn list(data: WebData, session: Session) -> WebResult<Payload<ListOrgResponse>> {
    let user = session.user(&data.driver)?;
    let orgs = Org::list_available(&data.driver)?
        .into_iter()
        // Check if the user is allowed to access this org
        .filter(|x| {
            let access = match can_access(&data.driver, &user, &x.id, OrgScope::GetOrg) {
                Ok(x) => x,
                Err(_) => return false, // Skip on Err
            };

            access.accessible
        })
        .map(|x| proto::Org {
            name: x.name,
            id: x.id
        })
        .collect::<Vec<_>>();

    Ok(Payload(ListOrgResponse {
        orgs
    }))
}