use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use indicium::simple::{SearchIndexBuilder, SearchType};
use sea_orm::DatabaseConnection;

use crate::{
    api_error::ApiResult,
    extra::Params,
    get::{get_lives, get_movies, get_series, Value},
    login,
};

#[actix_web::get("/live/{text}")]
async fn live(
    credentials: BearerAuth,
    path: ActixWeb::Path<String>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let mut text = path.into_inner();
    text = urlencoding::decode(&text)?.into_owned();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let list = get_lives(&None, Params::new(&login), client).await?;

    let result = search(&list, text);

    Ok(HttpResponse::Ok().json(result))
}

#[actix_web::get("/movie/{text}")]
async fn movie(
    credentials: BearerAuth,
    path: ActixWeb::Path<String>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let mut text = path.into_inner();
    text = urlencoding::decode(&text)?.into_owned();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let list = get_movies(&None, Params::new(&login), client).await?;

    let result = search(&list, text);

    Ok(HttpResponse::Ok().json(result))
}

#[actix_web::get("/serie/{text}")]
async fn serie(
    credentials: BearerAuth,
    path: ActixWeb::Path<String>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let mut text = path.into_inner();
    text = urlencoding::decode(&text)?.into_owned();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let list = get_series(&None, Params::new(&login), client).await?;

    let result = search(&list, text);

    Ok(HttpResponse::Ok().json(result))
}

pub fn search(list: &[Value], text: String) -> Vec<&Value> {
    let mut search_index = SearchIndexBuilder::default()
        .search_type(SearchType::And)
        .exclude_keywords(None)
        .build();

    for (i, item) in list.iter().enumerate() {
        if let Some(name) = &item.name {
            search_index.insert(&i, name);
        }
    }

    let mut result = Vec::new();
    for i in search_index.search(&text) {
        result.push(&list[*i]);
    }
    result
}
