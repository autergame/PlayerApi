use actix_web::{get, post, web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use rand::{rngs::OsRng, RngCore};
use sea_orm::{ActiveValue, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::{
    api_error::{ApiError, ApiResult},
    entities::prelude::*,
    extra::{get_days_ago, get_json, Params},
    home,
};

#[derive(Deserialize)]
pub struct LoginResponse {
    pub user_info: UserInfo,
}

// #[utoipa::path(
//     params(
//         Login
//     ),
//     responses(
//         (status = 200, description = "Login auth key", body = String),
// 		(status = 500, description = "Cant login", body = ApiErrorJson),
//     )
// )]
#[post("/login")]
pub async fn login(
    login: ActixWeb::Json<Login>,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<HttpResponse> {
    let mut login = login.into_inner();

    let mut server = url::Url::parse(&login.server)?;

    if server.path() == "/" {
        server.set_path("player_api.php");
    }

    login.server = server.to_string();

    if let Some(login) = LoginEntity::find()
        .filter(
            Condition::all()
                .add(LoginColumn::Server.eq(&login.server))
                .add(LoginColumn::Username.eq(&login.username))
                .add(LoginColumn::Password.eq(&login.password)),
        )
        .one(db.get_ref())
        .await?
    {
        if let Some(session) = SessionEntity::find_by_id(login.id)
            .one(db.get_ref())
            .await?
        {
            return Ok(HttpResponse::Ok().body(session.auth_key));
        }
    }

    let mut user_info = get_login_info(&login, client.clone()).await?;

    let mut random_bytes = [0u8; 16];
    OsRng.try_fill_bytes(&mut random_bytes)?;
    let auth_key = hex::encode(random_bytes);

    let session_res = SessionEntity::insert(SessionActiveModel {
        id: ActiveValue::default(),
        auth_key: ActiveValue::Set(auth_key.clone()),
    })
    .exec(db.get_ref())
    .await?;

    login.id = session_res.last_insert_id;
    user_info.id = session_res.last_insert_id;

    let month_ago = get_days_ago(30);
    home::make(&login, month_ago, db.clone(), client).await?;

    LoginEntity::insert(Into::<LoginActiveModel>::into(login))
        .exec(db.get_ref())
        .await?;

    UserInfoEntity::insert(Into::<UserInfoActiveModel>::into(user_info))
        .exec(db.get_ref())
        .await?;

    Ok(HttpResponse::Ok().body(auth_key))
}

// #[utoipa::path(
//     responses(
//         (status = 200, description = "Logoff"),
// 		(status = 500, description = "Cant Logoff", body = ApiErrorJson),
//     ),
// 	security(
// 		("auth_key" = [])
// 	)
// )]
#[get("/logoff")]
pub async fn logoff(
    credentials: BearerAuth,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let session: SessionActiveModel = get_session(auth_key, &db).await?.into();

    SessionEntity::delete(session).exec(db.get_ref()).await?;

    Ok(HttpResponse::Ok().finish())
}

pub async fn get_login_info(
    loginv: &Login,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<UserInfo> {
    let params = Params::new(loginv);

    let login_response = get_json::<LoginResponse>(&params, client).await?;

    if login_response.user_info.auth > 0 {
        return Ok(login_response.user_info);
    }

    Err(ApiError::AccountNotFound)
}

pub async fn get_session(auth_key: &str, db: &DatabaseConnection) -> ApiResult<Session> {
    SessionEntity::find()
        .filter(SessionColumn::AuthKey.eq(auth_key))
        .one(db)
        .await?
        .ok_or(ApiError::WrongAuthKey)
}

pub async fn get_login(session: &Session, db: &DatabaseConnection) -> ApiResult<Login> {
    LoginEntity::find()
        .filter(LoginColumn::Id.eq(session.id))
        .one(db)
        .await?
        .ok_or(ApiError::AccountNotFound)
}
