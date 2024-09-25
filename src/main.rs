use anyhow::{Context, Result};
use std::env;
use std::str::FromStr;
mod models;
mod store;
mod time;
mod types;
mod utils;

use crate::store::App;
use axum::extract::Request;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::{
    http::{HeaderMap, HeaderValue},
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use tokio::net::TcpListener;

const cell: &'static str = env!("CELL", "couldnt find cell env");

// static OWNER: once_cell::sync::Lazy<metrics::Counter> =
//     once_cell::sync::Lazy::new(|| metrics::counter!("OWNER_CHANGED", "region" => cell));

fn metrics_app() -> axum::Router {
    let recorder_handle = setup_metrics_recorder();
    axum::Router::new().route(
        "/metrics",
        get(move || std::future::ready(recorder_handle.render())),
    )
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        10.0, 50.0, 100.0, 300.0, 500.0, 750.0, 1000.0, 1500.0, 2000.0, 3000.0, 5000.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("latency_tracker_r".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .set_buckets_for_metric(
            Matcher::Full("replication_lag".to_string()),
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

async fn map(req: axum::http::Request<axum::body::Body>) -> axum::http::Request<axum::body::Body> {
    if req
        .headers()
        .get("x-region")
        .map(|val| val != crate::types::cell)
        .unwrap_or_default()
    {
        //OWNER.increment(1);
        metrics::counter!("OWNER_CHANGED", "region" => cell).increment(1);
    }
    req
}

async fn map_responsse(
    mut res: axum::http::Response<axum::body::Body>,
) -> axum::http::Response<axum::body::Body> {
    res.headers_mut().insert(
        hyper::header::HeaderName::from_static("x-region"),
        HeaderValue::from_static(crate::types::cell),
    );
    res
}

async fn start_app() {
    let store = App::create_state().await.expect("state creation failed");

    let server_host = env::var("SERVER_HOST").unwrap_or("localhost".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("8000".to_string());

    let router = axum::Router::new()
        .route("/init_db", get(init_db))
        .route("/init_lag/:pid", get(init_lag))
        .route("/record_lag/:pid/:init_time", get(record_lag))
        .route("/create/account/:merchant_id", get(create_account))
        .route("/create/:payment_id/:merchant_id", get(create_payment)) // create payment intent
        .route("/pay/:payment_id/:version", get(pay)) // create payment attempt
        .route("/update_intent/:payment_intent_id", get(update_intent))
        .route(
            "/update_attempt/pay/:version/:payment_attempt_id",
            get(update_attempt),
        )
        .route(
            "/retrieve_by_id/payment_attempt/:payment_attempt_id/:payment_id",
            get(retrieve_by_id),
        )
        .route(
            "/retrieve/payment_attempt/:payment_id",
            get(retrieve_attempt),
        )
        .route("/retrieve/payment_intent/:payment_id", get(retrieve))
        .with_state(store)
        .route("/health", get(|| async { "OK" }))
        .layer(axum::middleware::map_request(map))
        .layer(axum::middleware::map_response(map_responsse));
    axum::serve(
        TcpListener::bind((
            server_host,
            server_port
                .parse::<u16>()
                .context("invalid server port")
                .expect("invalid server port"),
        ))
        .await
        .expect("port binding failed"),
        router,
    )
    .await
    .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_, _) = tokio::join!(start_metrics_server(), start_app());
    Ok(())
}

async fn init_lag(State(app) : State<App>, Path(pid) : Path<String>) -> Result<impl IntoResponse, DB_ERR> {
    #[cfg(feature = "replicate")]
    {
        let _ = app.db.create_intent(pid.clone(), true).await.map_err(|e| DB_ERR(e.to_string()))?;
        app.peer.call_next_cell(pid, chrono::Utc::now().timestamp_millis()).await.map_err(|e| DB_ERR(e.to_string()))?;
    }
    Ok(axum::Json(()))
}
          
async fn record_lag(State(app) : State<App>, Path((pid, init_time)) : Path<(String, String)>) -> Result<impl IntoResponse, DB_ERR>{
    tokio::spawn(record_pi_lag(init_time,pid,app));
    Ok(axum::Json(()))
}

async fn record_pi_lag(init_time: String, pid : String, app : App) -> Result<(),String>{
    let mut retry_count =0;
    loop{
        if retry_count == 10{
            return Err("retry exceeded".to_string())
        }
        if let Ok(_) = app.db.retrieve_intent(pid.as_str()).await{
            let now = chrono::Utc::now().timestamp_millis();
            init_time
                .parse()
                .ok()
                .map(|start : i64| metrics::histogram!("replication_lag").record((now-start) as f64));
            return Ok(())
        }
        retry_count = retry_count + 1;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}

async fn init_db(State(app): State<App>) -> Result<impl IntoResponse, String> {
    let _ = app.db.prepare().await.map_err(|_| "init failed")?;
    Ok(axum::Json(()))
}
async fn create_account(
    State(app): State<App>,
    Path(merchant_id): Path<String>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .create_account(merchant_id)
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(()))
}

async fn create_payment(
    State(app): State<App>,
    Path((payment_id, merchant_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .retrieve_account(merchant_id)
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    let res = app
        .db
        .create_intent(payment_id, false)
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(res))
}

async fn pay(
    State(app): State<App>,
    Path((payment_id, version)): Path<(String, String)>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .retrieve_intent(payment_id.as_ref())
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    let res = app
        .db
        .create_attempt(payment_id, version)
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(res))
}
async fn update_attempt(
    State(app): State<App>,
    Path((version, payment_attempt_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .update_attempt(payment_attempt_id.as_ref(), version)
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(()))
}

async fn update_intent(
    State(app): State<App>,
    Path(payment_intent_id): Path<String>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .update_intent(payment_intent_id.as_ref())
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(()))
}
async fn retrieve_attempt(
    State(app): State<App>,
    Path(payment_id): Path<String>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .retrieve_all(payment_id.as_ref())
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(()))
}

async fn retrieve_by_id(
    State(app): State<App>,
    Path((payment_attempt_id, payment_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .retrieve_by_id(payment_attempt_id, payment_id)
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(()))
}

async fn retrieve(
    State(app): State<App>,
    Path(payment_id): Path<String>,
) -> Result<impl IntoResponse, DB_ERR> {
    let _ = app
        .db
        .retrieve_intent(payment_id.as_ref())
        .await
        .map_err(|e| DB_ERR(e.to_string()))?;
    Ok(axum::Json(()))
}

struct DB_ERR(String);

impl IntoResponse for DB_ERR {
    fn into_response(self) -> axum::response::Response {
        println!("servers error is : {}", self.0);
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}
