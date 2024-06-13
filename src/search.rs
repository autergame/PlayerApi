use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use indicium::simple::{SearchIndexBuilder, SearchType};
use sea_orm::DatabaseConnection;

use crate::{
    api_error::ApiResult,
    extra::Params,
    get::{get_lives, get_movies, get_series, Kind},
    login,
};

#[actix_web::get("/search/{kind}/{text}")]
pub async fn search(
    credentials: BearerAuth,
    path: ActixWeb::Path<(Kind, String)>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let (kind, mut text) = path.into_inner();
    text = urlencoding::decode(&text)?.into_owned();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let params = Params::new(&login);

    let list = match kind {
        Kind::Live => get_lives(None, params, client).await?,
        Kind::Movie => get_movies(None, params, client).await?,
        Kind::Serie => get_series(None, params, client).await?,
    };

    let mut search_index = SearchIndexBuilder::default()
        .search_type(SearchType::And)
        .exclude_keywords(None)
        .build();

    for (i, item) in list.iter().enumerate() {
        search_index.insert(&i, &item.name);
    }

    let mut result = Vec::new();

    for i in search_index.search(&text) {
        result.push(&list[*i]);
    }

    Ok(HttpResponse::Ok().json(result))
}
