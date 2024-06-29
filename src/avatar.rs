use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{
    api_error::{ApiError, ApiResult},
    entities::prelude::*,
    extra::BoolResult,
    login,
};

#[actix_web::get("/get")]
pub async fn get(
    credentials: BearerAuth,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    let avatar = AvatarEntity::find()
        .filter(AvatarColumn::SessionId.eq(session.id))
        .all(db.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(avatar))
}

#[actix_web::get("/store/{name}")]
async fn store(
    credentials: BearerAuth,
    path: ActixWeb::Path<String>,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let name = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    let exist = AvatarEntity::find()
        .filter(AvatarColumn::SessionId.eq(session.id))
        .filter(AvatarColumn::Name.eq(&name))
        .one(db.get_ref())
        .await?;

    if exist.is_none() {
        AvatarEntity::insert(AvatarActiveModel {
            id: Default::default(),
            session_id: ActiveValue::Set(session.id),
            name: ActiveValue::Set(name),
        })
        .exec(db.get_ref())
        .await?;
    }

    Ok(HttpResponse::Ok().json(BoolResult {
        result: exist.is_none(),
    }))
}

#[actix_web::get("/remove/{id}")]
async fn remove(
    credentials: BearerAuth,
    path: ActixWeb::Path<i64>,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let id = path.into_inner();
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    let avatar: AvatarActiveModel = AvatarEntity::find()
        .filter(AvatarColumn::Id.eq(id))
        .filter(AvatarColumn::SessionId.eq(session.id))
        .one(db.get_ref())
        .await?
        .ok_or(ApiError::WrongId)?
        .into();

    let result = AvatarEntity::delete(avatar).exec(db.get_ref()).await?;

    Ok(HttpResponse::Ok().json(BoolResult {
        result: result.rows_affected > 0,
    }))
}
