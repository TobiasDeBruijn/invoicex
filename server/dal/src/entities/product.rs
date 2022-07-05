use mysql::prelude::Queryable;
use mysql::{params, Row, Transaction, TxOpts};
use crate::{Driver, gen_id};
use crate::entities::{Entity, Org};

#[derive(Debug, Clone)]
pub struct Product<'a> {
    driver: &'a Driver,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub org_id: String,
    pub product_code: Option<String>,
    pub price_per_unit: f32,
    pub tax_percentage: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ProductBuilder<'a> {
    pub name: String,
    pub org: &'a Org<'a>,
    pub product_code: Option<String>,
    pub description: Option<String>,
    pub price_per_unit: f32,
    pub tax_percentage: Option<f32>,
}

impl<'a> Entity<'a> for Product<'a> {
    type Information = ProductBuilder<'a>;

    fn create(driver: &'a Driver, builder: Self::Information) -> crate::Result<Self> {
        let mut tx = driver.start_transaction(TxOpts::default())?;
        let id = gen_id();

        tx.exec_drop("INSERT INTO products (id, org_id, name, product_code, description, price_per_unit, tax_percentage) VALUES (:id, :org_id, :name, :product_code, :description, :price_per_unit, :tax_percentage)", params! {
            "id" => &id,
            "org_id" => &builder.org.id,
            "name" => &builder.name,
            "product_code" => &builder.product_code,
            "description" => &builder.description,
            "price_per_unit" => builder.price_per_unit,
            "tax_percentage" => builder.tax_percentage
        })?;

        tx.commit()?;

        Ok(Self {
            driver,
            id,
            name: builder.name,
            description: builder.description,
            product_code: builder.product_code,
            price_per_unit: builder.price_per_unit,
            org_id: builder.org.id.clone(),
            tax_percentage: builder.tax_percentage,
        })
    }

    fn remove(self) -> crate::Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        tx.exec_drop("DELETE FROM products WHERE id = :id", params! {
            "id" => &self.id
        })?;
        tx.commit()?;
        Ok(())
    }

    fn update(&mut self) -> crate::Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        tx.exec_drop("UPDATE products SET name = :name, description = :description, product_code = :product_code, :price_per_unit = :price_per_unit, :tax_percentage WHERE id = :id", params! {
            "name" => &self.name,
            "description" => &self.description,
            "product_code" => &self.product_code,
            "price_per_unit" => &self.price_per_unit,
            "tax_percentage" => &self.tax_percentage,
            "id" => &self.id
        })?;
        tx.commit()?;
        Ok(())
    }

    fn get(driver: &'a Driver, id: String) -> crate::Result<Option<Self>> {
        let mut tx = driver.start_transaction(TxOpts::default())?;
        let res = Self::get_with_tx(&mut tx, driver, id)?;
        tx.commit()?;
        Ok(res)
    }
}

impl<'a> Product<'a> {
    fn get_with_tx(tx: &mut Transaction, driver: &'a Driver, id: String) -> crate::Result<Option<Self>> {
        let row: Row = match tx.exec_first("SELECT name,description,org_id,product_code,price_per_unit,tax_percentage FROM products WHERE id = :id", params! {
            "id" => &id
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        Ok(Some(Self {
            driver,
            id,
            name: row.get("name").unwrap(),
            description: row.get("description").unwrap(),
            product_code: row.get("product_code").unwrap(),
            org_id: row.get("org_id").unwrap(),
            price_per_unit: row.get("price_per_unit").unwrap(),
            tax_percentage: row.get("tax_percentage").unwrap(),
        }))
    }

    pub fn list_for_org(driver: &'a Driver, org: &Org<'a>) -> crate::Result<Vec<Self>> {
        let mut tx = driver.start_transaction(TxOpts::default())?;
        let rows: Vec<Row> = tx.exec("SELECT id FROM orgs WHERE org_id = :org_id", params! {
            "org_id" => &org.id
        })?;

        let products = rows.into_iter()
            .map(|row| Ok(Self::get_with_tx(&mut tx, driver, row.get("id").unwrap())?.unwrap()))
            .collect::<crate::Result<Vec<_>>>()?;

        tx.commit()?;
        Ok(products)
    }
}