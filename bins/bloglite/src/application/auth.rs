use std::sync::Arc;

use chrono::{Duration, Local};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::config;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("未认证")]
    MissingToken,

    #[error("无效token")]
    InvalidToken,

    #[error("token已过期")]
    ExpiredToken,

    #[error("生成token失败")]
    GenerateTokenFailed,
    // #[error("密码错误")]
    // PasswordError,
}

// 实现lib_api::Error的类型自动实现
// impl<T: Error> From<T> for ErrorResponse
// 所以不需要显式实现
//
// 为 app::auth_error 实现 api error trait
impl lib_api::ApiError for AuthError {
    fn as_error_code(&self) -> lib_api::ErrorCode {
        match self {
            AuthError::MissingToken => lib_api::ErrorCode::MissingCredentials,
            AuthError::ExpiredToken => lib_api::ErrorCode::InvalidToken,
            AuthError::InvalidToken => lib_api::ErrorCode::InvalidCredentials,
            AuthError::GenerateTokenFailed => lib_api::ErrorCode::InternalError,
        }
    }
}

// 采用双token机制
// 访问token
#[derive(Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub exp: i64,
}

impl AccessClaims {
    const DURATION: Duration = Duration::minutes(10);
    pub fn new(sub: impl Into<String>) -> Self {
        Self {
            sub: sub.into(),
            exp: (Local::now() + Self::DURATION).timestamp(),
        }
    }
}

// 刷新token
#[derive(Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,
    pub exp: i64,
}

impl RefreshClaims {
    const DURATION: Duration = Duration::days(180);
    pub fn new(sub: impl Into<String>) -> Self {
        Self {
            sub: sub.into(),
            exp: (Local::now() + Self::DURATION).timestamp(),
        }
    }
}

pub struct JwtConfig {
    header: Header,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtConfig {
    fn new(secret: &[u8]) -> Self {
        Self {
            header: Header::default(),
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            validation: Validation::default(),
        }
    }

    pub fn sign<T: Serialize>(&self, claims: T) -> Result<String, AuthError> {
        jsonwebtoken::encode(&self.header, &claims, &self.encoding_key)
            .ok()
            .ok_or(AuthError::GenerateTokenFailed)
    }

    pub fn validate<T: DeserializeOwned>(&self, token: impl AsRef<str>) -> Result<T, AuthError> {
        let token_data =
            match jsonwebtoken::decode::<T>(token.as_ref(), &self.decoding_key, &self.validation) {
                Ok(data) => data,
                Err(error) => {
                    return Err(match error.kind() {
                        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                            AuthError::ExpiredToken
                        }
                        _ => AuthError::InvalidToken,
                    })
                }
            };

        Ok(token_data.claims)
    }
}

#[derive(Clone)]
pub struct JwtState {
    access_jwt: Arc<JwtConfig>,
    refresh_jwt: Arc<JwtConfig>,
}

impl JwtState {
    pub fn new() -> Self {
        Self {
            access_jwt: Arc::new(JwtConfig::new(
                std::env::var("ACCESS_SECRET").unwrap().as_bytes(),
            )),
            refresh_jwt: Arc::new(JwtConfig::new(
                std::env::var("REFRESH_SECRET").unwrap().as_bytes(),
            )),
        }
    }

    pub fn sign_access_token(&self) -> Result<String, AuthError> {
        self.access_jwt
            .sign(AccessClaims::new(env!("CARGO_PKG_NAME")))
    }

    pub fn validate_access_token(&self, token: impl AsRef<str>) -> Result<AccessClaims, AuthError> {
        self.access_jwt.validate(token)
    }

    pub fn validate_refresh_token(
        &self,
        token: impl AsRef<str>,
    ) -> Result<RefreshClaims, AuthError> {
        self.refresh_jwt.validate(token)
    }

    /// 将生成的 refresh token 写入 auth config
    pub fn generate_and_write_auth_config(&self) {
        let auth = self
            .refresh_jwt
            .sign(RefreshClaims::new(format!(
                "{}@{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )))
            .unwrap();

        config::write_auth_config(auth);
    }
}
