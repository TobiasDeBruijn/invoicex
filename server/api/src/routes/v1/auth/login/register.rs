use actix_multiresponse::Payload;
use dal::entities::{Entity, User, UserBuilder};
use proto::{AuthenticationMethod, RegisterRequest, RegisterResponse};
use proto::register_request::Authentication;
use crate::error::{Error, WebResult};
use crate::WebData;

pub async fn register(data: WebData, payload: Payload<RegisterRequest>) -> WebResult<Payload<RegisterResponse>> {
    let auth_method = AuthenticationMethod::from_i32(payload.authentication_method).ok_or(Error::BadRequest("Invalid authentication method".to_string()))?;
    let auth = match &payload.authentication {
        Some(x) => x,
        None => return Err(Error::BadRequest("Missing authentication".to_string()))
    };

    let user_auth = match auth_method {
        AuthenticationMethod::Password => {
            match auth {
                Authentication::Password(x) => dal::entities::Authentication::Password {
                    password: x.clone(),
                    pepper: data.config.password_pepper.clone()
                },
            }
        }
    };

    let mut user = User::create(&data.driver, UserBuilder {
        name: payload.name.clone(),
        email: payload.email.clone(),
        authentication: user_auth,
    })?;

    let _association = user.associate_email(&payload.email)?;

    // TODO send an email

    Ok(Payload(RegisterResponse {
        user: Some(proto::User {
            id: user.id,
            name: user.name,
            email: user.email
        }),
    }))
}
