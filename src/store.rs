use crate::models::*;
use anyhow::Context;
use fred::prelude::ClientLike;
use scylla::CachingSession;
use std::env;
use openssl::ssl::{SslContextBuilder, SslVerifyMode, SslMethod, SslFiletype};
use std::path::PathBuf;
#[cfg(feature = "cassandra")]
use cassandra_cpp::*;
use reqwest::Client;

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
    #[cfg(feature = "replicate")]
    pub peer: RClient
}

#[derive(Clone)]
pub struct RClient{
    client : Client,
    addr : String
}

impl RClient{
    pub async fn call_next_cell(&self, pid: String, timestamp : i64) -> std::result::Result<(), Box<dyn std::error::Error>>{
        let req = format!("{}{}/{}",self.addr, pid,timestamp);
        self.client.get(req).send().await?;
        Ok(())
    }

    fn new() -> std::result::Result<Self, Box<dyn std::error::Error>>{
        let peer_addr = env::var("LAG_PEER_ADDR").context("lag peer addr not found")?;
        let client = Client::builder()
                    .pool_max_idle_per_host(10)
                    .pool_idle_timeout(std::time::Duration::from_secs(90))
                    .build()?;
        Ok(Self{
            client,
            addr : format!("http://{}:8000/record_lag/", peer_addr)
        })
    }
}

impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            db: dyn_clone::clone_box(&*(self.db)),
            #[cfg(feature = "replicate")]
            peer : self.peer.clone(),
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

            #[cfg(feature = "replicate")]
            peer : RClient::new()?
        })
    }
}

#[cfg(feature = "cassandra")]
use std::sync::Arc;
pub struct CassClient {
    pub cassandra_session: Arc<scylla::CachingSession>,
    pub account_session: Arc<scylla::CachingSession>,
}

impl Clone for CassClient{
    fn clone(&self) -> Self {
        Self {  cassandra_session : Arc::clone(&self.cassandra_session) , account_session: Arc::clone(&self.account_session) }
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
        // let mut cluster = Cluster::default();
        // cluster
        //     .set_contact_points(&url)?
        //     .set_port(port)?
        //     .set_credentials(&username, &password)?
        //     .set_load_balance_round_robin();
        
        // cluster.set_load_balance_dc_aware::<()>(datacenter.as_str(), 0, false)?;

        // let session = cluster.connect().await?;
        let cert_url = env::var("CASS_CERT_URL").context("CASSANDRA CERT URL NOT FOUND")?;
        let acc_keyspace = env::var("ACC_KEYSPACE").context("accounts keyspace not found")?;
        let scylla_url : Vec<String>= url.split(',').map(|ip| format!("{}:{}", ip, port)).collect();

        let certdir = tokio::fs::canonicalize(PathBuf::from(cert_url)).await?;
        let mut context_builder = SslContextBuilder::new(SslMethod::tls())?;
        context_builder.set_certificate_file(certdir.as_path(), SslFiletype::PEM)?;
        context_builder.set_verify(SslVerifyMode::NONE);
        //context_builder.set_hostname(url)?;


        let mut scylla_session = scylla::SessionBuilder::new()
            .known_nodes(scylla_url.clone());

        #[cfg(feature="keyspaces")]
        {
            scylla_session = scylla_session
                .ssl_context(Some(context_builder.build()))
                .user(username.clone(), password.clone());
        }     
        let updated_acc_session = scylla_session.host_filter(Arc::new(scylla::host_filter::DcHostFilter::new(datacenter.clone())))
            .use_keyspace(acc_keyspace, false)
            .build()
            .await?;

        let mut context_builder_r = SslContextBuilder::new(SslMethod::tls())?;
        context_builder_r.set_certificate_file(certdir.as_path(), SslFiletype::PEM)?;
        context_builder_r.set_verify(SslVerifyMode::NONE);

        let mut p_session = scylla::SessionBuilder::new()
            .known_nodes(scylla_url);
        #[cfg(feature="keyspaces")]
        {   p_session = p_session
                .ssl_context(Some(context_builder_r.build()))
                .user(username, password);
        }    
        let updated_session = p_session.host_filter(Arc::new(scylla::host_filter::DcHostFilter::new(datacenter)))
            .use_keyspace("payments", false)
            .build()
            .await?;
        let caching_session: CachingSession<std::collections::hash_map::RandomState> = CachingSession::from(updated_acc_session, 1024usize);
        let payment_session : CachingSession<std::collections::hash_map::RandomState> = CachingSession::from(updated_session, 1024usize);
        Ok(Self {
            cassandra_session: Arc::new(payment_session),
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
