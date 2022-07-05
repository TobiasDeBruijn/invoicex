use actix_web::web;
use actix_web::web::ServiceConfig;
use dal::entities::{Org, Product};
use crate::routable::Routable;

mod create;
mod get;
mod list;
mod remove;
mod update;

pub struct Router;

impl Routable for Router {
    fn configure(config: &mut ServiceConfig) {
        config.service(web::scope("/product")
            .route("", web::get().to(get::get))
            .route("/create", web::post().to(create::create))
            .route("/list", web::get().to(list::list))
            .route("/remove", web::post().to(remove::remove))
            .route("/update", web::post().to(update::update))
        );
    }
}

fn dal_product_to_proto(org: &Org<'_>, product: Product<'_>) -> proto::Product {
    proto::Product {
        id: product.id,
        name: product.name,
        description: product.description,
        org: Some(proto::Org {
            name: org.name.clone(),
            id: org.id.clone(),
        }),
        product_code: product.product_code,
        tax_percentage: product.tax_percentage,
        price_per_unit: product.price_per_unit
    }
}