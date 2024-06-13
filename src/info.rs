use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{
    api_error::{ApiError, ApiResult},
    entities::prelude::*,
    login::{self, get_login_info},
};

#[actix_web::get("/info")]
async fn info(
    credentials: BearerAuth,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    let user_info = get_update_user_info(session, db, client).await?;

    Ok(HttpResponse::Ok().json(user_info))
}

async fn get_update_user_info(
    session: Session,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<UserInfo> {
    let login = login::get_login(&session, &db).await?;

    let user_info = UserInfoEntity::find()
        .filter(UserInfoColumn::Id.eq(session.id))
        .one(db.get_ref())
        .await?
        .ok_or(ApiError::AccountNotFound)?;

    match get_login_info(&login, client).await {
        Ok(mut user_info_ret) => {
            user_info_ret.id = user_info.id;

            UserInfoEntity::update(Into::<UserInfoActiveModel>::into(user_info_ret.clone()))
                .exec(db.get_ref())
                .await?;

            Ok(user_info_ret)
        }
        Err(err) => {
            if let ApiError::AccountNotFound = err {
                SessionEntity::delete(Into::<SessionActiveModel>::into(session))
                    .exec(db.get_ref())
                    .await?;
            }
            Err(err)
        }
    }
}
