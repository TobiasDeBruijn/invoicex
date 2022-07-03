use crate::{Result, Driver};

mod user;

pub use user::*;

pub trait Entity<'a>: Sized {
    type Information;

    fn create(driver: &'a Driver, builder: Self::Information) -> Result<Self>;

    fn delete(self) -> Result<()>;

    fn update(&mut self) -> Result<()>;

    fn get(driver: &'a Driver, id: String) -> Result<Option<Self>>;
}