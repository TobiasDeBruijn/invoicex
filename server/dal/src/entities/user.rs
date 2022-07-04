use mysql::{params, Row, TxOpts};
use mysql::prelude::Queryable;
use crate::{Driver, Error, gen_id, Result};
use crate::entities::Entity;
use proc::Stringify;
use std::str::FromStr;
use rand::RngCore;
use crate::hashing::{hash, verify};

#[derive(Debug, Clone)]
pub struct User<'a> {
    driver: &'a Driver,
    pub id: String,
    pub name: String,
    pub email: String
}

pub struct UserBuilder {
    pub name: String,
    pub email: String,
    pub authentication: Authentication,
}

impl<'a> Entity<'a> for User<'a> {
    type Information = UserBuilder;

    fn create(driver: &'a Driver, builder: Self::Information) -> Result<Self> {
        let id = gen_id();

        let mut tx = driver.start_transaction(TxOpts::default())?;
        tx.exec_drop("INSERT INTO users (id, name, email) VALUES (:id, :name, :email) VALUES (:id, :name, :email)", params! {
            "id" => &id,
            "name" => &builder.name,
            "email" => &builder.email
        })?;

        let auth_method = AuthenticationMethod::from(&builder.authentication);
        tx.exec_drop("INSERT INTO user_authentication_methods (id, method) VALUES (:id, :method)", params! {
            "id" => &id,
            "method" => auth_method.to_string(),
        })?;

        match builder.authentication {
            Authentication::Password { password, pepper } => {
                let mut salt = [0u8; 16];
                rand::thread_rng().fill_bytes(&mut salt);

                let hash = hash(&password, salt, &pepper)?;

                tx.exec_drop("INSERT INTO user_passwords (id, hash) VALUES (:id, :hash)", params! {
                    "id" => &id,
                    "hash" => &hash,
                })?;
            }
        }

        tx.commit()?;

        Ok(Self {
            driver,
            id,
            name: builder.name,
            email: builder.email,
        })
    }

    fn delete(self) -> Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;

        tx.exec_drop("DELETE FROM users WHERE id = :id", params! {
            "id" => &self.id
        })?;

        tx.exec_drop("DELETE FROM user_passwords WHERE id = :id", params! {
            "id" => &self.id
        })?;

        tx.exec_drop("DELETE FROM user_authentication_methods WHERE id = :id", params! {
            "id" => &self.id
        })?;

        tx.exec_drop("DELETE FROM user_sessions WHERE id = :id", params! {
            "id" => &self.id,
        })?;

        tx.exec_drop("DELETE FROM service_tokens WHERE associated_user_id = :id", params! {
            "id" => &self.id
        })?;

        tx.commit()?;
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        let mut conn = self.driver.get_conn()?;
        conn.exec_drop("UPDATE users SET email = :email, name = :name WHERE id = :id", params! {
            "email" => &self.email,
            "name" => &self.name,
            "id" => &self.id
        })?;

        Ok(())
    }

    fn get(driver: &'a Driver, id: String) -> Result<Option<Self>> {
        let mut conn = driver.get_conn()?;
        let row: Row = match conn.exec_first("SELECT email,name FROM users WHERE id = :id", params! {
            "id" => &id
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        Ok(Some(Self {
            driver,
            id,
            name: row.get("name").unwrap(),
            email: row.get("email").unwrap()
        }))
    }
}

#[derive(Debug, Stringify)]
pub enum AuthenticationMethod {
    Password,
}

pub enum Authentication {
    Password {
        password: String,
        pepper: String,
    },
}

impl From<&Authentication> for AuthenticationMethod {
    fn from(x: &Authentication) -> Self {
        match x {
            Authentication::Password { .. } => Self::Password,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionDescription {
    pub id: String,
    pub expires_at: i64,
    pub last_used: i64,
}

#[derive(Debug, Clone)]
pub struct EmailAssociation {
    pub verification_token: String,
    pub expires_at: i64,
}

impl<'a> User<'a> {
    pub fn get_by_email(driver: &'a Driver, email: &str) -> Result<Option<Self>> {
        let mut conn = driver.get_conn()?;
        let row: Row = match conn.exec_first("SELECT id,name FROM users WHERE email = :email", params! {
            "email" => email,
        })? {
            Some(x) => x,
            None => return Ok(None),
        };

        Ok(Some(Self {
            driver,
            id: row.get("id").unwrap(),
            name: row.get("name").unwrap(),
            email: email.to_string(),
        }))
    }

    pub fn delete_session(&mut self, session: &str) -> Result<()> {
        let mut conn = self.driver.get_conn()?;
        conn.exec_drop("DELETE FROM user_sessions WHERE id = :id", params! {
            "id" => &session
        })?;
        Ok(())
    }

    pub fn is_password_correct(&self, provided_password: &str, pepper: &str) -> Result<bool> {
        let mut conn = self.driver.get_conn()?;
        let row: Row = match conn.exec_first("SELECT hash FROM user_passwords WHERE id = :id", params! {
            "id" => &self.id
        })? {
            Some(x) => x,
            None => return Ok(false),
        };

        let existing_hash: String = row.get("hash").unwrap();
        let is_valid = verify(&existing_hash, provided_password, pepper)?;
        Ok(is_valid)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionDescription>> {
        let mut conn = self.driver.get_conn()?;
        let rows: Vec<Row> = conn.exec("SELECT id,expires_at,last_used FROM user_sessions WHERE user_id = :user_id", params! {
            "user_id" => &self.id
        })?;

        let result = rows.into_iter()
            .map(|x| SessionDescription {
                id: x.get("id").unwrap(),
                expires_at: x.get("expires_at").unwrap(),
                last_used: x.get("last_used").unwrap(),
            })
            .collect::<Vec<_>>();
        Ok(result)
    }

    pub fn get_by_session(driver: &'a Driver, session: &str) -> Result<Option<Self>> {
        let mut conn = driver.get_conn()?;
        let row: Row = match conn.exec_first("SELECT user_id,expires_at FROM user_sessions WHERE id = :id", params! {
            "id" => &session,
        })? {
            Some(x) => x,
            None => return Ok(None),
        };

        let user_id: String = row.get("user_id").unwrap();
        let expires_at: i64 = row.get("expires_at").unwrap();

        if time::OffsetDateTime::now_utc().unix_timestamp() > expires_at {
            conn.exec_drop("DELETE FROM user_sessions WHERE id = :id", params! {
                "id" => &session
            })?;

            return Ok(None);
        }

        conn.exec_drop("UPDATE user_sessions SET last_used = :last_used WHERE id = :id", params! {
            "id" => &session,
            "last_used" => time::OffsetDateTime::now_utc().unix_timestamp(),
        })?;

        let user = Self::get(driver, user_id)?;
        Ok(user)
    }

    pub fn is_email_verified(&self, email: &str) -> Result<bool> {
        let mut conn = self.driver.get_conn()?;
        let row: Row = match conn.exec_first("SELECT verified FROM user_emails WHERE email = :email AND user_id = :user_id", params! {
            "email" => &email,
            "user_id" => &self.id
        })? {
            Some(x) => x,
            None => return Ok(false)
        };

        let verified: bool = row.get("verified").unwrap();
        Ok(verified)
    }

    pub fn verifiy_email(&mut self, verification_token: &str) -> Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        let row: Row = match tx.exec_first("SELECT user_id,expires_at,email FROM user_email_verification_tokens WHERE token = :token", params! {
            "token" => verification_token
        })? {
            Some(x) => x,
            None => return Err(Error::UnknownToken),
        };

        let stored_user_id: String = row.get("user_id").unwrap();
        let expires_at: i64 = row.get("expires_at").unwrap();

        if stored_user_id.ne(&self.id) {
            return Err(Error::UnknownToken);
        }

        if time::OffsetDateTime::now_utc().unix_timestamp() > expires_at {
            return Err(Error::ExpiredToken);
        }

        // Token is now known to be valid. Next step is to activate the Email
        let email: String = row.get("email").unwrap();
        tx.exec_drop("UPDATE user_emails SET verified = true WHERE email = :email AND user_id = :user_id", params! {
            "email" => &email,
            "user_id" => &self.id
        })?;

        tx.exec_drop("UPDATE users SET email = :email WHERE id = :id", params! {
            "id" => &self.id,
            "email" => &email
        })?;

        // Email is now active. Next step is cleanup
        tx.exec_drop("DELETE FROM user_email_verification_tokens WHERE token = :token", params! {
            "token" => verification_token
        })?;

        tx.commit()?;

        Ok(())
    }

    pub fn associate_email(&mut self, email: &str) -> Result<EmailAssociation> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;
        tx.exec_drop("INSERT INTO user_emails (email, user_id) VALUES (:email, :user_id)", params! {
            "email" => email,
            "user_id" => &self.id
        })?;

        let expires_at = (time::OffsetDateTime::now_utc() + time::Duration::days(7)).unix_timestamp();
        let token = gen_id();

        tx.exec_drop("INSERT INTO user_email_verification_tokens (token, email, user_id, expires_at) VALUES (:token, :email, :user_id, :expires_at)", params! {
            "token" => &token,
            "email" => email,
            "user_id" => &self.id,
            "expires_at" => expires_at
        })?;

        tx.commit()?;

        Ok(EmailAssociation {
            verification_token: token,
            expires_at,
        })
    }

    pub fn create_session(&mut self) -> Result<SessionDescription> {
        let id = gen_id();
        // A user session ID is prefixed with US_, add the prefix
        let id = format!("US_{id}");

        let mut conn = self.driver.get_conn()?;

        let expires_at = (time::OffsetDateTime::now_utc() + time::Duration::days(30)).unix_timestamp();
        let last_used = time::OffsetDateTime::now_utc().unix_timestamp();

        conn.exec_drop("INSERT INTO user_sessions (id, user_id, last_used, expires_at)", params! {
            "id" => &id,
            "user_id" => &self.id,
            "last_used" => last_used,
            "expires_at" => expires_at
        })?;

        Ok(SessionDescription {
            id,
            expires_at,
            last_used
        })
    }

    pub fn get_authentication_method(&self) -> Result<Option<AuthenticationMethod>> {
        let mut conn = self.driver.get_conn()?;
        let row: Row = match conn.exec_first("SELECT method FROM user_authentication_methods WHERE id = :id", params! {
            "id" => &self.id
        })? {
            Some(x) => x,
            None => return Ok(None)
        };

        Ok(Some(AuthenticationMethod::from_str(
            &row.get::<String, &str>("method").unwrap()
        ).map_err(|_| Error::UnknownEnumVariant)?))
    }

    pub fn set_authentication(&mut self, auth: Authentication) -> Result<()> {
        let mut tx = self.driver.start_transaction(TxOpts::default())?;

        tx.exec_drop("UPDATE user_authentication_methods SET method = :method WHERE id = :id", params! {
            "method" => AuthenticationMethod::from(&auth).to_string(),
            "id" => &self.id
        })?;

        match auth {
            Authentication::Password { password, pepper } => {
                let mut salt = [0u8; 16];
                rand::thread_rng().fill_bytes(&mut salt);
                let hash = hash(&password, salt, &pepper)?;

                tx.exec_drop("UPDATE user_passwords SET hash = :hash WHERE id = :id", params! {
                    "hash" => &hash,
                    "id" => &self.id
                })?;
            }
        }

        Ok(())
    }
}