use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    api_error::ApiResult,
    login::{self},
};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Live,
    Movie,
    Serie,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Live => write!(f, "live"),
            Kind::Movie => write!(f, "movie"),
            Kind::Serie => write!(f, "series"),
        }
    }
}

#[actix_web::get("/link/{kind}/{id}/{container_extension}")]
async fn link(
    credentials: BearerAuth,
    path: ActixWeb::Path<(Kind, String, String)>,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let (kind, id, container_extension) = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let url = url::Url::parse(&login.server)?;
    let base = url.origin().unicode_serialization();

    let result = format!(
        "{}/{}/{}/{}/{}.{}",
        base, kind, login.username, login.password, id, container_extension
    );

    Ok(HttpResponse::Ok().json(result))
}
