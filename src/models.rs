use crate::store::RedisClient;
use crate::types::*;
use anyhow::Context;
use axum::response::IntoResponse;
use stargate_grpc::query::QueryBuilder;

#[cfg(feature = "cassandra")]
use crate::store::CassClient;

#[cfg(feature = "cassandra")]
use cassandra_cpp::{BindRustType, LendingIterator};
use fred::prelude::{HashesInterface, ServerInterface};
use futures::{lock, StreamExt};
#[async_trait::async_trait]
pub trait PaymentIntentInterface {
    async fn create_intent(&self, payment_id: String, use_client_id : bool) -> Result<PaymentIntentResponse, Box<dyn std::error::Error>>;
    async fn retrieve_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>>;
    async fn update_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait::async_trait]
pub trait PaymentAttemptInterface {
    async fn create_attempt(
        &self,
        payment_id: String,
        version: String,
    ) -> Result<PaymentAttemptResponse, Box<dyn std::error::Error>>;
    async fn retrieve_all<'a>(&self, payment_id: &'a str)
        -> Result<(), Box<dyn std::error::Error>>;
    async fn update_attempt<'a>(
        &self,
        payment_id: &'a str,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn retrieve_by_id(
        &self,
        payment_attempt_id : String,
        payment_id : String,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

#[async_trait::async_trait]
pub trait MerchantAccountInterface {
    async fn create_account(
        &self,
        merchant_id : String,
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn retrieve_account(
        &self,
        merchant_id : String,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

fn insert_intent_cql() -> String {
    "INSERT INTO payments.payment_intents (payment_id, merchant_id, status, amount, currency, amount_captured, customer_id, description, return_url, metadata, connector_id, shipping_address_id, billing_address_id, statement_descriptor_name, statement_descriptor_suffix, created_at, modified_at, last_synced, setup_future_usage, off_session, client_secret, active_attempt_id, business_country, business_label, order_details, allowed_payment_method_types, connector_metadata, feature_metadata, attempt_count, profile_id, merchant_decision, payment_link_id, payment_confirm_source, updated_by, surcharge_applicable, request_incremental_authorization, incremental_authorization_allowed, authorization_count, session_expiry, fingerprint_id, request_external_three_ds_authentication, charges, frm_metadata) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);" 
        .to_owned()
}
fn insert_attempt_cql() -> String {
    "INSERT INTO payments.payment_attempts ( payment_id, merchant_id, attempt_id, status, amount, currency, save_to_locker, connector, error_message, offer_amount, surcharge_amount, tax_amount, payment_method_id, payment_method, connector_transaction_id, capture_method, capture_on, confirm, authentication_type, created_at, modified_at, last_synced, cancellation_reason, amount_to_capture, mandate_id, browser_info, error_code, payment_token, connector_metadata, payment_experience, payment_method_type, payment_method_data, business_sub_label, straight_through_algorithm, preprocessing_step_id, mandate_details, error_reason, multiple_capture_count, connector_response_reference_id, amount_capturable, updated_by, merchant_connector_id, authentication_data, encoded_data, unified_code, unified_message, net_amount, external_three_ds_authentication_attempted, authentication_connector, authentication_id, mandate_data, fingerprint_id, payment_method_billing_address_id, charge_id, client_source, client_version ) VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? );"
        .to_owned()
}

fn select_payment_attempt_all() -> String {
    "SELECT * FROM payments.payment_attempts WHERE payment_id = ? AND merchant_id = ?;".to_owned()
}

fn update_attempt_cql() -> String {
    "UPDATE payments.payment_attempts set connector_metadata = ? WHERE payment_id = ? AND merchant_id = ? AND attempt_id = ?;"
      .to_owned()
}

fn update_intent_cql() -> String {
    "UPDATE payments.payment_intents set status = ? WHERE payment_id = ? AND merchant_id = ?;"
        .to_string()
}

fn retrieve_by_id() -> String{
    "SELECT * from payments.payment_attempts WHERE payment_id = ? AND merchant_id = ? AND attempt_id = ?;".to_string()
}

#[cfg(feature = "cassandra")]
use charybdis::operations::Insert;
use charybdis::options::Consistency;
#[cfg(feature = "cassandra")]
#[async_trait::async_trait]
impl MerchantAccountInterface for CassClient {
    async fn create_account(&self, merchant_id : String) -> Result<(), Box<dyn std::error::Error>>{
        let account = MerchantAccount::new(merchant_id)?;
        let query = account.insert().consistency(Consistency::EachQuorum);
        let _result = crate::utils::time_wrapper(query.execute(self.account_session.as_ref()), "merchant_account", "CREATE").await?;
        Ok(())
    }

    async fn retrieve_account(&self, merchant_id : String) -> Result<(), Box<dyn std::error::Error>>{
        let query = MerchantAccount::find_by_merchant_id(merchant_id).consistency(Consistency::LocalQuorum);
        let _result = crate::utils::time_wrapper(query.execute(self.account_session.as_ref()), "merchant_account", "FIND").await?;
    }
}
#[cfg(feature = "astra")]
#[async_trait::async_trait]
impl MerchantAccountInterface for crate::store::SGPool{
    async fn create_account(
        &self,
        merchant_id : String,
    ) -> Result<(), Box<dyn std::error::Error>> {Ok(())}

    async fn retrieve_account(
        &self,
        merchant_id : String,
    ) -> Result<(), Box<dyn std::error::Error>> {Ok(())}

}

#[cfg(feature = "astra")]
#[async_trait::async_trait]
impl PaymentIntentInterface for crate::store::SGPool {
    async fn create_intent(&self, payment_id: String, use_client_id : bool) -> Result<PaymentIntentResponse, Box<dyn std::error::Error>>{
        let query = stargate_grpc::Query::builder()
                .keyspace("payments")
                .query(insert_intent_cql().as_str())
                .consistency(stargate_grpc::Consistency::LocalQuorum);
        let pi = PaymentIntent::new(payment_id, use_client_id);
        let payment_id = pi.payment_id.clone();

        let updated_query = pi.bind_statement(query)?.build();
        let mut client = self.pool.get().await.unwrap();

        crate::utils::time_wrapper(client.execute_query(updated_query), "payment_intent", "CREATE").await?;
        //client.execute_query(updated_query).await?;
        
        Ok(PaymentIntentResponse{pi : payment_id})
    }
    async fn retrieve_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>>{
        let query = stargate_grpc::Query::builder()
                        .keyspace("payments")
                        .consistency(stargate_grpc::Consistency::LocalQuorum)
                        .query(retrieve_payment_cql().as_str())
                        .bind((payment_id, "kaps"))
                        .build();
        
        let mut client = self.pool.get().await.unwrap();
        //client.execute_query(query).await?;
        crate::utils::time_wrapper(client.execute_query(query), "payment_intent", "FIND").await?;
        Ok(())
    }
    async fn update_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>>{
        let query = stargate_grpc::Query::builder()
                        .keyspace("payments")
                        .consistency(stargate_grpc::Consistency::LocalQuorum)
                        .query(update_intent_cql().as_str())
                        .bind(("SUCCESS",payment_id, "kaps"))
                        .build();
        let mut client = self.pool.get().await.unwrap();
        crate::utils::time_wrapper(client.execute_query(query), "payment_intent", "UPDATE").await?;
        //client.execute_query(query).await?;           
        
        Ok(())
    }
}

#[cfg(feature = "astra")]
#[async_trait::async_trait]
impl PaymentAttemptInterface for crate::store::SGPool{
    async fn create_attempt(
        &self,
        payment_id: String,
        version: String,
    ) -> Result<PaymentAttemptResponse, Box<dyn std::error::Error>>{
        let query = stargate_grpc::Query::builder().keyspace("payments")
                .consistency(stargate_grpc::Consistency::LocalQuorum)
                .query(insert_attempt_cql().as_str());
        let pa = PaymentAttempt::new(payment_id , version);
        let payment_attempt_id = pa.attempt_id.clone();
        
        let updated_query = pa.bind_statement(query)?.build();

        let mut client = self.pool.get().await.unwrap();
        //client.execute_query(updated_query).await?; 
        crate::utils::time_wrapper(client.execute_query(updated_query), "payment_attempt", "CREATE").await?;
        Ok(PaymentAttemptResponse { pa: payment_attempt_id })

    }
    
    async fn retrieve_all<'a>(&self, payment_id: &'a str)
        -> Result<(), Box<dyn std::error::Error>>{
     let query = stargate_grpc::Query::builder()
                        .keyspace("payments")
                        .consistency(stargate_grpc::Consistency::LocalQuorum)
                        .query(select_payment_attempt_all().as_str())
                        .bind((payment_id, "kaps"))
                        .build();
        
        let mut client = self.pool.get().await.unwrap();
        //client.execute_query(query).await?;
        crate::utils::time_wrapper(client.execute_query(query), "payment_attempt", "FIND_ALL").await?;
        Ok(())
    }

    async fn update_attempt<'a>(
        &self,
        payment_id: &'a str,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>>{
     let query = stargate_grpc::Query::builder()
                        .keyspace("payments")
                        .consistency(stargate_grpc::Consistency::LocalQuorum)
                        .query(update_attempt_cql().as_str())
                        .bind((enum_parse(&get_large_value())?, payment_id, "kaps", version))
                        .build();
        
        let mut client = self.pool.get().await.unwrap();
        //client.execute_query(query).await?;
        crate::utils::time_wrapper(client.execute_query(query), "payment_attempt", "UPDATE").await?;
        Ok(())
    }

    async fn retrieve_by_id(
        &self,
        payment_attempt_id : String,
        payment_id : String
    ) -> Result<(), Box<dyn std::error::Error>>{
        let query = stargate_grpc::Query::builder()
                        .keyspace("payments")
                        .consistency(stargate_grpc::Consistency::LocalQuorum)
                        .query(retrieve_by_id().as_str())
                        .bind((payment_id, "kaps", payment_attempt_id))
                        .build();
        
        let mut client = self.pool.get().await.unwrap();
        //client.execute_query(query).await?;
         crate::utils::time_wrapper(client.execute_query(query), "payment_attempt", "FIND").await?;
        Ok(())    
    }
}

#[cfg(feature = "cassandra")]
#[async_trait::async_trait]
impl PaymentAttemptInterface for CassClient {
    async fn create_attempt(
        &self,
        payment_id: String,
        version: String,
    ) -> Result<PaymentAttemptResponse, Box<dyn std::error::Error>> {
        let mut statement = self.cassandra_session.statement(insert_attempt_cql());
        let _= statement.set_consistency(cassandra_cpp::Consistency::ONE)?;
        let pa = PaymentAttempt::new(payment_id,version);
        (&pa).populate_statement(&mut statement)?;
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "CREATE").await?;
        Ok(PaymentAttemptResponse { pa: pa.attempt_id })
    }

    async fn retrieve_all<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut statement = self
            .cassandra_session
            .statement(select_payment_attempt_all());

        statement.bind(0, payment_id)?;
        statement.bind(1, "kaps")?;
        statement.set_consistency(cassandra_cpp::Consistency::LOCAL_QUORUM)?;
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "FIND_ALL").await?;
        //let mut rows = rows.iter();

        let _ = rows.iter().next().context("No rows found")?;
        Ok(())
    }
    async fn update_attempt<'a>(
        &self,
        payment_intent_id: &'a str,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut statement = self.cassandra_session.statement(update_attempt_cql());
        for_opt(&mut statement, &Some(get_large_value()), 0)?;
        statement.bind(1, payment_intent_id)?;
        statement.bind(2, "kaps")?;
        statement.bind(3, version.as_str())?;
        statement.set_consistency(cassandra_cpp::Consistency::ONE)?;
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "UPDATE").await?;
        Ok(())
    }

    async fn retrieve_by_id(
        &self,
        payment_attempt_id : String,
        payment_id : String
    ) -> Result<(), Box<dyn std::error::Error>>{
        let mut statement = self.cassandra_session.statement(retrieve_by_id());
        statement.bind(0, payment_id.as_str())?;
        statement.bind(1, "kaps")?;
        statement.bind(2, payment_attempt_id.as_str())?;
        statement.set_consistency(cassandra_cpp::Consistency::LOCAL_QUORUM)?;
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "FIND").await?;
        if rows.row_count()  == 0u64 {
            return Err("no rows found".into())
        }
        Ok(())
    }
}

//TODO: convert to generated statements
fn retrieve_payment_cql() -> String {
    "SELECT * from payments.payment_intents WHERE payment_id = ? AND merchant_id = ?;".to_owned()
}

#[cfg(feature = "cassandra")]
#[async_trait::async_trait]
impl PaymentIntentInterface for CassClient {
    async fn create_intent(&self, payment_id: String) -> Result<PaymentIntentResponse, Box<dyn std::error::Error>> {
        let mut statement = self.cassandra_session.statement(insert_intent_cql());

        let pi = PaymentIntent::new(payment_id);
        (&pi).populate_statement(&mut statement)?;

        //println!("what is statement {:?} ", statement);
        statement.set_consistency(cassandra_cpp::Consistency::ONE)?;
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_intent", "CREATE").await
            .map_err(|e| {
                println!("intent create error {}", e.to_string());
                e
            })?;
        Ok(PaymentIntentResponse{pi : pi.payment_id})
    }

    async fn retrieve_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut statement = self.cassandra_session.statement(retrieve_payment_cql());

        statement.bind(0, payment_id)?;
        statement.bind(1, "kaps")?;
        statement.set_consistency(cassandra_cpp::Consistency::LOCAL_QUORUM)?;
    
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_intent", "FIND").await?;
        if rows.row_count() == 0u64 {
            return Err("no record found".into())
        }
        //TODO : deserialize
        Ok(())
    }

    async fn update_intent<'a>(
        &self,
        payment_intent_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut statement = self.cassandra_session.statement(update_intent_cql());

        statement.bind(0, "SUCCESS")?;
        statement.bind(1, payment_intent_id)?;

        statement.bind(2,"kaps")?;
        statement.set_consistency(cassandra_cpp::Consistency::ONE)?;

        let _rows =
            crate::utils::time_wrapper(statement.execute(), "payment_intent", "UPDATE").await?;
        Ok(())
    }
}

#[cfg(feature = "redis")]
#[async_trait::async_trait]
impl PaymentIntentInterface for RedisClient {
    async fn create_intent(&self, payment_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let payment_intent = PaymentIntent::new(payment_id.clone());

        crate::utils::time_wrapper(
            async {
                self.pool
                    .hsetnx::<(), _, _, _>(
                        format!("mer_kaps_pay_{}", payment_id),
                        format!("pi_{}", payment_id),
                        serde_json::to_vec(&payment_intent)
                            .unwrap_or_default()
                            .as_slice(),
                    )
                    .await
                    .map_err(|err| eprintln!("{:?}", err));

                self.pool.wait(self.replicas, self.timeout).await
            },
            "redis_payment_intent",
            "INSERT",
        )
        .await?;
        Ok(())
    }
    async fn retrieve_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = format!("mer_kaps_pay_{}", payment_id);
        let field = format!("pi_{}", payment_id);
        crate::utils::time_wrapper(
            self.pool.hget::<(), _, _>(key, field),
            "redis_payment_intent",
            "FIND",
        )
        .await?;
        Ok(())
    }

    async fn update_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut payment_intent = PaymentIntent::new(payment_id.to_string());

        payment_intent.status = String::from("SUCCESS");

        crate::utils::time_wrapper(
            async {
                self.pool
                    .hset::<(), _, _>(
                        format!("mer_kaps_pay_{}", payment_id),
                        (
                            format!("pi_{}", payment_id),
                            serde_json::to_vec(&payment_intent)
                                .unwrap_or_default()
                                .as_slice(),
                        ),
                    )
                    .await
                    .map_err(|err| eprintln!("{:?}", err));
                self.pool.wait(self.replicas, self.timeout).await
            },
            "redis_payment_intent",
            "UPDATE",
        )
        .await?;
        Ok(())
    }
}

#[cfg(feature = "redis")]
#[async_trait::async_trait]
impl PaymentAttemptInterface for RedisClient {
    async fn create_attempt(
        &self,
        payment_id: String,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payment_attempt = PaymentAttempt::new(payment_id.clone(), version);

        crate::utils::time_wrapper(
            async {
                self.pool
                    .hsetnx::<(), _, _, _>(
                        format!("mer_kaps_pay_{}", payment_id),
                        format!("pa_{}", payment_id),
                        serde_json::to_vec(&payment_attempt)
                            .unwrap_or_default()
                            .as_slice(),
                    )
                    .await
                    .map_err(|err| eprintln!("{:?}", err));
                self.pool.wait(self.replicas, self.timeout).await
            },
            "redis_payment_attempt",
            "INSERT",
        )
        .await?;
        Ok(())
    }

    async fn retrieve_all<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::utils::time_wrapper(
            self.pool
                .next()
                .hscan::<&str, &str>(
                    format!("mer_kaps_pay_{}", payment_id).as_str(),
                    "pa_*",
                    None,
                )
                .map(|val| val)
                .collect::<Vec<_>>(),
            "redis_payment_attempt",
            "FIND_ALL",
        )
        .await;
        Ok(())
    }
    async fn update_attempt<'a>(
        &self,
        payment_intent_id: &'a str,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut payment_attempt = PaymentAttempt::new(payment_intent_id.to_string(), version);

        payment_attempt.status = AttemptStatus::Charged;

        crate::utils::time_wrapper(
            async {
                self.pool
                    .hset::<(), _, _>(
                        format!("mer_kaps_pay_{}", payment_intent_id),
                        (
                            format!("pi_{}", payment_intent_id),
                            serde_json::to_vec(&payment_attempt)
                                .unwrap_or_default()
                                .as_slice(),
                        ),
                    )
                    .await
                    .map_err(|err| eprintln!("{:?}", err));
                self.pool.wait(self.replicas, self.timeout).await
            },
            "redis_payment_intent",
            "UPDATE",
        )
        .await?;
        Ok(())
    }
    async fn retrieve_by_id(&self, _ : String, _ : String) -> Result<(), Box<dyn std::error::Error>>{
        Ok(())
    }
}
