use std::str::FromStr;
use mysql::prelude::Queryable;
use mysql::{params, Params, Row, Transaction, TxOpts};
use crate::{Driver, Error, gen_id};
use crate::entities::{Entity, User};
use proc::{Stringify, Variants, ScopeList};

#[derive(Debug, Clone)]
pub struct Org<'a> {
    driver: &'a Driver,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct OrgBuilder<'a> {
    pub name: String,
    pub creator: &'a User<'a>
}

#[derive(Debug, Clone, PartialEq, Eq, Stringify, Variants, ScopeList)]
pub enum OrgScope {
    /// Allows the user to add a user to the org
    #[admin]
    AddUser,
    /// Allows the user to view the organization
    GetOrg,
    /// ALlows the user to delete the organization
    DeleteOrg,
    /// Allows the user to update the organization
    UpdateOrg,
}

#[derive(Debug, Clone)]
pub struct OrgUserLink<'a> {
    pub org: Org<'a>,
    pub user: User<'a>
}

impl<'a> Entity<'a> for Org<'a> {
    type Information = OrgBuilder<'a>;

    fn create(driver: &'a Driver, builder: Self::Information) -> crate::Result<Self> {
        let mut tx = driver.start_transaction(TxOpts::default())?;
        let id = gen_id();

        tx.exec_drop("INSERT INTO orgs (id, name, created_at) VALUES (:id, :name, :created_at)", params! {
            "id" => &id,
            "name" => &builder.name,
            "created_at" => time::OffsetDateTime::now_utc().unix_timestamp()
        })?;

        tx.exec_drop("INSERT INTO org_user_links (org_id, user_id, org_admin) VALUES (:org_id, :user_id, true)", params! {
            "org_id" => &id,
            "user_id" => &builder.creator.id
        })?;

        for scope in OrgScope::variants() {
            let scope: &OrgScope = scope;
            tx.exec_drop("INSERT INTO org_user_link_scopes (org_id, user_id, scope_name) VALUES (:org_id, :user_id, :scope_name)", params! {
                "org_id" => &id,
                "user_id" => &builder.creator.id,
                "scope_name" => &scope.to_string()
            })?;
        }

        tx.commit()?;

        Ok(Self {
            driver,
            id,
            name: builder.name
        })
    }

    fn remove(self) -> crate::Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        tx.exec_drop("DELETE FROM org_user_links WHERE org_id = :org_id", params! {
            "org_id" => &self.id
        })?;

        tx.exec_drop("DELETE FROM org_user_link_scopes WHERE org_id = :org_id", params! {
            "org_id" => &self.id
        })?;

        tx.exec_drop("DELETE FROM orgs WHERE id = :id", params! {
            "id" => &self.id
        })?;

        tx.commit()?;

        Ok(())
    }

    fn update(&mut self) -> crate::Result<()> {
        let mut conn = self.driver.get_conn()?;
        conn.exec_drop("UPDATE orgs SET name = :name WHERE id = :id", params! {
            "id" => &self.id,
            "name" => &self.name
        })?;

        Ok(())
    }

    fn get(driver: &'a Driver, id: String) -> crate::Result<Option<Self>> {
        let mut conn = driver.get_conn()?;
        let row: Row = match conn.exec_first("SELECT name FROM orgs WHERE id = :id", params! {
            "id" => &id
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        Ok(Some(Self {
            driver,
            id,
            name: row.get("name").unwrap(),
        }))
    }
}

/// A user who is part of an organization
pub struct OrgUser<'a> {
    /// The user itself
    pub user: User<'a>,
    /// Whether the user is an organization admin.
    /// When this is `true`, it does not matter which scopes
    /// the user has, they can do anything within the organization.
    pub is_org_admin: bool,
    /// The scopes that have been granted to the user
    pub scopes: Vec<OrgScope>,
}

impl<'a> Org<'a> {
    /// List all known orgs
    pub fn list_available(driver: &'a Driver) -> crate::Result<Vec<Self>> {
        let mut conn = driver.get_conn()?;
        let rows: Vec<Row> = conn.exec("SELECT id FROM orgs", Params::Empty)?;
        let orgs = rows.into_iter()
            .map(|x| Ok(Org::get(driver, x.get("id").unwrap())?.unwrap()))
            .collect::<crate::Result<Vec<_>>>()?;
        Ok(orgs)
    }

    /// Add a user to the organization. If `admin` is set to `true`, all scopes will be granted.
    /// If `admin` is set to false, only non-admin scopes will be granted.
    pub fn add_user(&mut self, user: &User<'_>, admin: bool) -> crate::Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        tx.exec_drop("INSERT INTO org_user_links (:org_id, :user_id, :org_admin) VALUES (:org_id, :user_id, :org_admin)", params! {
            "org_id" => &self.id,
            "user_id" => &user.id,
            "org_admin" => admin,
        })?;

        for scope in OrgScope::default_scopes() {
            let scope: &OrgScope = scope;
            self.set_scope_with_tx(&mut tx, user, scope, true)?;
        }

        if admin {
            for scope in OrgScope::admin_scopes() {
                let scope: &OrgScope = scope;
                self.set_scope_with_tx(&mut tx, user, scope, true)?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// Remove a user from the organization.
    /// If the user is the last user, the organization is **not** automatically deleted, this is up to the callee
    pub fn remove_user(&mut self, user: &User<'_>) -> crate::Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        tx.exec_drop("DELETE FROM org_user_link_scopes WHERE org_id = :org_id AND user_id = :user_id", params! {
            "org_id" => &self.id,
            "user_id" => &user.id
        })?;

        tx.exec_drop("DELETE FROM org_user_links WHERE org_id = :org_id AND user_id = :user_id", params! {
            "org_id" => &self.id,
            "user_id" => &user.id
        })?;

        tx.commit()?;
        Ok(())
    }

    /// Set a scope using the provided transaction
    fn set_scope_with_tx(&mut self, tx: &mut Transaction, user: &User<'_>, scope: &OrgScope, enabled: bool) -> crate::Result<()> {
        if enabled {
            tx.exec_drop("INSERT INTO org_user_link_scopes (org_id, user_id, scope_name) VALUES (:org_id, :user_id, :scope_name)", params! {
                "org_id" => &self.id,
                "user_id" => &user.id,
                "scope_name" => &scope.to_string()
            })?;
        } else {
            tx.exec_drop("DELETE FROM org_user_link_scopes WHERE user_id = :user_id AND org_id = :org_id", params! {
                "user_id" => &user.id,
                "org_id" => &self.id,
            })?;
        }

        Ok(())
    }

    /// Enable or disable a scope on a user for this organization
    pub fn set_scope(&mut self, user: &User<'_>, scope: &OrgScope, enabled: bool) -> crate::Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        self.set_scope_with_tx(&mut tx, user, scope, enabled)?;
        tx.commit()?;
        Ok(())
    }

    /// List all users in the organization
    pub fn list_users(&self) -> crate::Result<Vec<OrgUser<'a>>> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        let rows: Vec<Row> = tx.exec("SELECT user_id,org_admin FROM org_user_links WHERE org_id = :org_id", params! {
            "org_id" => &self.id
        })?;

        let users = rows.into_iter()
            .map(|x| {
                let user_id: String = x.get("id").unwrap();
                let org_admin: bool = x.get("org_admin").unwrap();

                let user = User::get(&self.driver, user_id.clone())?.ok_or_else(|| Error::InvalidState(format!("User {user_id} is linked to organization {}, but does not exist", &self.id)))?;
                let scopes = self.list_scopes_with_tx(&mut tx, &user)?;
                Ok(OrgUser {
                    user,
                    is_org_admin: org_admin,
                    scopes
                })
            })
            .collect::<crate::Result<Vec<_>>>()?;
        Ok(users)
    }

    /// List the scopes a user has in the organization using the provided transaction
    fn list_scopes_with_tx(&self, tx: &mut Transaction, user: &User<'_>) -> crate::Result<Vec<OrgScope>> {
        let rows: Vec<Row> = tx.exec("SELECT scope_name FROM org_user_link_scopes WHERE org_id = :org_id AND user_id = :user_id", params! {
            "org_id" => &self.id,
            "user_id" => &user.id,
        })?;

        let scopes = rows.into_iter()
            .map(|x| {
                let scope_name: String = x.get("scope_name").unwrap();
                OrgScope::from_str(&scope_name).map_err(|_| Error::UnknownEnumVariant)
            })
            .collect::<crate::Result<Vec<_>>>()?;
        Ok(scopes)
    }

    /// List all scopes a user has in the organization
    pub fn list_scopes(&self, user: &User<'_>) -> crate::Result<Vec<OrgScope>> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        let scopes = self.list_scopes_with_tx(&mut tx, user)?;
        tx.commit()?;
        Ok(scopes)
    }
}