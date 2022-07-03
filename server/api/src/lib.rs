use actix_web::{App, HttpServer, web};
use dal::Driver;

mod error;
mod routable;
mod routes;

#[derive(Debug, Clone)]
pub struct Config {
    pub frontend_host: String,
    pub port: u16,
    pub password_pepper: String,
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct AppData {
    pub config: Config,
    pub driver: Driver,
}

pub(crate) type WebData = web::Data<AppData>;

pub async fn start(config: Config, driver: Driver) -> std::io::Result<()> {
    let appdata = AppData {
        config: config.clone(),
        driver
    };

    let data = web::Data::new(appdata);
    HttpServer::new(move || App::new()
        .app_data(data.clone())
        .wrap(tracing_actix_web::TracingLogger::default())
        .wrap(actix_cors::Cors::permissive())
    )
        .bind(format!("[::]:{}", config.port))?
        .run()
        .await
}