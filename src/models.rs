use std::collections::HashMap;

use crate::store::{DynamoClient, RedisClient};
use crate::types::*;
use anyhow::Context;
use axum::response::IntoResponse;

#[cfg(feature = "cassandra")]
use crate::store::CassClient;

#[cfg(feature = "cassandra")]
use cassandra_cpp::{BindRustType, LendingIterator};
use fred::prelude::{HashesInterface, ServerInterface};
use futures::{lock, StreamExt};

use aws_sdk_dynamodb::types::*;

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
        let _result = crate::utils::time_wrapper(query.execute(self.account_session.as_ref()), "merchant_account", "CREATE", None).await?;
        Ok(())
    }

    async fn retrieve_account(&self, merchant_id : String) -> Result<(), Box<dyn std::error::Error>>{
        let query = MerchantAccount::find_by_merchant_id(merchant_id).consistency(Consistency::LocalQuorum);
        let _result = crate::utils::time_wrapper(query.execute(self.account_session.as_ref()), "merchant_account", "FIND", None).await?;
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
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "CREATE", Some(pa.attempt_id.clone())).await?;
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
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "FIND_ALL", Some(payment_id.to_owned())).await?;
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
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "UPDATE", Some(payment_intent_id.to_owned())).await?;
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
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_attempt", "FIND", Some(payment_attempt_id)).await?;
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
    async fn create_intent(&self, payment_id: String, use_client_id : bool) -> Result<PaymentIntentResponse, Box<dyn std::error::Error>> {
        let mut statement = self.cassandra_session.statement(insert_intent_cql());

        let pi = PaymentIntent::new(payment_id, use_client_id);
        (&pi).populate_statement(&mut statement)?;

        //println!("what is statement {:?} ", statement);
        statement.set_consistency(cassandra_cpp::Consistency::ONE)?;
        let _rows = crate::utils::time_wrapper(statement.execute(), "payment_intent", "CREATE", Some(pi.payment_id.clone())).await
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
    
        let rows = crate::utils::time_wrapper(statement.execute(), "payment_intent", "FIND", Some(payment_id.to_owned())).await?;
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
            crate::utils::time_wrapper(statement.execute(), "payment_intent", "UPDATE", Some(payment_intent_id.to_owned())).await?;
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

#[async_trait::async_trait]
 impl PaymentIntentInterface for crate::store::DynamoClient {
    async fn create_intent(&self, payment_id: String, use_client_id : bool) -> Result<PaymentIntentResponse, Box<dyn std::error::Error>>{
        let pi = PaymentIntent::new(payment_id, use_client_id);
        let payment_id = pi.payment_id.clone();
        let query = pi.put_item(&self.client)?;
        query.send().await?;
        Ok(PaymentIntentResponse{pi : payment_id})
    }

    async fn retrieve_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>>{
        retrieve_helper(&self.client, payment_id).await?;
        Ok(())
    }
    async fn update_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>>{
        let mut record = retrieve_helper(&self.client, payment_id).await?;
        let entry = record.get_mut("status").ok_or(anyhow::Error::msg("field not found"))?;
        *entry = AttributeValue::S("SUCCESS".to_string());
        let mut builder = self.client.put_item().table_name("payment_intents".to_string());
        for (k,v) in record.into_iter(){
            builder = builder.item(k,v)
        }
        builder.send().await;
        Ok(())

    }
}

async fn retrieve_helper(client : &aws_sdk_dynamodb::Client, payment_id : &'_ str) -> Result<HashMap<String, AttributeValue>, Box<dyn std::error::Error>>
{
    let key_av = AttributeValue::S(payment_id.to_string());
    match client
            .query()
            .table_name("payment_intents".to_string())
            .key_condition_expression("#key = :value".to_string())
            .expression_attribute_names("#key".to_string(), "payment_id".to_string())
            .expression_attribute_values(":value".to_string(), key_av)
            .select(Select::AllAttributes)
            .send()
            .await
        {
            Ok(resp) => {
                if let Some(el) = resp.items
                    .and_then(|mut v| v.pop())
                {
                    return Ok(el)
                }
                Err("no pi found".into())
            },
            Err(e) => Err(e.into())
        }
}

#[async_trait::async_trait]
impl PaymentAttemptInterface for DynamoClient{
    async fn create_attempt(
        &self,
        payment_id: String,
        version: String,
    ) -> Result<PaymentAttemptResponse, Box<dyn std::error::Error>>
    {
        let pa = PaymentAttempt::new(payment_id, version);
        let attempt_id = pa.attempt_id.clone();
        let query = pa.put_item(&self.client)?;
        query.send().await;
        Ok(PaymentAttemptResponse { pa: attempt_id })
    }

    async fn retrieve_all<'a>(&self, payment_id: &'a str)
        -> Result<(), Box<dyn std::error::Error>>
    {
        let _arr = retrieve_attempt_helper(&self.client, payment_id, None).await?;
        Ok(())
    }

    async fn update_attempt<'a>(
        &self,
        payment_id: &'a str,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>>
    {
        let mut obj =
            retrieve_attempt_helper(&self.client, payment_id , Some(version.as_str()))
            .await?
            .pop()
            .ok_or(anyhow::Error::msg("no record found"))?;
        
        let entry = obj.get_mut("status").ok_or(anyhow::Error::msg("field not found"))?;
        *entry = AttributeValue::S("SUCCESS".to_string());
        let mut builder = self.client.put_item().table_name("payment_attempts".to_string());
        for (k,v) in obj.into_iter(){
            builder = builder.item(k,v)
        }
        builder.send().await;
        Ok(())

    }

    async fn retrieve_by_id(
        &self,
        payment_attempt_id : String,
        payment_id : String,
    ) -> Result<(), Box<dyn std::error::Error>>
    {
        let _obj =
            retrieve_attempt_helper(&self.client, payment_id.as_str(), Some(payment_attempt_id.as_str()))
            .await?
            .pop()
            .ok_or(anyhow::Error::msg("no record found"))?;
        Ok(())
    }
}


async fn retrieve_attempt_helper(client : &aws_sdk_dynamodb::Client, payment_id : &'_ str, version : Option<&'_ str>) -> Result<Vec<HashMap<String, AttributeValue>>, Box<dyn std::error::Error>>
{
    let key_av = AttributeValue::S(payment_id.to_string());
    
    let mut queryBuilder = client
            .query()
            .table_name("payment_intents".to_string())
            .key_condition_expression("#key = :value".to_string())
            .expression_attribute_names("#key".to_string(), "payment_id".to_string())
            .expression_attribute_values(":value".to_string(), key_av);
    match version {
        Some(v) => {
            let skey_av =  AttributeValue::S(v.to_string());
            queryBuilder = queryBuilder.key_condition_expression("#key = :value".to_string())
                .expression_attribute_names("#key".to_string(), "attempt_id".to_string())
                .expression_attribute_values(":value".to_string(), skey_av);
        },
        None => {}
    }
    
    match queryBuilder
            .select(Select::AllAttributes)
            .send()
            .await
    {
        Ok(resp) => {
            if let Some(el) = resp.items
            {
                return Ok(el)
            }
            Err("no pi found".into())
        },
        Err(e) => Err(e.into())
    }
}


#[async_trait::async_trait]
impl MerchantAccountInterface for DynamoClient{
    async fn create_account(
        &self,
        merchant_id : String,
    ) -> Result<(), Box<dyn std::error::Error>>{Ok(())}

    async fn retrieve_account(
        &self,
        merchant_id : String,
    ) -> Result<(), Box<dyn std::error::Error>>{Ok(())}
}