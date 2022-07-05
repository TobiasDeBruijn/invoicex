use actix_multiresponse::Payload;
use actix_web::web;
use serde::Deserialize;
use dal::entities::{OrgScope, Product};
use proto::ProductListResponse;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::routes::v1::product::dal_product_to_proto;
use crate::session::Session;
use crate::WebData;

#[derive(Debug, Deserialize)]
pub struct Query {
    org_id: String,
}

pub async fn list(data: WebData, session: Session, query: web::Query<Query>) -> WebResult<Payload<ProductListResponse>> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &query.org_id, OrgScope::GetProduct)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()))?;
    }

    let products = Product::list_for_org(&data.driver, &access.org)?
        .into_iter()
        .map(|x| dal_product_to_proto(&access.org, x))
        .collect::<Vec<_>>();

    Ok(Payload(ProductListResponse {
        products
    }))
}
