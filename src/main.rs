use std::env;
use anyhow::{Context, Result};
mod store;
mod models;
mod types;

use axum::{response::IntoResponse, routing::get};
use tokio::net::TcpListener;
use crate::store::App;
use axum::extract::{State, Path};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let config = Config::get_config().await?;
    let store = App::create_state().await?;

    let server_host = env::var("SERVER_HOST").unwrap_or("localhost".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("8000".to_string());

    let router = axum::Router::new()
        .route("/create/:payment_id", get(create_payment)) // create payment intent
        .route("/pay/:payment_id", get(pay))// create payment attempt
        .route("/update_attempt/pay/:payment_attempt_id", get(update))
        .route("/retrieve/payment_attempt/:payment_id", get(retrieve_attempt))
        .route("/retrieve/payment_intent/:payment_id", get(retrieve))
        .with_state(store)
        .route("/health", get(|| async { "OK"}));

    let server = axum::serve(
        TcpListener::bind((server_host
                                , server_port.parse::<u16>().context("invalid server port")?)).await?,
        router
    );

    server.await?;
    
    Ok(())

}

async fn create_payment(State(app) : State<App> , Path(payment_id): Path<String>) -> Result<impl IntoResponse , String>{
    let _ = app.db.create_intent(payment_id).await.map_err(|_| "db failure")?;
    Ok(axum::Json(()))
}

async fn pay(State(app) : State<App> , Path(payment_id): Path<String>) -> Result<impl IntoResponse , String>{
    let _ = app.db.retrieve_intent(payment_id.as_ref()).await.map_err(|_| "db failure")?;
    let _ = app.db.create_attempt(payment_id).await.map_err(|_| "db failure")?;
    Ok(axum::Json(()))
}
async fn update(State(app) : State<App> , Path(payment_attempt_id): Path<String>) -> Result<impl IntoResponse , String>{
    let _ = app.db.update_attempt(payment_attempt_id.as_ref()).await.map_err(|_| "db failure")?;
    Ok(axum::Json(()))
}
async fn retrieve_attempt(State(app) : State<App> , Path(payment_id): Path<String>) -> Result<impl IntoResponse , String>{
    let _ = app.db.retrieve_all(payment_id.as_ref()).await.map_err(|_| "db failure")?;
    Ok(axum::Json(()))
}

async fn retrieve(State(app): State<App>, Path(payment_id) : Path<String>) -> Result<impl IntoResponse, String>
{
    let _ = app.db.retrieve_intent(payment_id.as_ref()).await.map_err(|_| "db failure")?;
    Ok(axum::Json(()))
}