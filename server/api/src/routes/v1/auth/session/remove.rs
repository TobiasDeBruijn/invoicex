use actix_web::web;
use serde::Deserialize;
use crate::empty::Empty;
use crate::error::WebResult;
use crate::session::Session;
use crate::WebData;

#[derive(Debug, Deserialize)]
pub struct Query {
    id: Option<String>,
    all: Option<bool>,
}

pub async fn remove(data: WebData, session: Session, query: web::Query<Query>) -> WebResult<Empty> {
    let mut user = session.user(&data.driver)?;

    if let Some(id) = &query.id {
        user.delete_session(id)?;
    } else if let Some(true) = &query.all {
        user.list_sessions()?
            .into_iter()
            .map(|x| user.delete_session(&x.id))
            .collect::<Result<Vec<_>, dal::Error>>()?;
    } else {
        user.delete_session(&session.id)?;
    }

    Ok(Empty)
}
