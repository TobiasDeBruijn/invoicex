use tracing::info;
use crate::config::Config;

mod config;

#[tokio::main]
async fn main() {
    setup_subscriber();

    info!("Starting InvoiceX v{}", env!("CARGO_PKG_VERSION"));

    info!("Reading configuration");
    let config = Config::new().await.expect("Reading configuration");

    info!("Initializing Mysql driver");
    let driver = dal::get_driver(dal::DriverConfig {
        host: config.mysql.host,
        database: config.mysql.database,
        username: config.mysql.username,
        password: config.mysql.password
    }).expect("Initializing mysql driver");

    info!("Initializing DAL");
    dal::init(&driver).expect("Initializing DAL");

    info!("Starting web server");
    api::start(api::Config {
        frontend_host: config.http.frontend_host,
        port: config.http.port,
        password_pepper: config.security.password_pepper
    }, driver).await.expect("Starting web server");
    // This method doesn't return as long as the web server is running


    info!("Shutting down");
}fn setup_subscriber() {
    let sub = tracing_subscriber::FmtSubscriber::builder()
        .pretty()
        .compact()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(sub).expect("Setting subscriber");
}
