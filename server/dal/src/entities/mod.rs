use crate::{Result, Driver};

mod user;
mod org;
mod product;

pub use user::*;
pub use org::*;
pub use product::*;

pub trait Entity<'a>: Sized {
    type Information;

    fn create(driver: &'a Driver, builder: Self::Information) -> Result<Self>;

    fn remove(self) -> Result<()>;

    fn update(&mut self) -> Result<()>;

    fn get(driver: &'a Driver, id: String) -> Result<Option<Self>>;
}