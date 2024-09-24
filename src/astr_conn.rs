use stargate_grpc::*;
use bb8;
use std::str::FromStr;
use std::env;
use std::result::Result;
use anyhow::Context;
pub struct StargateClientManager {
    config : StargateConfig
}

impl StargateClientManager{
    pub fn new() -> Result<Self, Box<dyn std::error::Error>>{
        let config = StargateConfig::get_config_from_env()?;
        Ok(Self{
            config
        })
    }
}


pub struct StargateConfig {
    cluster_id : String,
    token : String,
    region : String,
}

impl StargateConfig{
    pub fn get_config_from_env() -> Result<Self, Box<dyn std::error::Error>>{
        let database_id = env::var("ASTRA_DB_ID").context("ASTRA DB ID not found")?;
        let region = env::var("ASTRA_REGION").context("ASTRA REGION NOT found")?;
        let token = env::var("ASTRA_DB_TOKEN").context("ASTRA_DB_TOKEN not found")?; 
        Ok(Self{
            cluster_id : database_id,
            token,
            region
        })
    }
}

#[async_trait::async_trait]
impl bb8::ManageConnection for StargateClientManager{
    type Connection  = StargateClient;
    type Error = String;

    async fn connect(&self) -> Result<Self::Connection, Self::Error>{
        let astra_uri = format!("https://{}-{}.apps.astra.datastax.com/stargate", self.config.cluster_id, self.config.region);
        let client = StargateClient::builder()
            .uri(astra_uri).map_err(|e| e.to_string())?
            .auth_token(AuthToken::from_str(self.config.token.as_str()).map_err(|e| e.to_string())?)
            .tls(Some(client::default_tls_config().map_err(|e| e.to_string())?))
            .connect()
            .await.map_err(|e| e.to_string())?;
        Ok(client)
    }

    async fn is_valid(&self, _conn : &mut Self::Connection) -> Result<(), Self::Error>{
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        false
    }

}

pub async fn get_pool() -> Result<bb8::Pool<StargateClientManager>, Box<dyn std::error::Error>> {
    let mgr = StargateClientManager::new()?;
    bb8::Pool::builder()
        .max_size(10u32)
        .build(mgr).await.map_err(|e| e.into())
}