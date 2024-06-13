use actix_cors::Cors;
use actix_web::{
    dev::ServiceRequest, error::InternalError, web as ActixWeb, App, HttpResponse, HttpServer,
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use migrator::Migrator;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::prelude::*;
use std::{env, time::Duration};
// use utoipa::{
//     openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
//     Modify, OpenApi,
// };
// use utoipa_swagger_ui::SwaggerUi;

mod entities;
mod migrator;

mod api_error;
mod avatar;
mod extra;
mod favorite;
mod get;
mod home;
mod info;
mod link;
mod login;
mod search;
mod watching;

#[actix_web::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    let port = if args.len() > 1 {
        args[1].parse::<u16>().expect("Could not parse port")
    } else {
        panic!("Port is required")
    };

    let cwd = env::current_dir().expect("Could not get current directory");
    let user_agent = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    println!("Starting server on 0.0.0.0:{port} user agent {user_agent} cwd {cwd:?}");
    println!();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_str(&user_agent)
            .expect("Could not add user agent to reqwest header"),
    );

    let client = ActixWeb::Data::new(
        reqwest::Client::builder()
            //.proxy(reqwest::Proxy::all("http://localhost:8888").unwrap())
            .danger_accept_invalid_certs(true)
            .default_headers(headers)
            .build()
            .expect("Could not build reqwest client"),
    );

    //let openapi = ApiDoc::openapi();

    let db = Database::connect("sqlite://data.db?mode=rwc")
        .await
        .expect("Could not connect to database");

    // Migrator::fresh(&db)
    //     .await
    //     .expect("Could not setup database");
    Migrator::up(&db, None)
        .await
        .expect("Could not setup database");

    let db = ActixWeb::Data::new(db);

    let db_clone = db.clone();
    let client_clone = client.clone();

    actix_web::rt::spawn(async move {
        loop {
            if let Err(error) = home::make_homes(db_clone.clone(), client_clone.clone()).await {
                println!("{:?}", error);
            }
            if let Err(error) = watching::clean(db_clone.clone()).await {
                println!("{:?}", error);
            }

            actix_web::rt::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .service(index)
            .service(login::login)
            .service(
                ActixWeb::scope("")
                    .wrap(HttpAuthentication::bearer(validator))
                    .service(info::info)
                    .service(link::link)
                    .service(home::home)
                    .service(login::logoff)
                    .service(search::search)
                    .service(
                        ActixWeb::scope("/avatar")
                            .service(avatar::get)
                            .service(avatar::store)
                            .service(avatar::remove),
                    )
                    .service(
                        ActixWeb::scope("/favorite")
                            .service(favorite::get)
                            .service(favorite::store)
                            .service(favorite::remove),
                    )
                    .service(
                        ActixWeb::scope("/watching")
                            .service(watching::get)
                            .service(watching::store)
                            .service(watching::remove),
                    )
                    .service(
                        ActixWeb::scope("/get")
                            .service(get::get)
                            .service(get::info)
                            .service(get::categories),
                    ),
            )
            // .service(
            //     SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            // )
            .app_data(ActixWeb::QueryConfig::default().error_handler(|err, _| {
                InternalError::from_response(err, HttpResponse::BadRequest().finish()).into()
            }))
            .app_data(client.clone())
            .app_data(db.clone())
    })
    .bind(("0.0.0.0", port))
    .expect("Could not bind server port")
    .run()
    .await
    .expect("Could not run server")
}

#[actix_web::get("/")]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok().body("Online")
}

async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (actix_web::Error, ServiceRequest)> {
    let auth_key = credentials.token();

    if let Some(db) = req.app_data::<ActixWeb::Data<DatabaseConnection>>() {
        if login::get_session(auth_key, db).await.ok().is_some() {
            return Ok(req);
        }
    }

    Err((
        actix_web::error::ErrorUnauthorized("auth key is invalid"),
        req,
    ))
}

// #[derive(OpenApi)]
// #[openapi(
// 	paths(
// 		login::login,
// 		login::logoff,
// 	),
// 	components(
// 		schemas(Login)
// 	),
// 	tags(
// 		(name = "login", description = "Login management endpoints.")
// 	),
// 	modifiers(&SecurityAddon)
// )]
// struct ApiDoc;

// struct SecurityAddon;

// impl Modify for SecurityAddon {
//     fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
//         let components = openapi.components.as_mut().unwrap();
//         components.add_security_scheme(
//             "auth_key",
//             SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Basic).build()),
//         )
//     }
// }
