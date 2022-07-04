use mysql::{OptsBuilder, Pool};
use rand::Rng;
use thiserror::Error;

mod hashing;
pub mod entities;

pub type Driver = mysql::Pool;
type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Mysql(#[from] mysql::Error),
    #[error("{0}")]
    Refinery(#[from] refinery::Error),
    #[error("Unknown enum variant")]
    UnknownEnumVariant,
    #[error("{0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("Unknown token")]
    UnknownToken,
    #[error("Token has expired")]
    ExpiredToken,
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("./migrations/");
}

pub struct DriverConfig {
    pub host: String,
    pub database: String,
    pub username: String,
    pub password: String
}

pub fn get_driver(config: DriverConfig) -> Result<Driver> {
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some(config.host))
        .db_name(Some(config.database))
        .user(Some(config.username))
        .pass(Some(config.password));
    let pool = Pool::new(opts)?;
    Ok(pool)
}

pub fn init(driver: &Driver) -> Result<()> {
    let mut conn = driver.get_conn()?;
    migrations::migrations::runner()
        .set_migration_table_name("__invoicex_migrations")
        .run(&mut conn)?;

    Ok(())
}

pub(crate) fn gen_id() -> String {
    rand::thread_rng().sample_iter(rand::distributions::Alphanumeric).take(32).map(char::from).collect()
}
