pub trait StorageInterface : PaymentIntentInterface + PaymentAttemptInterface
        + Clone + Send + Sync + 'static
{}



pub trait PaymentIntentInterface{
    fn create(&self, payment_intent: PaymentIntent) -> Result<(), anyhow::Error>;
    fn retrieve<'a>(&self, payment_id : &'a str) -> Result<(), anyhow::Error>;
}


pub trait PaymentAttemptInterface{
    fn create(&self, payment_create: PaymentCreate) -> Result<(), anyhow::Error>;
    fn retrieve_by_id<'a>(&self, payment_attempt_id: &'a str) -> Result<(), anyhow::Error>;
    fn retrieve_all<'a>(&self, payment_id: &'a str) -> Result<(), anyhow::Error>;
}

use cassandra_cpp::*;


struct CassClient{
    cassandra_session: Session
}


impl CassClient{
    pub async fn new(config: CassConfig) -> Result<Self, anyhow::Error>{
        let mut cluster = Cluster::default();
        let session = cluster
            .set_contact_points(&config.url)?
            .set_credentials(&config.username, &config.password)?
            .set_load_balance_round_robin()
            .connect().await?;
        Ok(Self{
            cassandra_session : session
        })
    }
}


impl PaymentIntentInterface for CassClient{
    //TODO
}

impl PaymentAttemptInterface for CassClient{
    //TODO
}

struct RedisClient{
    redis_client: Any
}

impl RedisClient{

}

impl PaymentIntentInterface for RedisClient{
    //TODO
}

impl PaymentAttemptInterface for RedisClient{
    //TODO
}