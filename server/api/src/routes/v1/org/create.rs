use actix_multiresponse::Payload;
use dal::entities::{Entity, Org, OrgBuilder};
use proto::{CreateOrgRequest, CreateOrgResponse};
use crate::error::WebResult;
use crate::session::Session;
use crate::WebData;

pub async fn create(data: WebData, session: Session, payload: Payload<CreateOrgRequest>) -> WebResult<Payload<CreateOrgResponse>> {
    let user = session.user(&data.driver)?;
    let org = Org::create(&data.driver, OrgBuilder {
        name: payload.name.clone(),
        creator: &user
    })?;

    Ok(Payload(CreateOrgResponse {
        org: Some(proto::Org {
            name: org.name,
            id: org.id
        })
    }))
}
