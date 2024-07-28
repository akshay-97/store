use cassandra_cpp::*;
use crate::models::*;
use std::env;
use anyhow::Context;

#[async_trait::async_trait]
pub trait StorageInterface : dyn_clone::DynClone +
    PaymentIntentInterface + PaymentAttemptInterface + Send + Sync + 'static 
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
            db : Box::new(RedisClient::new()?),

        }
    )
    }   
}

#[derive(Clone)]
pub struct CassClient{
    pub cassandra_session: Session
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
