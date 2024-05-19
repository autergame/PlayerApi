use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    api_error::ApiResult,
    extra::{get_json, opt_num_from_str_or_num, CustomDeref, IdType, Params},
    login,
};

#[derive(Serialize)]
#[serde(untagged)]
enum ResultInfo {
    Live(Vec<Epg>),
    Movie(Box<MovieInfo>),
    Serie(Box<SerieInfo>),
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Live,
    Movie,
    Serie,
}

#[derive(Deserialize)]
struct Get {
    kind: Kind,
    category_id: Option<String>,
}

#[actix_web::routes]
#[get("/all/{kind}")]
#[get("/category/{kind}/{category_id}")]
async fn get(
    credentials: BearerAuth,
    path: ActixWeb::Path<Get>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let get = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let params = Params::new(&login);

    let result = match get.kind {
        Kind::Live => get_lives(&get.category_id, params, client).await?,
        Kind::Movie => get_movies(&get.category_id, params, client).await?,
        Kind::Serie => get_series(&get.category_id, params, client).await?,
    };

    Ok(HttpResponse::Ok().json(result))
}

#[actix_web::get("/info/{kind}/{id}")]
async fn info<'a>(
    credentials: BearerAuth,
    path: ActixWeb::Path<(Kind, String)>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let (kind, id) = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let params = Params::new(&login);

    let result = match kind {
        Kind::Live => ResultInfo::Live(get_live_epg(&id, params, client).await?),
        Kind::Movie => ResultInfo::Movie(Box::new(get_movie_info(&id, params, client).await?)),
        Kind::Serie => ResultInfo::Serie(Box::new(get_serie_info(&id, params, client).await?)),
    };

    Ok(HttpResponse::Ok().json(result))
}

#[actix_web::get("/categories/{kind}")]
async fn categories<'a>(
    credentials: BearerAuth,
    path: ActixWeb::Path<Kind>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let kind = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;
    let login = login::get_login(&session, &db).await?;

    let params = Params::new(&login);

    let result = get_categories(&kind, params, client).await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn get_lives<'a>(
    category_id: &'a Option<String>,
    mut params: Params<'a>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<Vec<Value>> {
    params.action = Some("get_live_streams");
    params.id = category_id.deref_map(IdType::Category);

    get_json(&params, client).await
}

#[derive(Serialize, Deserialize)]
struct Epg {
    title: Option<String>,
    description: Option<String>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    start_timestamp: Option<i64>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    stop_timestamp: Option<i64>,
}

#[derive(Serialize, Deserialize)]
struct EpgListings {
    epg_listings: Vec<Epg>,
}

async fn get_live_epg<'a>(
    id: &'a str,
    mut params: Params<'a>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<Vec<Epg>> {
    params.action = Some("get_short_epg");
    params.id = Some(IdType::Live(id));

    Ok(get_json::<EpgListings>(&params, client).await?.epg_listings)
}

pub async fn get_movies<'a>(
    category_id: &'a Option<String>,
    mut params: Params<'a>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<Vec<Value>> {
    params.action = Some("get_vod_streams");
    params.id = category_id.deref_map(IdType::Category);

    get_json(&params, client).await
}

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    name: Option<String>,

    plot: Option<String>,
    cast: Option<String>,
    genre: Option<String>,
    duration: Option<String>,
    director: Option<String>,
    release_date: Option<String>,
    youtube_trailer: Option<String>,

    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    rating: Option<f64>,

    #[serde(alias = "movie_image")]
    cover: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct MovieData {
    #[serde(alias = "stream_id")]
    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    id: Option<i64>,

    name: Option<String>,
    container_extension: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MovieInfo {
    info: Info,

    #[serde(alias = "movie_data")]
    data: MovieData,
}

pub async fn get_movie_info<'a>(
    id: &'a str,
    mut params: Params<'a>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<MovieInfo> {
    params.action = Some("get_vod_info");
    params.id = Some(IdType::Movie(id));

    get_json(&params, client).await
}

pub async fn get_series<'a>(
    category_id: &'a Option<String>,
    mut params: Params<'a>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<Vec<Value>> {
    params.action = Some("get_series");
    params.id = category_id.deref_map(IdType::Category);

    get_json(&params, client).await
}

#[derive(Debug, Serialize, Deserialize)]
struct Season {
    poster_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EpisodeInfo {
    #[serde(alias = "movie_image")]
    image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub id: Option<i64>,

    info: EpisodeInfo,
    title: Option<String>,
    pub container_extension: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerieInfo {
    info: Info,
    seasons: Vec<Season>,
    pub episodes: BTreeMap<String, Vec<Episode>>,
}

pub async fn get_serie_info<'a>(
    id: &'a str,
    mut params: Params<'a>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<SerieInfo> {
    params.action = Some("get_series_info");
    params.id = Some(IdType::Serie(id));

    get_json(&params, client).await
}

#[derive(Serialize, Deserialize)]
struct Category {
    #[serde(alias = "category_id")]
    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    id: Option<i64>,

    #[serde(alias = "category_name")]
    name: Option<String>,
}

async fn get_categories<'a>(
    kind: &'a Kind,
    mut params: Params<'a>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<Vec<Category>> {
    params.action = Some(match kind {
        Kind::Live => "get_live_categories",
        Kind::Movie => "get_vod_categories",
        Kind::Serie => "get_series_categories",
    });

    get_json(&params, client).await
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Value {
    #[serde(alias = "series_id", alias = "stream_id")]
    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub id: Option<i64>,

    pub name: Option<String>,

    #[serde(alias = "cover", alias = "stream_icon")]
    pub icon: Option<String>,

    #[serde(skip_serializing)]
    #[serde(alias = "last_modified")]
    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub added: Option<i64>,

    #[serde(skip_serializing)]
    #[serde(default, deserialize_with = "opt_num_from_str_or_num")]
    pub rating: Option<f64>,

    #[serde(skip_serializing)]
    pub episode_id: Option<i64>,

    #[serde(skip_serializing)]
    pub container_extension: Option<String>,
}

impl Value {
    pub fn from_movie_info(movie_info: MovieInfo, container_extension: bool) -> Value {
        Value {
            id: movie_info.data.id,
            name: movie_info.data.name,
            icon: movie_info.info.cover,
            added: None,
            rating: movie_info.info.rating,
            episode_id: None,
            container_extension: if container_extension {
                movie_info.data.container_extension
            } else {
                None
            },
        }
    }
    pub fn from_serie_info(
        serie_info: SerieInfo,
        value_id: i64,
        episode_id: Option<i64>,
        container_extension: Option<String>,
    ) -> Value {
        Value {
            id: Some(value_id),
            name: serie_info.info.name,
            icon: serie_info.info.cover,
            added: None,
            rating: serie_info.info.rating,
            episode_id,
            container_extension,
        }
    }
}
