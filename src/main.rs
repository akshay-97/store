use std::env;
use anyhow::{Context, Result};
mod store;
mod models;
mod types;
mod utils;
mod time;

use axum::{response::IntoResponse, routing::get};
use tokio::net::TcpListener;
use crate::store::App;
use axum::extract::{State, Path};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle, Matcher};

fn metrics_app() -> axum::Router {
    let recorder_handle = setup_metrics_recorder();
    axum::Router::new().route("/metrics", get(move || std::future::ready(recorder_handle.render())))
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        10.0, 50.0, 100.0, 300.0, 500.0, 750.0, 1000.0, 1500.0, 2000.0, 3000.0 , 5000.0
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("latency_tracker".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

async fn start_metrics_server() {
    let app = metrics_app();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    // tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}


async fn start_app(){
    let store = App::create_state().await.expect("state creation failed");

    let server_host = env::var("SERVER_HOST").unwrap_or("localhost".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("8000".to_string());

    let router = axum::Router::new()
        .route("/init_db", get(init_db))
        .route("/create/:payment_id", get(create_payment)) // create payment intent
        .route("/pay/:payment_id/:version", get(pay))// create payment attempt
        .route("/update_intent/:payment_intent_id", get(update_intent))
        .route("/update_attempt/pay/:version/:payment_attempt_id", get(update_attempt))
        .route("/retrieve/payment_attempt/:payment_id", get(retrieve_attempt))
        .route("/retrieve/payment_intent/:payment_id", get(retrieve))
        .with_state(store)
        .route("/health", get(|| async { "OK"}));
    axum::serve(
        TcpListener::bind((server_host
                                , server_port.parse::<u16>().context("invalid server port").expect("invalid server port"))).await.expect("port binding failed"),
        router).await.unwrap()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_, _) = tokio::join!(start_metrics_server(), start_app());
    Ok(())

}

async fn init_db(State(app) : State<App>) -> Result<impl IntoResponse, String>{
    let _ = app.db.prepare().await.map_err(|_| "init failed")?;
    Ok(axum::Json(()))
}
async fn create_payment(State(app) : State<App> , Path(payment_id): Path<String>) -> Result<impl IntoResponse , String>{
    let _ = app.db.create_intent(payment_id).await.map_err(|e| e.to_string())?;
    Ok(axum::Json(()))
}

async fn pay(State(app) : State<App> ,Path((payment_id,version)): Path<(String,String)>) -> Result<impl IntoResponse , String>{
    let _ = app.db.retrieve_intent(payment_id.as_ref()).await.map_err(|e| e.to_string())?;
    let _ = app.db.create_attempt(payment_id, version).await.map_err(|e| e.to_string())?;
    Ok(axum::Json(()))
}
async fn update_attempt(State(app) : State<App> , Path((version, payment_attempt_id)): Path<(String,String)>) -> Result<impl IntoResponse , String>{
    let _ = app.db.update_attempt(payment_attempt_id.as_ref(), version).await.map_err(|e| e.to_string())?;
    Ok(axum::Json(()))
}

async fn update_intent(State(app) : State<App> , Path(payment_intent_id): Path<String>) -> Result<impl IntoResponse , String>{
    let _ = app.db.update_intent(payment_intent_id.as_ref()).await.map_err(|e| e.to_string())?;
    Ok(axum::Json(()))
}
async fn retrieve_attempt(State(app) : State<App> , Path(payment_id): Path<String>) -> Result<impl IntoResponse , String>{
    let _ = app.db.retrieve_all(payment_id.as_ref()).await.map_err(|e| e.to_string())?;
    Ok(axum::Json(()))
}

async fn retrieve(State(app): State<App>, Path(payment_id) : Path<String>) -> Result<impl IntoResponse, String>
{
    let _ = app.db.retrieve_intent(payment_id.as_ref()).await.map_err(|e| e.to_string())?;
    Ok(axum::Json(()))
}