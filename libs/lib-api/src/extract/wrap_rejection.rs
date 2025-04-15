use axum::extract::{FromRequest, Request};

use crate::ApiError;

#[derive(Debug)]
pub struct WrapRejection<T>(pub T);

impl<S, T> FromRequest<S> for WrapRejection<T>
where
    T: FromRequest<S>,
    T::Rejection: ApiError,
    S: Send + Sync,
{
    type Rejection = crate::ErrorResponse;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(T::from_request(req, state).await.map(Self)?)
    }
}

impl<T> std::ops::Deref for WrapRejection<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for WrapRejection<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

macro_rules! as_error_code {
    ($name:ty, $error_code:expr) => {
        impl crate::ApiError for $name {
            fn as_error_code(&self) -> crate::ErrorCode {
                $error_code
            }
        }
    };
}

as_error_code!(
    axum::extract::multipart::MultipartRejection,
    crate::ErrorCode::InvalidInput
);

as_error_code!(
    axum::extract::rejection::JsonRejection,
    crate::ErrorCode::InvalidInput
);
