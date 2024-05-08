use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
//use utoipa::ToSchema;

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum ApiError {
    RequestServerError,
    AccountNotFound,
    NotFound,

    WrongEpisodeId,
    WrongAuthKey,
    WrongAvatar,
    WrongId,

    SystemTime,
    DataBase,
    ParseInt,
    UrlParse,
    FromUtf8,
    Reqwest,
    Channel,
    Poison,
    Serde,
    OsRng,
    Io,
}

#[derive(/*ToSchema,*/ Serialize)]
struct ApiErrorJson {
    //#[schema(example = "0")]
    error: u8,
    //#[schema(example = "AccountNotFound")]
    hint: String,
}

impl From<ApiError> for ApiErrorJson {
    fn from(err: ApiError) -> ApiErrorJson {
        ApiErrorJson {
            error: err as u8,
            hint: format!("{:?}", err),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for ApiError {
    fn from(_: std::sync::PoisonError<T>) -> ApiError {
        ApiError::Poison
    }
}

impl From<std::time::SystemTimeError> for ApiError {
    fn from(_: std::time::SystemTimeError) -> ApiError {
        ApiError::SystemTime
    }
}

impl From<rand::Error> for ApiError {
    fn from(_: rand::Error) -> ApiError {
        ApiError::OsRng
    }
}

impl From<url::ParseError> for ApiError {
    fn from(_: url::ParseError) -> ApiError {
        ApiError::UrlParse
    }
}

impl From<std::string::FromUtf8Error> for ApiError {
    fn from(_: std::string::FromUtf8Error) -> ApiError {
        ApiError::FromUtf8
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(_: reqwest::Error) -> ApiError {
        ApiError::Reqwest
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(_: serde_json::Error) -> ApiError {
        ApiError::Serde
    }
}

impl From<std::io::Error> for ApiError {
    fn from(_: std::io::Error) -> ApiError {
        ApiError::Io
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(_: sea_orm::DbErr) -> ApiError {
        ApiError::DataBase
    }
}

impl From<std::num::ParseIntError> for ApiError {
    fn from(_: std::num::ParseIntError) -> ApiError {
        ApiError::ParseInt
    }
}

impl From<tokio::sync::oneshot::error::RecvError> for ApiError {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> ApiError {
        ApiError::Channel
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::NotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().json(ApiErrorJson::from(*self)),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

pub trait ErrAs<T> {
    fn err_as<O>(self, op: O) -> Result<T, O>;
}

impl<T, E> ErrAs<T> for Result<T, E> {
    fn err_as<O>(self, op: O) -> Result<T, O> {
        match self {
            Ok(t) => Ok(t),
            Err(_) => Err(op),
        }
    }
}
