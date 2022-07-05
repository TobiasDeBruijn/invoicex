use actix_multiresponse::Payload;
use dal::entities::{Entity, OrgScope, Product};
use proto::ProductRemoveRequest;
use crate::empty::Empty;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn remove(data: WebData, session: Session, payload: Payload<ProductRemoveRequest>) -> WebResult<Empty> {
    let product = Product::get(&data.driver, payload.product_id.clone())?.ok_or(Error::NotFound("Product not found".to_string()))?;
    let access = can_access(&data.driver, &session.user(&data.driver)?, &product.org_id, OrgScope::RemoveProduct)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()))?;
    }

    product.remove()?;
    Ok(Empty)
}
