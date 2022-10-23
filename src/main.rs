use axum::response::IntoResponse;
use axum::routing::get;
use axum::{
    body::{Body, BoxBody},
    extract,
    http::{Request, StatusCode, Uri},
    response::Response,
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr};
use tower::util::ServiceExt;
use tower_http::services::ServeDir;

mod app_error;
use crate::app_error::AppError;

#[derive(Deserialize, Serialize)]
struct CreateUser {
    email: String,
    password: String,
}


async fn get_handler(payload: extract::Json<CreateUser>) -> Result<impl IntoResponse, AppError> {
    let v = serde_json::to_string_pretty(&payload.0).map_err(|e| AppError::server_err(&e));
    Ok(v)
}

async fn static_file_handler(uri: Uri) -> Result<Response<BoxBody>, AppError> {

    println!("uri: {:?}", uri);
    let res = get_static_file(uri.clone()).await.map_err(|(stat, msg)| AppError::make_err(stat.as_u16(), &msg))?;

    if res.status() == StatusCode::NOT_FOUND {
        match format!("{}.html", uri).parse() {
            Ok(uri_html) => match get_static_file(uri_html).await {
                Ok(res) => Ok(res),
                Err((status, msg)) => Err(AppError::make_err(status.as_u16(), &msg)),
            },
            Err(err) => Err(AppError::make_err(StatusCode::NOT_FOUND.as_u16(), &err)),
        }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    match ServeDir::new("./static").oneshot(req).await {
        Ok(res) => Ok(res.map(|b| axum::body::boxed(b))),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Somthing went wrong: {}", err),
        )),
    }
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/get", post(get_handler))
        .nest("/static", get(static_file_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
