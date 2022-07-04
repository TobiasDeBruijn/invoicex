use std::future::Future;
use std::pin::Pin;
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use dal::Driver;
use dal::entities::{Entity, User};
use crate::error::{Error, WebResult};
use crate::WebData;

pub struct Session {
    pub id: String,
    pub user_id: String,
}

impl Session {
    pub fn user<'a>(&self, driver: &'a Driver) -> WebResult<User<'a>> {
        Ok(User::get(driver, self.id.clone())?.unwrap())
    }
}

impl FromRequest for Session {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let data: &WebData = req.app_data().unwrap();
            let authorization = req.headers().get("authorization")
                .ok_or(Error::Unauthorized("Missing Authorization header".to_string()))?
                .to_str().map_err(|e| Error::Unauthorized(format!("Invalid Authorization header: {e}")))?;

            if authorization.starts_with("US_") {
                let user = User::get_by_session(&data.driver, &authorization.to_string())?.ok_or(Error::Unauthorized("Session does not exist or has expired".to_string()))?;

                Ok(Self {
                    id: authorization.to_string(),
                    user_id: user.id.clone()
                })
            } else {
                todo!("Service tokens")
            }
        })
    }
}

