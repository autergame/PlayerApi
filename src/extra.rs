use actix_web::web as ActixWeb;
use api_error::{ApiError, ApiResult, ErrAs};
use entities::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{api_error, entities};

#[derive(Serialize, Debug)]
pub enum IdType {
    #[serde(rename(serialize = "stream_id"))]
    Live(i64),

    #[serde(rename(serialize = "vod_id"))]
    Movie(i64),

    #[serde(rename(serialize = "series_id"))]
    Serie(i64),

    #[serde(rename(serialize = "category_id"))]
    Category(i64),
}

#[derive(Serialize, Debug)]
pub struct Params<'a> {
    #[serde(flatten)]
    pub login: &'a Login,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'a str>,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdType>,
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
        println!();
    }

    let request = request
        .error_for_status()
        .err_as(ApiError::RequestServerError)?;
    let response = request.text().await?;

    Ok(serde_json::from_str::<T>(&response)?)
}

pub fn get_days_ago(days: i64) -> i64 {
    chrono::Utc::now().timestamp() - (days * 24 * 60 * 60)
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

pub fn default_on_null<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Default,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

pub fn num_from_str_or_num<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr + Default,
    T::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Number(number) => {
            if !number.as_str().is_empty() {
                number.as_str().parse().map_err(serde::de::Error::custom)
            } else {
                Ok(T::default())
            }
        }
        serde_json::Value::String(string) => {
            if !string.is_empty() {
                string.parse().map_err(serde::de::Error::custom)
            } else {
                Ok(T::default())
            }
        }
        serde_json::Value::Null => Ok(T::default()),
        _ => Err(serde::de::Error::custom("Not a number or string")),
    }
}

// pub fn opt_num_from_str_or_num<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
// where
//     D: serde::Deserializer<'de>,
//     T: std::str::FromStr + serde::Deserialize<'de>,
// 	<T as std::str::FromStr>::Err: std::fmt::Display + std::fmt::Debug,
// {
//     Ok(Some(num_from_str_or_num(deserializer)?))
// }
