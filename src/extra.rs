use actix_web::web as ActixWeb;
use api_error::{ApiError, ApiResult, ErrAs};
use entities::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{api_error, entities};

#[derive(Serialize, Debug)]
pub enum IdType<'a> {
    #[serde(rename(serialize = "stream_id"))]
    Live(&'a str),

    #[serde(rename(serialize = "vod_id"))]
    Movie(&'a str),

    #[serde(rename(serialize = "series_id"))]
    Serie(&'a str),

    #[serde(rename(serialize = "category_id"))]
    Category(&'a str),
}

#[derive(Serialize, Debug)]
pub struct Params<'a> {
    #[serde(flatten)]
    pub login: &'a Login,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'a str>,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType<'a>>,
}

impl<'a> Params<'a> {
    pub fn new(login: &Login) -> Params {
        Params {
            login,
            action: None,
            id: None,
        }
    }
}

pub async fn get_json<T>(
    params: &Params<'_>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let request = client
        .get(&params.login.server)
        .query(params)
        .send()
        .await?;

	if cfg!(debug_assertions) {
    	println!("get_json: {:?}", request);
	}

    let request = request
        .error_for_status()
        .err_as(ApiError::RequestServerError)?;
    let response = request.text().await?;

    Ok(serde_json::from_str::<T>(&response)?)
}

pub fn get_month_ago() -> ApiResult<i64> {
    Ok((std::time::UNIX_EPOCH.elapsed()?.as_secs() - (30 * 24 * 60 * 60)) as i64)
}

pub trait CustomDeref<T> {
    fn deref_or_else<E>(&self, err: E) -> Result<&T, E>;

    fn deref_map<'a, U, F>(&'a self, f: F) -> Option<U>
    where
        T: std::ops::Deref,
        F: FnOnce(&'a T::Target) -> U,
        <T as std::ops::Deref>::Target: 'a;
}

impl<T> CustomDeref<T> for Option<T> {
    fn deref_or_else<E>(&self, err: E) -> Result<&T, E> {
        self.as_ref().ok_or(err)
    }

    fn deref_map<'a, U, F>(&'a self, f: F) -> Option<U>
    where
        T: std::ops::Deref,
        F: FnOnce(&'a T::Target) -> U,
        <T as std::ops::Deref>::Target: 'a,
    {
        self.as_ref().map(|v| f(v.deref()))
    }
}

pub fn opt_num_from_str_or_num<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: std::str::FromStr,
    D: serde::Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(number) => Ok(T::from_str(number.as_str()).ok()),
        serde_json::Value::String(string) => Ok(T::from_str(&string).ok()),
        _ => Ok(None),
    }
}
