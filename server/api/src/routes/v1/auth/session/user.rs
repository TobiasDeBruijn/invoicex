use actix_multiresponse::Payload;
use crate::error::WebResult;
use crate::session::Session;
use crate::WebData;

pub async fn user(data: WebData, session: Session) -> WebResult<Payload<proto::User>> {
    let user = session.user(&data.driver)?;
    Ok(Payload(proto::User {
        id: user.id,
        name: user.name,
        email: user.email
    }))
}
