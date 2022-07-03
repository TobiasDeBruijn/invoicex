use actix_multiresponse::Payload;
use dal::entities::User;
use proto::{AuthenticationMethod, LoginRequest, LoginResponse};
use proto::login_request::Authentication;
use crate::error::{Error, WebResult};
use crate::WebData;

pub async fn login(data: WebData, payload: Payload<LoginRequest>) -> WebResult<Payload<LoginResponse>> {
    let mut user = User::get_by_email(&data.driver, &payload.email)?.ok_or(Error::Unauthorized(String::default()))?;

    let method = AuthenticationMethod::from_i32(payload.authentication_method).ok_or(Error::BadRequest("Invalid authentication method".to_string()))?;
    let authentication = match &payload.authentication {
        Some(x) => x,
        None => return Err(Error::BadRequest("Missing authentication".to_string()))
    };

    match method {
        AuthenticationMethod::Password => {
            match authentication {
                Authentication::Password(password) => {
                    if !user.is_password_correct(password, &data.config.password_pepper)? {
                        return Err(Error::Unauthorized(String::default()));
                    }
                },
            }
        }
    }

    let session = user.create_session()?;

    Ok(Payload(LoginResponse {
        session: Some(proto::Session {
            id: session.id,
            expires_at: session.expires_at,
            last_used: session.last_used
        }),
        user: Some(proto::User {
            id: user.id,
            name: user.name,
            email: user.email
        })
    }))
}
