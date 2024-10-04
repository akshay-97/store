use anyhow::{Context, Result};
use std::env;
mod models;
mod store;
mod time;
mod types;
mod utils;
mod astr_conn;

use crate::store::App;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{response::IntoResponse, routing::get};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use tokio::net::TcpListener;

fn metrics_app() -> axum::Router {
    let recorder_handle = setup_metrics_recorder();
    axum::Router::new().route(
        "/metrics",
        get(move || std::future::ready(recorder_handle.render())),
    )
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[1.0, 2.0, 3.0, 4.0,
        10.0, 50.0, 100.0,300.0
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("latency_tracker_e".to_string()),
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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();
    // tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn start_app() {
    let store = App::create_state().await.expect("state creation failed");

    let owner = metrics::counter!("OWNER_CHANGED", "region" => crate::types::cell);

    let server_host = env::var("SERVER_HOST").unwrap_or("localhost".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or("8000".to_string());
    let trace = tower::ServiceBuilder::new().layer(
        tower_http::trace::TraceLayer::new_for_http().on_request(
            move |req: &hyper::Request<axum::body::Body>, _: &tracing::Span| {
                if req
                    .headers()
                    .get("x-region")
                    .map(|val| val != crate::types::cell)
                    .unwrap_or_default()
                {
                    owner.increment(1);
                }
            },
        ),
    );

    let router = axum::Router::new()
        .route("/init_db", get(init_db))
        .route("/create/account/:merchant_id", get(create_account))
        .route("/init_lag/:pid", get(init_lag))
        .route("/record_lag/:pid/:init_time", get(record_lag))
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
        .route("/create/payment_method/:customer/:id", get(create_method))
        .route("/find/payment_methods/:customer_id", get(find_all_customer))
        .layer(trace)
        .with_state(store)
        .route("/health", get(|| async { "OK" }));
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
        if retry_count == 100{
            println!("retry exceeded");
            metrics::histogram!("replication_lag").record(10000f64);
            return Err("retry exceeded".to_string())
        }
        if let Ok(_) = app.db.retrieve_intent(pid.as_str()).await{
            let now = chrono::Utc::now().timestamp_millis();
            println!("found entry, attempting to push metric");
            init_time
                .parse()
                .ok()
                .map(|start : i64| metrics::histogram!("replication_lag").record((now-start) as f64));
            return Ok(())
        }
        retry_count = retry_count + 1;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn init_db(State(app) : State<App>) -> Result<impl IntoResponse, String>{
    let _ = app.db.prepare().await.map_err(|e| e.to_string())?;
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

async fn create_method(State(app) : State<App>,
    Path((customer_id, pid)) : Path<(String, String)>) -> Result<impl IntoResponse, DB_ERR>{
        let _ = app
            .db
            .create_payment_method(customer_id , pid)
            .await
            .map_err(|e| DB_ERR(e.to_string()))?;
        Ok(axum::Json(()))
}

async fn find_all_customer(State(app) : State<App>, Path(customer_id) : Path<String>)
    -> Result<impl IntoResponse , DB_ERR>{
       Err::<(), DB_ERR>(DB_ERR("ask me".to_string()))
}
struct DB_ERR(String);

impl IntoResponse for DB_ERR {
    fn into_response(self) -> axum::response::Response {
        println!("ERROR: {}", self.0); 
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

