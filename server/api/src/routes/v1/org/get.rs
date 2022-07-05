use actix_multiresponse::Payload;
use actix_web::web;
use serde::Deserialize;
use dal::entities::OrgScope;
use proto::GetOrgResponse;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

#[derive(Deserialize)]
pub struct Query {
    id: String,
}

pub async fn get(data: WebData, session: Session, query: web::Query<Query>) -> WebResult<Payload<GetOrgResponse>> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &query.id, OrgScope::GetOrg)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    let org_users = access.org.list_users()?;
    let org_users = org_users.into_iter()
        .map(|x| {
            let scopes = OrgScope::variants()
                .into_iter()
                .map(|scope| proto::OrgScope {
                    name: scope.to_string(),
                    enabled: x.scopes.contains(&scope)
                })
                .collect::<Vec<_>>();

            proto::OrgUser {
                user: Some(proto::User {
                    name: x.user.name,
                    id: x.user.id,
                    email: x.user.email
                }),
                is_org_admin: x.is_org_admin,
                org_scopes: scopes,
            }
        })
        .collect::<Vec<_>>();

    Ok(Payload(GetOrgResponse {
        org: Some(proto::Org {
            id: access.org.id,
            name: access.org.name
        }),
        org_users
    }))
}
