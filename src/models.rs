use crate::store::RedisClient;
use crate::types::*;
use anyhow::Context;

#[cfg(feature = "cassandra")]
use crate::store::CassClient;

#[cfg(feature = "cassandra")]
use cassandra_cpp::{BindRustType, LendingIterator};
use fred::prelude::{HashesInterface, ServerInterface};
use futures::{lock, StreamExt};
#[async_trait::async_trait]
pub trait PaymentIntentInterface {
    async fn create_intent(&self, payment_id: String) -> Result<(), Box<dyn std::error::Error>>;
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
    ) -> Result<(), Box<dyn std::error::Error>>;
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
use charybdis::{operations::Update, options::Consistency, serializers::FromJson};
#[async_trait::async_trait]
impl MerchantAccountInterface for CassClient {
    async fn create_account(&self, merchant_id : String) -> Result<(), Box<dyn std::error::Error>>{
        let account = MerchantAccount::new(merchant_id)?;
        let query = account.insert();
        let _result = crate::utils::time_wrapper(query.execute(self.account_session.as_ref()), "merchant_account", "CREATE").await?;
        Ok(())
    }

    async fn retrieve_account(&self, merchant_id : String) -> Result<(), Box<dyn std::error::Error>>{
        let query = MerchantAccount::find_by_merchant_id(merchant_id).consistency(Consistency::LocalQuorum);
        let _result = crate::utils::time_wrapper(query.execute(self.account_session.as_ref()), "merchant_account", "FIND").await?;
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        let p_a = PaymentAttempt::new(payment_id, version)?;
        let query = p_a.insert();
        let _result = crate::utils::time_wrapper(query.execute(&self.cassandra_session.as_ref()), "payment_attempts", "CREATE").await?;
        Ok(())
    }

    async fn retrieve_all<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = PaymentAttempt::find_by_merchant_id_and_payment_id("kaps".to_owned(), payment_id.to_owned());
        let result = crate::utils::time_wrapper(query.execute(self.cassandra_session.as_ref()), "payment_attempts", "FIND_ALL").await?;
        let ver = result.try_collect().await?.is_empty();
        if ver{
            return Err(
                "no records found".into()
            )
        }
        Ok(())
    }
    async fn update_attempt<'a>(
        &self,
        payment_intent_id: &'a str,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let update = UpdateMetadataPaymentAttempt {merchant_id : "kaps".to_string(), payment_id : payment_intent_id.to_owned(), attempt_id : version, connector_metadata : Some("AAAAB3NzaC1yc2EAAAADAQABAAACAQC+gCy875BbJDjy/KDczr84xswL1edCE82IkOCBeYuOPbmhR251K9r7UFjlXioa5UnMpRals/pMSkz9MF7yUTzPDg+NjfmsZ8rgHqTKJ7z/yUWEyxd3TcUh0xkkYMCfrA+9rLqgolBiasAOApBDYTi0BsBlfAgNIaTgg7xTX7PHUzceAvujJel1Q6V+bABnFvlDu6kWUhXlrPafWRPUSQz2wEsO7vqrE9UfP+CtuXrJ+t6pMbkVDGc0+JWaPJjBXMjxljBfZHw7UVbHmPlYYTwOGD/IWOgisFfvnutR4JvDZA5elWqkXj+ZEsOw4QFXw71o+b2YWrRBa8l+AFn//zPQvLB753wQVuhzmsidToLss2DfGLdrYKVuCTX7a7OhxKDYRYeyZgeqWK8xVqiyayXgvuxcZV2g+mHi2WuUGGJ6Ycj+JZ9Vh67EnplDmJAKCFXCenS4ou9rMCHqD6i9UVgzakzxy/wd5Cj6R26uKqKo8rZDw9D6zKDzF45NbVh+obAFh/9MuzSCaaL5pXWPUI0kI7iZ8lU7rC8HAj5HhynLZd+rfQazVo+qcoQMxO+A9+fFubru41Aku6siQgv6oXNiGSOcb4bgEDlCBQ/uQgNCn9Vdq3f1yWqC1eAQtwoB4YTE2DZrY1TVZiN202JQNweIIOQUANyKRVV2ITZketmeZQ==".to_string())};
        crate::utils::time_wrapper(update.update().execute(&self.cassandra_session), "payment_attempts", "UPDATE").await?;
        Ok(())
    }

    async fn retrieve_by_id(
        &self,
        payment_attempt_id : String,
        payment_id : String
    ) -> Result<(), Box<dyn std::error::Error>>{
        let query = PaymentAttempt::find_by_merchant_id_and_payment_id_and_attempt_id("kaps".to_owned(), payment_id, payment_attempt_id);
        let _result = crate::utils::time_wrapper(query.execute(self.cassandra_session.as_ref()), "payment_attempts", "FIND").await?;
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
    async fn create_intent(&self, payment_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let p_i = PaymentIntent::new(payment_id)?;
        let query = p_i.insert();
        let _result = crate::utils::time_wrapper(query.execute(self.cassandra_session.as_ref()), "payment_intents", "CREATE").await?;
        Ok(())
    }

    async fn retrieve_intent<'a>(
        &self,
        payment_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = PaymentIntent::find_by_merchant_id_and_payment_id("kaps".to_owned(), payment_id.to_owned());
        let _result = crate::utils::time_wrapper(query.execute(self.cassandra_session.as_ref()), "payment_intents", "FIND").await?;
        Ok(())
    }

    async fn update_intent<'a>(
        &self,
        payment_intent_id: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let update = UpdateStatusPaymentIntent { merchant_id : "kaps".to_string(), payment_id : payment_intent_id.to_owned(), status : "NEW".to_owned()};
        let _result = crate::utils::time_wrapper(update.update().execute(self.cassandra_session.as_ref()), "payment_intents", "UPDATE").await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl PaymentIntentInterface for RedisClient {
    async fn create_intent(&self, payment_id: String) -> Result<(), Box<dyn std::error::Error>> {
        let payment_intent = PaymentIntent::new(payment_id.clone())?;

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
        let mut payment_intent = PaymentIntent::new(payment_id.to_string())?;

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

#[async_trait::async_trait]
impl PaymentAttemptInterface for RedisClient {
    async fn create_attempt(
        &self,
        payment_id: String,
        version: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payment_attempt = PaymentAttempt::new(payment_id.clone(), version)?;

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
        let mut payment_attempt = PaymentAttempt::new(payment_intent_id.to_string(), version)?;

        payment_attempt.status = "Charged".to_string();

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
