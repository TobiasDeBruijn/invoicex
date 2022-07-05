use actix_multiresponse::Payload;
use dal::entities::{OrgScope, Product, ProductBuilder, Entity};
use proto::{ProductCreateRequest, ProductCreateResponse};
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn create(data: WebData, session: Session, payload: Payload<ProductCreateRequest>) -> WebResult<Payload<ProductCreateResponse>> {
    let access = can_access(&data.driver, &session.user(&data.driver)?, &payload.org_id, OrgScope::CreateProduct)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    let product = Product::create(&data.driver, ProductBuilder {
        name: payload.name.clone(),
        description: payload.description.clone(),
        org: &access.org,
        product_code: payload.product_code.clone(),
        price_per_unit: payload.price_per_unit,
        tax_percentage: payload.tax_percentage,
    })?;

    Ok(Payload(ProductCreateResponse {
        product_id: product.id,
    }))
}
