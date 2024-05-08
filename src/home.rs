use actix_web::{web as ActixWeb, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use ordered_float::OrderedFloat;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use std::cmp::Reverse;

use crate::{
    api_error::ApiResult,
    entities::{home::Kind, prelude::*},
    extra::{get_month_ago, Params},
    get::{get_movies, get_series, Value},
    login,
};

#[derive(Serialize, Debug, Clone)]
pub struct Homes {
    top: Vec<Home>,
    movies: Vec<Home>,
    series: Vec<Home>,
}

#[actix_web::get("/home")]
pub async fn home(
    credentials: BearerAuth,
    db: ActixWeb::Data<DatabaseConnection>,
) -> ApiResult<HttpResponse> {
    let auth_key = credentials.token();

    let session = login::get_session(auth_key, &db).await?;

    let homev = HomeEntity::find()
        .filter(HomeColumn::SessionId.eq(session.id))
        .all(db.get_ref())
        .await?;

    let home = Homes {
        top: homev
            .iter()
            .filter(|x| x.kind == Kind::TopMovie || x.kind == Kind::TopSerie)
            .cloned()
            .collect(),
        movies: homev
            .iter()
            .filter(|x| x.kind == Kind::Movie)
            .cloned()
            .collect(),
        series: homev
            .iter()
            .filter(|x| x.kind == Kind::Serie)
            .cloned()
            .collect(),
    };

    Ok(HttpResponse::Ok().json(home))
}

pub async fn make_homes(
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<()> {
    let month_ago = get_month_ago()?;
    let logins = LoginEntity::find().all(db.get_ref()).await?;

    for login in logins {
        let _ = make(&login, month_ago, db.clone(), client.clone()).await;
    }

    Ok(())
}

pub async fn make(
    login: &Login,
    month_ago: i64,
    db: ActixWeb::Data<DatabaseConnection>,
    client: ActixWeb::Data<reqwest::Client>,
) -> ApiResult<()> {
    let mut movies = get_movies(&None, Params::new(login), client.clone()).await?;
    let mut series = get_series(&None, Params::new(login), client).await?;

    let mut tops = movies
        .clone()
        .into_iter()
        .map(|x| (Kind::TopMovie, x))
        .chain(series.clone().into_iter().map(|x| (Kind::TopSerie, x)))
        .filter(|(_, y)| y.rating.is_some() && y.added.is_some_and(|z| z > month_ago))
        .collect::<Vec<(Kind, Value)>>();

    tops.sort_by_cached_key(|x| {
        (
            Reverse(OrderedFloat(x.1.rating.unwrap())),
            Reverse(x.1.added.unwrap()),
        )
    });
    tops.truncate(10);

    movies.sort_by_cached_key(|x| Reverse(x.added));
    movies.truncate(20);

    series.sort_by_cached_key(|x| Reverse(x.added));
    series.truncate(20);

    HomeEntity::delete_many()
        .filter(HomeColumn::SessionId.eq(login.id))
        .exec(db.get_ref())
        .await?;

    for (kind, top) in tops {
        HomeEntity::insert(HomeActiveModel {
            id: Default::default(),
            session_id: ActiveValue::Set(login.id),
            kind: ActiveValue::Set(kind),
            value_id: ActiveValue::Set(top.id),
            name: ActiveValue::Set(top.name),
            icon: ActiveValue::Set(top.icon),
        })
        .exec(db.get_ref())
        .await?;
    }

    for movie in movies {
        HomeEntity::insert(HomeActiveModel {
            id: Default::default(),
            session_id: ActiveValue::Set(login.id),
            kind: ActiveValue::Set(Kind::Movie),
            value_id: ActiveValue::Set(movie.id),
            name: ActiveValue::Set(movie.name),
            icon: ActiveValue::Set(movie.icon),
        })
        .exec(db.get_ref())
        .await?;
    }

    for serie in series {
        HomeEntity::insert(HomeActiveModel {
            id: Default::default(),
            session_id: ActiveValue::Set(login.id),
            kind: ActiveValue::Set(Kind::Serie),
            value_id: ActiveValue::Set(serie.id),
            name: ActiveValue::Set(serie.name),
            icon: ActiveValue::Set(serie.icon),
        })
        .exec(db.get_ref())
        .await?;
    }

    Ok(())
}
