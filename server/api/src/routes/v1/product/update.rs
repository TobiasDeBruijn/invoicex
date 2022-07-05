use actix_multiresponse::Payload;
use dal::entities::{Entity, OrgScope, Product};
use proto::ProductUpdateRequest;
use crate::empty::Empty;
use crate::error::{Error, WebResult};
use crate::routes::v1::can_access;
use crate::session::Session;
use crate::WebData;

pub async fn update(data: WebData, session: Session, payload: Payload<ProductUpdateRequest>) -> WebResult<Empty> {
    let mut product = Product::get(&data.driver, payload.product_id.clone())?.ok_or(Error::NotFound("Product not found".to_string()))?;
    let access = can_access(&data.driver, &session.user(&data.driver)?, &product.org_id, OrgScope::RemoveProduct)?;
    if !access.accessible {
        return Err(Error::Forbidden(String::default()));
    }

    if let Some(name) = &payload.name {
        product.name = name.clone();
    }

    if let Some(price_per_unit) = payload.price_per_unit {
        product.price_per_unit = price_per_unit;
    }

    if let Some(remove_description) = payload.remove_description {
        if remove_description {
            product.description = None;
        }
    } else {
        if let Some(description) = &payload.description {
            product.description = Some(description.clone());
        }
    }

    if let Some(remove_product_code) = payload.remove_product_code {
        if remove_product_code {
            product.product_code = None;
        }
    } else {
        if let Some(product_code) = &payload.product_code {
            product.product_code = Some(product_code.clone())
        }
    }

    if let Some(remove_tax_percentage) = payload.remove_tax_percentage {
        if remove_tax_percentage {
            product.tax_percentage = None;
        }
    } else {
        if let Some(tax_percentage) = payload.tax_percentage {
            product.tax_percentage = Some(tax_percentage);
        }
    }

    product.update()?;
    Ok(Empty)
}
