use actix_multiresponse::Payload;
use actix_web::web;
use crate::error::WebResult;
use crate::session::Session;
use crate::WebData;
use serde::Deserialize;
use dal::entities::OrgScope;
use proto::OrgUserListResponse;
use crate::error::Error;
use crate::routes::v1::org::get_user_org_scopes_proto;
use crate::routes::v1::can_access;

#[derive(Debug, Deserialize)]
pub struct Query {
    org_id: String,
}

pub async fn list(data: WebData, session: Session, query: web::Query<Query>) -> WebResult<Payload<OrgUserListResponse>> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &query.org_id, OrgScope::GetOrg)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()))
    }

    let org = access.org;
    let users = org.list_users()?;
    let org_users = users.into_iter()
        .map(|x| {
            Ok(proto::OrgUser {
                org_scopes: get_user_org_scopes_proto(&x.user, &org)?,
                user: Some(proto::User {
                    id: x.user.id,
                    name: x.user.name,
                    email: x.user.email
                }),
                is_org_admin: x.is_org_admin,
            })
        })
        .collect::<WebResult<Vec<_>>>()?;

    Ok(Payload(OrgUserListResponse {
        org_users
    }))

}
