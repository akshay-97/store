use crate::models::*;
use anyhow::Context;
use fred::prelude::ClientLike;
use scylla::CachingSession;
use std::env;

#[cfg(feature = "cassandra")]
use cassandra_cpp::*;

#[async_trait::async_trait]
pub trait Init {
    async fn prepare(&self) -> std::result::Result<(), Box<dyn std::error::Error>>;
}

#[async_trait::async_trait]
pub trait StorageInterface:
    dyn_clone::DynClone
    + PaymentIntentInterface
    + PaymentAttemptInterface
    + MerchantAccountInterface
    + Send
    + Sync
    + 'static
    + Init
{
}

pub struct App {
    pub db: Box<dyn StorageInterface>,
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            db: dyn_clone::clone_box(&*(self.db)),
        }
    }
}

impl App {
    pub async fn create_state() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            #[cfg(feature = "cassandra")]
            db: Box::new(CassClient::new().await?),

            #[cfg(feature = "redis")]
            db: Box::new(RedisClient::new().await?),
        })
    }
}

#[cfg(feature = "cassandra")]
use std::sync::Arc;
pub struct CassClient {
    //pub cassandra_session: Session,
    pub account_session: Arc<scylla::CachingSession>,
}

impl Clone for CassClient{
    fn clone(&self) -> Self {
        Self {  account_session: Arc::clone(&self.account_session) }
    }
}

#[cfg(feature = "cassandra")]
#[async_trait::async_trait]
impl Init for CassClient {
    async fn prepare(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // let _ = self
        //     .cassandra_session
        //     .execute(include_str!("schema.cql"))
        //     .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Init for RedisClient {
    async fn prepare(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[cfg(feature = "cassandra")]
impl StorageInterface for CassClient {}
#[cfg(feature = "redis")]
impl StorageInterface for RedisClient {}

#[cfg(feature = "cassandra")]
impl CassClient {
    pub async fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let url = env::var("CASSANDRA_URL").context("CASSANDRA_URL not found")?;
        let port = env::var("CASSANDRA_PORT")
            .ok()
            .and_then(|x| x.parse::<u16>().ok())
            .unwrap_or(9042);
        let password = env::var("CASSANDRA_PASSWORD").context("CASSANDRA_PASSWORD not found")?;
        let username = env::var("CASSANDRA_USERNAME").context("CASSANDRA_USERNAME not found")?;
        let datacenter = env::var("CASSANDRA_DC").context("datacenter not configured")?;
        set_level(LogLevel::DEBUG);
        let mut cluster = Cluster::default();
        cluster
            .set_contact_points(&url)?
            .set_port(port)?
            .set_credentials(&username, &password)?
            .set_load_balance_round_robin();
        
        cluster.set_load_balance_dc_aware::<()>(datacenter.as_str(), 0, false)?;

        let session = cluster.connect().await?;
        let acc_keyspace = env::var("ACC_KEYSPACE").context("accounts keyspace not found")?;
        let scylla_url : Vec<String>= url.split(',').map(|ip| format!("{}:{}", ip, port)).collect();
        let scylla_session = scylla::SessionBuilder::new()
            .known_nodes(scylla_url)
            .user(username, password)
            .host_filter(Arc::new(scylla::host_filter::DcHostFilter::new(datacenter)))
            .use_keyspace(acc_keyspace, false)
            .build()
            .await?;
        let caching_session: CachingSession<std::collections::hash_map::RandomState> = CachingSession::from(scylla_session, 1024usize);
        Ok(Self {
            //cassandra_session: session,
            account_session: Arc::new(caching_session),
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

#[derive(Clone)]
pub struct RedisClient {
    pub pool: fred::prelude::RedisPool,
    pub replicas: i64,
    pub timeout: i64,
}

impl RedisClient {
    pub async fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let connection_url = env::var("REDIS_CONNECTION_URL").context("REDIS URL not found")?;
        let pool_size = env::var("REDIS_POOL_SIZE").context("REDIS POOL size not found")?;
        let timeout = env::var("REDIS_WAIT_TIMEOUT").context("WAIT TIMEOUT not found")?;
        let replicas = env::var("REDIS_REPLICAS").context("Replicas not found")?;

        let config = fred::types::RedisConfig::from_url(&connection_url)?;
        let perf = fred::types::PerformanceConfig::default();
        let con_config = fred::types::ConnectionConfig::default();
        let pool = fred::prelude::RedisPool::new(
            config,
            Some(perf),
            Some(con_config),
            None,
            pool_size.parse()?,
        )?;

        pool.connect();
        pool.wait_for_connect().await?;
        Ok(Self {
            pool,
            replicas: replicas.parse()?,
            timeout: timeout.parse()?,
        })
    }
}
// struct RedisClient{
//     redis_client: String
// }

// impl RedisClient{
//     fn new() -> std::result::Result<Self, Box<dyn std::error::Error>>{
//         Ok(Self{
//             redis_client : "to impl".to_owned(),
//         })
//     }
// }
