use crate::{types::*, store::RedisClient};
use crate::store::CassClient;
use cassandra_cpp::{BindRustType, LendingIterator};
use anyhow::Context;

#[async_trait::async_trait]
pub trait PaymentIntentInterface{
    async fn create_intent(&self, payment_id: String) -> Result<(), Box<dyn std::error::Error>>;
    async fn retrieve_intent<'a>(&self, payment_id : &'a str) -> Result<(), Box<dyn std::error::Error>>;
    async fn update_intent<'a>(&self, payment_id : &'a str) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait::async_trait]
pub trait PaymentAttemptInterface{
    async fn create_attempt(&self, payment_id : String) -> Result<(), Box<dyn std::error::Error>>;
    async fn retrieve_by_id<'a>(&self, payment_attempt_id: &'a str) -> Result<(), Box<dyn std::error::Error>>;
    async fn retrieve_all<'a>(&self, payment_id: &'a str) -> Result<(), Box<dyn std::error::Error>>;
    async fn update_attempt<'a>(&self, payment_attempt_id: &'a str) -> Result<(), Box<dyn std::error::Error>>;
}

fn insert_intent_cql() -> String{
    "INSERT INTO payments.payment_intents (payment_id, merchant_id, status, amount, currency, amount_captured, customer_id, description, return_url, metadata, connector_id, shipping_address_id, billing_address_id, statement_descriptor_name, statement_descriptor_suffix, created_at, modified_at, last_synced, setup_future_usage, off_session, client_secret, active_attempt_id, business_country, business_label, order_details, allowed_payment_method_types, connector_metadata, feature_metadata, attempt_count, profile_id, merchant_decision, payment_link_id, payment_confirm_source, updated_by, surcharge_applicable, request_incremental_authorization, incremental_authorization_allowed, authorization_count, session_expiry, fingerprint_id, request_external_three_ds_authentication, charges, frm_metadata) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)" 
        .to_owned()
}
fn insert_attempt_cql() -> String{
    "INSERT INTO payments.payment_attempts ( payment_id, merchant_id, attempt_id, status, amount, currency, save_to_locker, connector, error_message, offer_amount, surcharge_amount, tax_amount, payment_method_id, payment_method, connector_transaction_id, capture_method, capture_on, confirm, authentication_type, created_at, modified_at, last_synced, cancellation_reason, amount_to_capture, mandate_id, browser_info, error_code, payment_token, connector_metadata, payment_experience, payment_method_type, payment_method_data, business_sub_label, straight_through_algorithm, preprocessing_step_id, mandate_details, error_reason, multiple_capture_count, connector_response_reference_id, amount_capturable, updated_by, merchant_connector_id, authentication_data, encoded_data, unified_code, unified_message, net_amount, external_three_ds_authentication_attempted, authentication_connector, authentication_id, mandate_data, fingerprint_id, payment_method_billing_address_id, charge_id, client_source, client_version ) VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? );"
        .to_owned()
}

fn select_payment_attempt_all() -> String{
    "SELECT * FROM payments.payment_attempts WHERE merchant_id = ? AND payment_id = ?;"
        .to_owned()
}


#[async_trait::async_trait]
impl PaymentAttemptInterface for CassClient {
    async fn create_attempt(&self, payment_id: String) -> Result<(), Box<dyn std::error::Error>>{
        let mut statement = self.cassandra_session.statement(insert_attempt_cql());
        
        PaymentAttempt::new(payment_id).populate_statement(&mut statement)?;
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "CREATE").await?;
        Ok(())
    }

    async fn retrieve_by_id<'a>(&self, payment_attempt_id: &'a str) -> Result<(),Box<dyn std::error::Error>>{
        let mut statement = self.cassandra_session.statement(include_str!("select_payment_attempt.cql"));

        statement.bind(0, payment_attempt_id)?;
        statement.bind(1, "kaps")?;
    
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "FIND").await?;
        //let mut rows = rows.iter();
    
        let _ = rows.iter().next().context("No rows found")?;
        Ok(())
    }

    async fn retrieve_all<'a>(&self, payment_id : &'a str) -> Result<(), Box<dyn std::error::Error>>{
        let mut statement = self.cassandra_session.statement(select_payment_attempt_all());

        statement.bind(0, "kaps")?;
        statement.bind(1, payment_id)?;
    
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "FIND_ALL").await?;
        //let mut rows = rows.iter();
    
        let _ = rows.iter().next().context("No rows found")?;
        Ok(())
    }
    async fn update_attempt<'a>(&self, _payment_attempt_id : &'a str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

//TODO: convert to generated statements
fn retrieve_payment_cql() -> String{
    "SELECT * from payments.payment_intents WHERE payment_id ? AND merchant_id ?;"
        .to_owned()
}

#[async_trait::async_trait]
impl PaymentIntentInterface for CassClient {
    async fn create_intent(&self, payment_id: String) -> Result<(), Box<dyn std::error::Error>>{
        let mut statement = self.cassandra_session.statement(insert_intent_cql());
        
        PaymentIntent::new(payment_id).populate_statement(&mut statement)?;
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_intent", "CREATE").await?;
        Ok(())
    }

    async fn retrieve_intent<'a>(&self, payment_id: &'a str) -> Result<(), Box<dyn std::error::Error>>{
        let mut statement = self.cassandra_session.statement(retrieve_payment_cql());

        statement.bind(0, payment_id)?;
        statement.bind(1, "kaps")?;
    
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_intent", "FIND").await?;
        let _row = rows.iter().next().context("No rows found")?;
        //TODO : deserialize
        Ok(())
    }

    async fn update_intent<'a>(&self, _payment_intent_id : &'a str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl PaymentAttemptInterface for RedisClient {
    async fn create_attempt(&self, payment_id: String) -> Result<() , Box<dyn std::error::Error>>{
        let key = format!("mid_kaps_payment_id_{}", payment_id);
        let field = format!("payment_id_{}", payment_id);
        let value = PaymentAttempt::new(payment_id);
        self.execute(
            key,
            Hsetnx(value, field) , // enum ; hsetnx<v>, hset<v>, hget
        ).await?;
        /*
        self.redis_conn.hsetnx(key, field, value)
        pub async fn wait<C: ClientLike>(client: &C, numreplicas: i64, timeout: i64) -> Result<RedisValue, RedisError>
        */
        Ok(())
    }
}