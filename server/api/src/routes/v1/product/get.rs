use actix_multiresponse::Payload;
use actix_web::web;
use serde::Deserialize;
use dal::entities::{Entity, OrgScope, Product};
use proto::ProductGetResponse;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::routes::v1::product::dal_product_to_proto;
use crate::session::Session;
use crate::WebData;

#[derive(Deserialize, Debug)]
pub struct Query {
    product_id: String,
}

pub async fn get(data: WebData, session: Session, query: web::Query<Query>) -> WebResult<Payload<ProductGetResponse>> {
    let product = Product::get(&data.driver, query.product_id.clone())?.ok_or(Error::NotFound("Product not found".to_string()))?;
    let access = can_access(&data.driver, &session.user(&data.driver)?, &product.org_id, OrgScope::GetProduct)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    Ok(Payload(ProductGetResponse {
        product: Some(dal_product_to_proto(&access.org, product))
    }))
}
