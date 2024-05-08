use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use std::str::FromStr;

use crate::{
    api_error::{ApiError, ApiResult},
    entities::{favorite::Kind, prelude::*},
    extra::Params,
    get::{get_lives, get_movie_info, get_serie_info, Value},
    login,
};

#[derive(Serialize, Clone, Debug)]
pub struct Favorites {
    lives: Vec<Favorite>,
    movies: Vec<Favorite>,
    series: Vec<Favorite>,
}

#[actix_web::get("/get/{avatar}")]
pub async fn get(
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

    let favorite = FavoriteEntity::find()
        .filter(FavoriteColumn::AvatarId.eq(avatar))
        .all(db.get_ref())
        .await?;

    let favorites = Favorites {
        lives: favorite
            .iter()
            .filter(|x| x.kind == Kind::Live)
            .cloned()
            .collect(),
        movies: favorite
            .iter()
            .filter(|x| x.kind == Kind::Movie)
            .cloned()
            .collect(),
        series: favorite
            .iter()
            .filter(|x| x.kind == Kind::Serie)
            .cloned()
            .collect(),
    };

    Ok(HttpResponse::Ok().json(favorites))
}

#[actix_web::get("/store/{avatar}/{kind}/{id}")]
async fn store(
    credentials: BearerAuth,
    path: ActixWeb::Path<(i64, Kind, String)>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
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

    let login = login::get_login(&session, &db).await?;

    let value = match kind {
        Kind::Live => {
            let id = i64::from_str(&id)?;
            let lives = get_lives(&None, Params::new(&login), client).await?;
            lives
                .into_iter()
                .find(|x| x.id == Some(id))
                .ok_or(ApiError::WrongId)?
        }
        Kind::Movie => {
            let movie_info = get_movie_info(&id, Params::new(&login), client).await?;
            Value::from_movie_info(movie_info, false)
        }
        Kind::Serie => {
            let int_id = i64::from_str(&id)?;
            let serie_info = get_serie_info(&id, Params::new(&login), client).await?;
            Value::from_serie_info(serie_info, int_id, None, None)
        }
    };

    let exist = FavoriteEntity::find()
        .filter(FavoriteColumn::AvatarId.eq(avatar))
        .filter(FavoriteColumn::Kind.eq(kind.clone()))
        .filter(FavoriteColumn::ValueId.eq(value.id))
        .one(db.get_ref())
        .await?;

    if exist.is_none() {
        FavoriteEntity::insert(FavoriteActiveModel {
            id: Default::default(),
            avatar_id: ActiveValue::Set(avatar),
            kind: ActiveValue::Set(kind),
            value_id: ActiveValue::Set(value.id),
            name: ActiveValue::Set(value.name),
            icon: ActiveValue::Set(value.icon),
        })
        .exec(db.get_ref())
        .await?;

        Ok(HttpResponse::Ok().body("true"))
    } else {
        Ok(HttpResponse::Ok().body("false"))
    }
}

#[actix_web::get("/remove/{avatar}/{kind}/{id}")]
async fn remove(
    credentials: BearerAuth,
    path: ActixWeb::Path<(i64, Kind, String)>,
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

    let favorite: FavoriteActiveModel = FavoriteEntity::find()
        .filter(FavoriteColumn::AvatarId.eq(avatar))
        .filter(FavoriteColumn::Kind.eq(kind))
        .filter(FavoriteColumn::ValueId.eq(id))
        .one(db.get_ref())
        .await?
        .ok_or(ApiError::WrongId)?
        .into();

    FavoriteEntity::delete(favorite).exec(db.get_ref()).await?;

    Ok(HttpResponse::Ok().body("true"))
}
