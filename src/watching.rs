use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use std::str::FromStr;

use crate::{
    api_error::{ApiError, ApiResult},
    entities::{prelude::*, watching::Kind},
    extra::Params,
    get::{get_movie_info, get_serie_info, Value},
    login,
};

#[actix_web::get("/get/{avatar}")]
async fn get(
    credentials: BearerAuth,
    path: ActixWeb::Path<i64>,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let avatar = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    AvatarEntity::find()
        .filter(AvatarColumn::Id.eq(avatar))
        .filter(AvatarColumn::SessionId.eq(session.id))
        .one(db.get_ref())
        .await?
        .ok_or(ApiError::WrongAvatar)?;

    let watching = WatchingEntity::find()
        .filter(WatchingColumn::AvatarId.eq(avatar))
        .all(db.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(watching))
}

#[derive(Deserialize, Debug)]
struct Store {
    avatar: i64,
    kind: Kind,
    time: i64,
    id: String,
    episode_id: Option<i64>,
}

#[actix_web::routes]
#[get("/store/{avatar}/{kind}/{time}/{id}")]
#[get("/store/{avatar}/{kind}/{time}/{id}/{episode_id}")]
async fn store(
    credentials: BearerAuth,
    path: ActixWeb::Path<Store>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let store = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    AvatarEntity::find()
        .filter(AvatarColumn::Id.eq(store.avatar))
        .filter(AvatarColumn::SessionId.eq(session.id))
        .one(db.get_ref())
        .await?
        .ok_or(ApiError::WrongAvatar)?;

    let login = login::get_login(&session, &db).await?;

    let value = match store.kind {
        Kind::Movie => {
            let movie_info = get_movie_info(&store.id, Params::new(&login), client).await?;
            Value::from_movie_info(movie_info, true)
        }
        Kind::Serie => {
            let id = i64::from_str(&store.id)?;
            let episode_id = store.episode_id.ok_or(ApiError::NotFound)?;

            if let Some(watching) = WatchingEntity::find()
                .filter(WatchingColumn::AvatarId.eq(store.avatar))
                .filter(WatchingColumn::Kind.eq(store.kind.clone()))
                .filter(WatchingColumn::ValueId.eq(id))
                .filter(WatchingColumn::EpisodeId.ne(episode_id))
                .one(db.get_ref())
                .await?
            {
                WatchingEntity::delete(Into::<WatchingActiveModel>::into(watching))
                    .exec(db.get_ref())
                    .await?;
            }

            let serie_info = get_serie_info(&store.id, Params::new(&login), client).await?;

            let container_extension = serie_info
                .episodes
                .iter()
                .map(|x| x.1.iter().find(|y| y.id == Some(episode_id)))
                .next()
                .flatten()
                .ok_or(ApiError::WrongEpisodeId)?
                .container_extension
                .clone();

            Value::from_serie_info(serie_info, id, Some(episode_id), container_extension)
        }
    };

    let exist = WatchingEntity::find()
        .filter(WatchingColumn::AvatarId.eq(store.avatar))
        .filter(WatchingColumn::Kind.eq(store.kind.clone()))
        .filter(WatchingColumn::ValueId.eq(value.id))
        .one(db.get_ref())
        .await?;

    if let Some(watching) = exist {
        WatchingEntity::update(WatchingActiveModel {
            id: ActiveValue::Set(watching.id),
            time: ActiveValue::Set(store.time),
            ..Default::default()
        })
        .exec(db.get_ref())
        .await?;
    } else {
        WatchingEntity::insert(WatchingActiveModel {
            id: Default::default(),
            avatar_id: ActiveValue::Set(store.avatar),
            kind: ActiveValue::Set(store.kind),
            value_id: ActiveValue::Set(value.id),
            name: ActiveValue::Set(value.name),
            icon: ActiveValue::Set(value.icon),
            time: ActiveValue::Set(store.time),
            episode_id: ActiveValue::Set(value.episode_id),
            container_extension: ActiveValue::Set(value.container_extension),
        })
        .exec(db.get_ref())
        .await?;
    }

    Ok(HttpResponse::Ok().body("true"))
}

#[actix_web::get("/remove/{avatar}/{kind}/{id}")]
async fn remove(
    credentials: BearerAuth,
    path: ActixWeb::Path<(i64, Kind, i64)>,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let (avatar, kind, id) = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    AvatarEntity::find()
        .filter(AvatarColumn::Id.eq(avatar))
        .filter(AvatarColumn::SessionId.eq(session.id))
        .one(db.get_ref())
        .await?
        .ok_or(ApiError::WrongAvatar)?;

    let watching: WatchingActiveModel = WatchingEntity::find()
        .filter(WatchingColumn::AvatarId.eq(avatar))
        .filter(WatchingColumn::Kind.eq(kind))
        .filter(WatchingColumn::ValueId.eq(id))
        .one(db.get_ref())
        .await?
        .ok_or(ApiError::WrongId)?
        .into();

    WatchingEntity::delete(watching).exec(db.get_ref()).await?;

    Ok(HttpResponse::Ok().body("true"))
}
