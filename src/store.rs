use cassandra_cpp::*;
use crate::models::*;
use std::env;
use anyhow::Context;

#[async_trait::async_trait]
pub trait Init{
    async fn prepare(&self) -> std::result::Result<(), Box<dyn std::error::Error>>{
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait StorageInterface : dyn_clone::DynClone +
    PaymentIntentInterface + PaymentAttemptInterface + Send + Sync + 'static  + Init
{}

pub struct App{
    pub db : Box<dyn StorageInterface>,
}

impl Clone for App{
    fn clone(&self) -> Self {
        Self{
            db : dyn_clone::clone_box(&*(self.db))
        }
    }
}

impl App{
    pub async fn create_state() -> std::result::Result<Self, Box<dyn std::error::Error>>{
        Ok(Self{
            #[cfg(feature= "cassandra")]
            db : Box::new(CassClient::new().await?),
            #[cfg(not(feature= "cassandra"))]
            db : Box::new(RedisClient::new().await?),

        }
    )
    }   
}

#[derive(Clone)]
pub struct CassClient{
    pub cassandra_session: Session
}

#[async_trait::async_trait]
impl Init for CassClient{
    async fn prepare(&self) -> std::result::Result<(),Box<dyn std::error::Error> > {
        let _ = self.cassandra_session.execute(include_str!("schema.cql")).await?;
        Ok(())
    }
}

impl StorageInterface for CassClient {}

impl CassClient{
    pub async fn new() -> std::result::Result<Self, Box<dyn std::error::Error>>{
        let url= env::var("CASSANDRA_URL").context("CASSANDRA_URL not found")?;
        let password = env::var("CASSANDRA_PASSWORD").context("CASSANDRA_PASSWORD not found")?;
        let username = env::var("CASSANDRA_USERNAME").context("CASSANDRA_USERNAME not found")?;
        let mut cluster = Cluster::default();
        cluster
            .set_contact_points(&url)?
            .set_credentials(&username, &password)?
            .set_load_balance_round_robin();
        
        let session = cluster.connect().await?;
        Ok(Self{
            cassandra_session: session
        })
    }

    // pub async fn from_conf(config: CassConfig) -> Result<Self, anyhow::Error>{
    //     let mut cluster = Cluster::default();
    //     let session = cluster
    //         .set_contact_points(&config.url)?
    //         .set_credentials(&config.username, &config.password)?
    //         .set_load_balance_round_robin()
    //         .connect().await?;
    //     Ok(Self{
    //         cassandra_session : session
    //     })
    // }
}


#[cfg(not(feature = "cassandra"))]
use fred::prelude::RedisPool;
use fred::{interfaces::ClientLike, bytes_utils::string::Storage};

#[cfg(not(feature = "cassandra"))]
#[derive(Clone)]
pub struct RedisClient{
    pub redis_conn : std::sync::Arc<RedisPool>,
    pub redis_ttl : i64,
    pub wait_config : WaitConfig,
}

#[derive(Clone)]
pub enum WaitConfig {
     NoWait,
     Wait(usize, i64)
}

impl WaitConfig{
    fn new() -> Self{
        let wait_timeout = env::var("REDIS_WAIT_TIMEOUT").ok();
        let wait_replicas = env::var("REDIS_WAIT_REPLICA").ok();
        
    }
}

#[cfg(not(feature = "cassandra"))]
impl RedisClient{
    // starting redis in cluster mode
    pub async fn new() -> std::result::Result<Self, Box<dyn std::error::Error>>{
        let redis_host = env::var("REDIS_HOST").context("REDIS_HOST not found")?;
        let redis_port = env::var("REDID_PORT").context("REDIS_PORT not found")?;
        let redis_url = format!("redis-cluster://{}:{}", redis_host, redis_port);
        let wait_config = WaitConfig::new();
        let redis_pool_size = env::var("REDIS_POOL")
                .ok()
                .and_then(|x| x.parse::<usize>().ok())
                .unwrap_or(3);
        let config = fred::types::RedisConfig::from_url(&redis_url)?;

        let pool = fred::prelude::RedisPool::new(
            config,
            None,
            None,
            None,
            redis_pool_size,
        )?;

        pool.connect();
        pool.wait_for_connect().await?;
        Ok(Self{
            redis_conn : std::sync::Arc::new(pool)
        })

    }
}

#[cfg(not(feature = "cassandra"))]
impl StorageInterface for RedisClient {}

#[cfg(not(feature = "cassandra"))]
#[async_trait::async_trait]
impl Init for RedisClient {
    async fn prepare(&self) -> std::result::Result<(), Box<dyn std::error::Error>>{
        // add ping command
        /*
        sekf
        */
        Ok(())
    }
}