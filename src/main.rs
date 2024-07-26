use std::env;

type ResultT<T> = Result<T, anyhow::Error>;

struct Config{
    #[cfg(feature = "cassandra")]
    db_config : Cassconfig,
    #[cfg(not(feature = "cassandra"))]
    db_config : RedisConfig
}

impl Config{
    pub async fn get_config() -> ResultT<Self>{
        let db_config = {
            #[cfg(feature = "cassandra")]
            CassConfig::new();
            #[cfg(not(feature = "cassandra"))]
            RedisConfig::new()
        }?;
        Ok(Self{ db_config})
    }
}

struct Cassconfig{
    url : String,
    password: String,
    username: String,
    r_factor: Option<usize>,
}

use anyhow::{Context, Result};

impl Cassconfig{
    fn new() -> Result<Self, anyhow::Error>{
        let url= env::var("CASSANDRA_URL").context("CASSANDRA_URL not found")?;
        let password = env::var("CASSANDRA_PASSWORD").context("CASSANDRA_PASSWORD not found")?;
        let username = env::var("CASSANDRA_USERNAME").context("CASSANDRA_USERNAME not found")?;
        Ok(Self{
            url,
            password,
            username,
            r_factor : None
        })
        
    }
}

impl RedisConfig{
    fn new() -> ResultT<Self>{
        let url= env::var("REDIS_URL").context("Redis url not found")?;
        let pool_size = env::var("REDIS_POOL_SIZE").ok().and_then(|x| x.parse::<usize>().ok()).unwrap_or(5);
        Ok(Self{
            url,
            pool_size
        })
    }
}

struct RedisConfig{
    url: String,
    pool_size : usize,
}


use axum::routing::get;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = Config::get_config().await?;
    let store = config.get_store().await?;

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


/*
trait Store {
    fn get_connection
}

*/