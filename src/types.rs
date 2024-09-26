use std::time::UNIX_EPOCH;

use charybdis::scylla::{CqlValue, FromCqlVal};
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use charybdis::macros::charybdis_model;
use charybdis::types::{Text, Time, Timestamp};
use chrono::Utc;

pub const cell: &'static str = env!("CELL", "couldnt find cell env");



trait GenerateId{
    fn get_table_name() -> String;
    fn generate_id<'a>(client_identifier : &'a str) -> String;
}
#[derive(Serialize, Deserialize)]
pub struct PaymentAttempt {
    pub payment_id: String,
    pub merchant_id: String,
    pub attempt_id: String,
    pub status: AttemptStatus,
    pub amount: i64,
    pub currency: Option<Currency>,
    pub save_to_locker: Option<bool>,
    pub connector: Option<String>,
    pub error_message: Option<String>,
    pub offer_amount: Option<i64>,
    pub surcharge_amount: Option<i64>,
    pub tax_amount: Option<i64>,
    pub payment_method_id: Option<String>,
    pub payment_method: Option<PaymentMethod>,
    pub connector_transaction_id: Option<String>,
    pub capture_method: Option<CaptureMethod>,
    pub capture_on: Option<time::PrimitiveDateTime>,
    pub confirm: bool,
    pub authentication_type: Option<AuthenticationType>,
    pub created_at: PrimitiveDateTime,
    pub modified_at: PrimitiveDateTime,
    pub last_synced: Option<PrimitiveDateTime>,
    pub cancellation_reason: Option<String>,
    pub amount_to_capture: Option<i64>,
    pub mandate_id: Option<String>,
    pub browser_info: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub payment_token: Option<String>,
    //TODO: check cql map type conversion
    pub connector_metadata: Option<serde_json::Value>,
    pub payment_experience: Option<PaymentExperience>,
    pub payment_method_type: Option<PaymentMethodType>,
    pub payment_method_data: Option<serde_json::Value>,
    pub business_sub_label: Option<String>,
    pub straight_through_algorithm: Option<serde_json::Value>,
    pub preprocessing_step_id: Option<String>,
    // providing a location to store mandate details intermediately for transaction
    pub mandate_details: Option<MandateDataType>,
    pub error_reason: Option<String>,
    pub multiple_capture_count: Option<i16>,
    // reference to the payment at connector side
    pub connector_response_reference_id: Option<String>,
    pub amount_capturable: i64,
    pub updated_by: String,
    pub merchant_connector_id: Option<String>,
    pub authentication_data: Option<serde_json::Value>,
    pub encoded_data: Option<String>,
    pub unified_code: Option<String>,
    pub unified_message: Option<String>,
    pub net_amount: Option<i64>,
    pub external_three_ds_authentication_attempted: Option<bool>,
    pub authentication_connector: Option<String>,
    pub authentication_id: Option<String>,
    pub mandate_data: Option<MandateDetails>,
    pub fingerprint_id: Option<String>,
    pub payment_method_billing_address_id: Option<String>,
    pub charge_id: Option<String>,
    pub client_source: Option<String>,
    pub client_version: Option<String>,
}

#[cfg(feature = "cassandra")]
use cassandra_cpp::{BindRustType, Statement};

impl PaymentAttempt {
    #[cfg(feature = "cassandra")]
    pub fn populate_statement(
        &self,
        stmt: &mut Statement,
    ) -> Result<(), Box<dyn std::error::Error>> {
        stmt.bind(0, self.payment_id.as_str())?;
        stmt.bind(1, self.merchant_id.as_str())?;
        stmt.bind(2, self.attempt_id.as_str())?;
        stmt.bind(3, enum_parse(&self.status)?.as_str())?;
        stmt.bind(4, self.amount)?;
        for_opt(stmt, &self.currency, 5)?;
        e_for_opt(stmt, &self.save_to_locker, 6)?;

        opt_string(stmt, &self.connector, 7)?;
        opt_string(stmt, &self.error_message, 8)?;
        e_for_opt(stmt, &self.offer_amount, 9)?;
        e_for_opt(stmt, &self.surcharge_amount, 10)?;
        e_for_opt(stmt, &self.tax_amount, 11)?;
        opt_string(stmt, &self.payment_method_id, 12)?;
        for_opt(stmt, &self.payment_method, 13)?;
        opt_string(stmt, &self.connector_transaction_id, 14)?;
        for_opt(stmt, &self.capture_method, 15)?;
        for_opt(stmt, &self.capture_on, 16)?;
        stmt.bind(17, self.confirm)?;
        for_opt(stmt, &self.authentication_type, 18)?;
        stmt.bind(19, enum_parse(&self.created_at)?.as_str())?;
        stmt.bind(20, enum_parse(&self.modified_at)?.as_str())?;
        for_opt(stmt, &self.last_synced, 21)?;

        opt_string(stmt, &self.cancellation_reason, 22)?;
        e_for_opt(stmt, &self.amount_to_capture, 23)?;
        opt_string(stmt, &self.mandate_id, 24)?;
        for_opt(stmt, &self.browser_info, 25)?;
        opt_string(stmt, &self.error_code, 26)?;
        opt_string(stmt, &self.payment_token, 27)?;
        for_opt(stmt, &self.connector_metadata, 28)?;
        for_opt(stmt, &self.payment_experience, 29)?;
        for_opt(stmt, &self.payment_method_type, 30)?;
        for_opt(stmt, &self.payment_method_data, 31)?;
        opt_string(stmt, &self.business_sub_label, 32)?;

        for_opt(stmt, &self.straight_through_algorithm, 33)?;
        opt_string(stmt, &self.preprocessing_step_id, 34)?;
        for_opt(stmt, &self.mandate_details, 35)?;
        opt_string(stmt, &self.error_reason, 36)?;
        e_for_opt(stmt, &self.multiple_capture_count, 37)?;
        opt_string(stmt, &self.connector_response_reference_id, 38)?;
        stmt.bind(39, self.amount_capturable)?;
        stmt.bind(40, self.updated_by.as_str())?;
        opt_string(stmt, &self.merchant_connector_id, 41)?;
        for_opt(stmt, &self.authentication_data, 42)?;
        opt_string(stmt, &self.encoded_data, 43)?;
        opt_string(stmt, &self.unified_code, 44)?;
        opt_string(stmt, &self.unified_message, 45)?;
        e_for_opt(stmt, &self.net_amount, 46)?;
        e_for_opt(stmt, &self.external_three_ds_authentication_attempted, 47)?;
        opt_string(stmt, &self.authentication_connector, 48)?;
        opt_string(stmt, &self.authentication_id, 49)?;
        for_opt(stmt, &self.mandate_data, 50)?;
        opt_string(stmt, &self.fingerprint_id, 51)?;
        opt_string(stmt, &self.payment_method_billing_address_id, 52)?;
        opt_string(stmt, &self.charge_id, 53)?;

        opt_string(stmt, &self.client_source, 54)?;
        opt_string(stmt, &self.client_version, 55)?;

        Ok(())
    }

    pub fn new(i: String, version: String) -> Self {
        Self {
            payment_id: i.clone(),
            merchant_id: "kaps".to_owned(),
            attempt_id: Self::generate_id(version.as_str()),
            status: AttemptStatus::AuthenticationFailed,
            amount: i64::MAX,
            currency: Some(Currency::USD),
            save_to_locker: Some(false),
            connector: Some("randomeString12412953w23421".to_owned()),
            error_message: Some("randomeString12412953w23421".to_owned()),
            offer_amount: Some(i64::MAX),
            surcharge_amount: Some(i64::MAX),
            tax_amount: Some(i64::MAX),
            payment_method_id: Some("randomeString12412953w23421".to_owned()),
            payment_method: Some(PaymentMethod::Card),
            connector_transaction_id: Some("randomeString12412953w23421".to_owned()),
            capture_method: Some(CaptureMethod::Automatic),
            //#[serde(default, with = "common_utils::custom_serde::iso8601::option")]
            capture_on: Some(time::PrimitiveDateTime::MAX),
            confirm: true,
            authentication_type: Some(AuthenticationType::NoThreeDs),
            //#[serde(with = "common_utils::custom_serde::iso8601")]
            created_at: time::PrimitiveDateTime::MAX,
            // #[serde(with = "common_utils::custom_serde::iso8601")]
            modified_at: time::PrimitiveDateTime::MAX,
            //#[serde(default, with = "common_utils::custom_serde::iso8601::option")]
            last_synced: Some(time::PrimitiveDateTime::MAX),
            cancellation_reason: Some("randomeString12412953w23421".to_owned()),
            amount_to_capture: Some(i64::MAX),
            mandate_id: Some("randomeString12412953w23421".to_owned()),
            browser_info: None,
            error_code: Some("randomeString12412953w23421".to_owned()),
            payment_token: Some("randomeString12412953w23421".to_owned()),
            connector_metadata: Some(get_large_value()), //Value
            payment_experience: None,
            payment_method_type: None,
            payment_method_data: Some(get_large_value()), //Value
            business_sub_label: Some("randomeString12412953w23421".to_owned()),
            straight_through_algorithm: None,
            preprocessing_step_id: Some("randomeString12412953w23421".to_owned()),
            // providing a location to store mandate details intermediately for transaction
            mandate_details: None,
            error_reason: Some("randomeString12412953w23421".to_owned()),
            multiple_capture_count: None,
            // reference to the payment at connector side
            connector_response_reference_id: Some("randomeString12412953w23421".to_owned()),
            amount_capturable: i64::MAX,
            updated_by: "foo".to_owned(),

            merchant_connector_id: Some("randomeString12412953w23421".to_owned()),
            authentication_data: Some(get_large_value()), //Value
            encoded_data: Some("randomeString12412953w23421".to_owned()),
            unified_code: Some("randomeString12412953w23421".to_owned()),
            unified_message: Some("randomeString12412953w23421".to_owned()),
            net_amount: Some(i64::MAX),
            external_three_ds_authentication_attempted: None,
            authentication_connector: Some("randomeString12412953w23421".to_owned()),
            authentication_id: Some("randomeString12412953w23421".to_owned()),
            mandate_data: None,
            fingerprint_id: Some("randomeString12412953w23421".to_owned()),
            payment_method_billing_address_id: Some("randomeString12412953w23421".to_owned()),
            charge_id: Some("randomeString12412953w23421".to_owned()),
            client_source: Some("randomeString12412953w23421".to_owned()),
            client_version: Some("randomeString12412953w23421".to_owned()),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PaymentIntent {
    pub payment_id: String,
    pub merchant_id: String,
    pub status: String,
    pub amount: i64,
    pub currency: Option<Currency>,
    pub amount_captured: Option<i64>,
    pub customer_id: Option<String>,
    pub description: Option<String>,
    pub return_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub connector_id: Option<String>,
    pub shipping_address_id: Option<String>,
    pub billing_address_id: Option<String>,
    pub statement_descriptor_name: Option<String>,
    pub statement_descriptor_suffix: Option<String>,
    pub created_at: PrimitiveDateTime,
    pub modified_at: PrimitiveDateTime,
    pub last_synced: Option<PrimitiveDateTime>,
    pub setup_future_usage: Option<String>,
    pub off_session: Option<bool>,
    pub client_secret: Option<String>,
    pub active_attempt_id: String,
    pub business_country: Option<String>,
    pub business_label: Option<String>,
    pub order_details: Option<Vec<serde_json::Value>>,
    pub allowed_payment_method_types: Option<serde_json::Value>,
    pub connector_metadata: Option<serde_json::Value>,
    pub feature_metadata: Option<serde_json::Value>,
    pub attempt_count: i16,
    pub profile_id: Option<String>,
    // Denotes the action(approve or reject) taken by merchant in case of manual review.
    // Manual review can occur when the transaction is marked as risky by the frm_processor, payment processor or when there is underpayment/over payment incase of crypto payment
    pub merchant_decision: Option<String>,
    pub payment_link_id: Option<String>,
    pub payment_confirm_source: Option<String>,

    pub updated_by: String,
    pub surcharge_applicable: Option<bool>,
    pub request_incremental_authorization: Option<String>,
    pub incremental_authorization_allowed: Option<bool>,
    pub authorization_count: Option<i32>,
    pub session_expiry: Option<PrimitiveDateTime>,
    pub fingerprint_id: Option<String>,
    pub request_external_three_ds_authentication: Option<bool>,
    pub charges: Option<serde_json::Value>,
    pub frm_metadata: Option<serde_json::Value>,
}

impl PaymentIntent {
    pub fn new(i: String, use_client_id : bool) -> Self {
        PaymentIntent {
            payment_id: if use_client_id {i} else {Self::generate_id(&i)},
            merchant_id: "kaps".to_string(),
            status: "Processing".to_string(),
            amount: 1234_i64,
            currency: Some(Currency::USD),
            amount_captured: None,
            customer_id: None,
            description: Some("randomeString12412953w23421".to_owned()),
            return_url: Some("randomeString12412953w23421".to_owned()),
            metadata: None,
            connector_id: Some("randomeString12412953w23421".to_owned()),
            shipping_address_id: Some("randomeString12412953w23421".to_owned()),
            billing_address_id: Some("randomeString12412953w23421".to_owned()),
            statement_descriptor_name: Some("randomeString12412953w23421".to_owned()),
            statement_descriptor_suffix: Some("randomeString12412953w23421".to_owned()),
            created_at: time::PrimitiveDateTime::MAX,
            modified_at: time::PrimitiveDateTime::MAX,
            last_synced: Some(time::PrimitiveDateTime::MAX),
            setup_future_usage: Some(String::from("OffSession")),
            off_session: Some(false),
            client_secret: Some("randomeString12412953w23421".to_owned()),
            active_attempt_id: "asdasdas".to_string(),

            business_country: None,
            business_label: Some("randomeString12412953w23421".to_owned()),
            order_details: None, //Option<Vec<pii::SecretSerdeValue>>,
            allowed_payment_method_types: None, //Value
            connector_metadata: Some(get_large_value()), //Value
            feature_metadata: Some(get_large_value()), //Value
            attempt_count: i16::MAX,
            profile_id: Some("randomeString12412953w23421".to_owned()),
            merchant_decision: Some("randomeString12412953w23421".to_owned()),
            payment_link_id: Some("randomeString12412953w23421".to_owned()),
            payment_confirm_source: None,
            updated_by: "asdasds".to_string(),

            surcharge_applicable: Some(false),
            request_incremental_authorization: None,
            incremental_authorization_allowed: Some(false),
            authorization_count: None,
            session_expiry: Some(time::PrimitiveDateTime::MAX),
            fingerprint_id: Some("randomeString12412953w23421".to_owned()),
            request_external_three_ds_authentication: Some(false),
            charges: None,      //Option<pii::SecretSerdeValue>,
            frm_metadata: None, //Option<pii::SecretSerdeValue>,
        }
    }

    #[cfg(feature = "cassandra")]
    pub fn populate_statement(
        &self,
        stmt: &mut Statement,
    ) -> Result<(), Box<dyn std::error::Error>> {
        stmt.bind(0, self.payment_id.as_str())?;
        stmt.bind(1, self.merchant_id.as_str())?;
        stmt.bind(2, self.status.as_str())?;
        #[cfg(feature = "cassandra")]
        stmt.bind(3, self.amount)?;
        for_opt(stmt, &self.currency, 4)?;
        e_for_opt(stmt, &self.amount_captured, 5)?;
        opt_string(stmt, &self.customer_id, 6)?;
        opt_string(stmt, &self.description, 7)?;
        opt_string(stmt, &self.return_url, 8)?;
        for_opt(stmt, &self.metadata, 9)?;
        opt_string(stmt, &self.connector_id, 10)?;
        opt_string(stmt, &self.shipping_address_id, 11)?;
        opt_string(stmt, &self.billing_address_id, 12)?;
        opt_string(stmt, &self.statement_descriptor_name, 13)?;
        opt_string(stmt, &self.statement_descriptor_suffix, 14)?;
        for_opt(stmt, &Some(self.created_at), 15)?;
        for_opt(stmt, &Some(self.modified_at), 16)?;
        for_opt(stmt, &self.last_synced, 17)?;
        opt_string(stmt, &self.setup_future_usage, 18)?;
        e_for_opt(stmt, &self.off_session, 19)?;
        opt_string(stmt, &self.client_secret, 20)?;
        stmt.bind(21, self.active_attempt_id.as_str())?;
        opt_string(stmt, &self.business_country, 22)?;
        opt_string(stmt, &self.business_label, 23)?;
        for_opt(stmt, &self.order_details, 24)?;
        for_opt(stmt, &self.allowed_payment_method_types, 25)?;
        for_opt(stmt, &self.connector_metadata, 26)?;
        for_opt(stmt, &self.feature_metadata, 27)?;
        stmt.bind(28, self.attempt_count)?;
        opt_string(stmt, &self.profile_id, 29)?;
        opt_string(stmt, &self.merchant_decision, 30)?;
        opt_string(stmt, &self.payment_link_id, 31)?;
        opt_string(stmt, &self.payment_confirm_source, 32)?;
        stmt.bind(33, self.updated_by.as_str())?;
        e_for_opt(stmt, &self.surcharge_applicable, 34)?;
        opt_string(stmt, &self.request_incremental_authorization, 35)?;
        e_for_opt(stmt, &self.incremental_authorization_allowed, 36)?;
        e_for_opt(stmt, &self.authorization_count, 37)?;
        for_opt(stmt, &self.session_expiry, 38)?;
        opt_string(stmt, &self.fingerprint_id, 39)?;
        e_for_opt(stmt, &self.request_external_three_ds_authentication, 40)?;
        for_opt(stmt, &self.charges, 41)?;
        for_opt(stmt, &self.frm_metadata, 42)?;
        Ok(())
    }
}

pub fn get_large_value() -> serde_json::Value {
    serde_json::json!({
      "merchant_id": "merchantasd",
      "locker_id": "m0010",
      "merchant_name": "NewAge Retailer",
      "merchant_details": {
        "primary_contact_person": "John Test",
        "primary_email": "JohnTest@test.com",
        "primary_phone": "sunt laborum",
        "secondary_contact_person": "John Test2",
        "secondary_email": "JohnTest2@test.com",
        "secondary_phone": "cillum do dolor id",
        "website": "www.example.com",
        "about_business": "Online Retail with a wide selection of organic products for North America",
        "address": {
          "line1": "1467",
          "line2": "Harrison Street",
          "line3": "Harrison Street",
          "city": "San Fransico",
          "state": "California",
          "zip": "94122",
          "country": "US"
        }
      },
      "return_url": "https://google.com/success",
      "webhook_details": {
        "webhook_version": 123124.12312412,
        "webhook_username": "ekart_retail",
        "webhook_password": "password_ekart@123",
        "payment_created_enabled": true,
        "payment_succeeded_enabled": true,
        "payment_failed_enabled": true
      },
      "routing_algorithm": {
        "type": "single",
        "data": "stripe"
      },
      "sub_merchants_enabled": false,
      "metadata": {
        "city": "NY",
        "unit": "245"
      },
      "primary_business_details": [
        {
          "country": "US",
          "business": "default"
        }
      ]
    })
}

impl GenerateId for PaymentIntent{
    fn get_table_name() -> String {
        "pai".to_string()
    }

    fn generate_id<'a>(client_identifier : &'a str) -> String {
        let now = Utc::now().timestamp_millis();
        let prefix = Self::get_table_name();
        format!("{}_{}_{}_{}", cell, prefix.as_str(), now.to_string().as_str(), client_identifier)

    }
}


impl GenerateId for PaymentAttempt{
    fn get_table_name() -> String {
        "paa".to_string()
    }
    
    fn generate_id<'a>(client_identifier : &'a str) -> String {
        let now = Utc::now().timestamp_millis();
        format!("{}_{}_{}_{}", cell, Self::get_table_name().as_str(), now, client_identifier)
    }
}

fn enum_parse<T: serde::Serialize>(em: &T) -> Result<String, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string(em)?)
}

#[cfg(feature = "cassandra")]
pub fn for_opt<T: serde::Serialize>(
    stat: &mut Statement,
    data: &Option<T>,
    loc: usize,
) -> Result<(), Box<dyn std::error::Error>>
where
{
    match data {
        Some(val) => stat.bind(loc, enum_parse(val)?.as_str())?,
        None => stat.bind_null(loc)?,
    };

    Ok(())
}

#[cfg(feature = "cassandra")]
fn opt_string(
    stat: &mut Statement,
    data: &Option<String>,
    loc: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    match data {
        Some(val) => stat.bind(loc, val.as_str())?,
        None => stat.bind_null(loc)?,
    };

    Ok(())
}

#[cfg(feature = "cassandra")]
fn e_for_opt<T: Clone>(
    stat: &mut Statement,
    data: &Option<T>,
    loc: usize,
) -> Result<(), Box<dyn std::error::Error>>
where
    cassandra_cpp::Statement: BindRustType<T>,
{
    match data {
        Some(val) => stat.bind(loc, val.clone())?,
        None => stat.bind_null(loc)?,
    };

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub enum AttemptStatus {
    Started,
    AuthenticationFailed,
    RouterDeclined,
    AuthenticationPending,
    AuthenticationSuccessful,
    Authorized,
    AuthorizationFailed,
    Charged,
    Authorizing,
    CodInitiated,
    Voided,
    VoidInitiated,
    CaptureInitiated,
    CaptureFailed,
    VoidFailed,
    AutoRefunded,
    PartialCharged,
    PartialChargedAndChargeable,
    Unresolved,
    Pending,
    Failure,
    PaymentMethodAwaited,
    ConfirmationAwaited,
    DeviceDataCollectionPending,
}

#[derive(Serialize, Deserialize)]
pub enum Currency {
    AED,
    ALL,
    AMD,
    ANG,
    AOA,
    ARS,
    AUD,
    AWG,
    AZN,
    BAM,
    BBD,
    BDT,
    BGN,
    BHD,
    BIF,
    BMD,
    BND,
    BOB,
    BRL,
    BSD,
    BWP,
    BYN,
    BZD,
    CAD,
    CHF,
    CLP,
    CNY,
    COP,
    CRC,
    CUP,
    CVE,
    CZK,
    DJF,
    DKK,
    DOP,
    DZD,
    EGP,
    ETB,
    EUR,
    FJD,
    FKP,
    GBP,
    GEL,
    GHS,
    GIP,
    GMD,
    GNF,
    GTQ,
    GYD,
    HKD,
    HNL,
    HRK,
    HTG,
    HUF,
    IDR,
    ILS,
    INR,
    IQD,
    JMD,
    JOD,
    JPY,
    KES,
    KGS,
    KHR,
    KMF,
    KRW,
    KWD,
    KYD,
    KZT,
    LAK,
    LBP,
    LKR,
    LRD,
    LSL,
    LYD,
    MAD,
    MDL,
    MGA,
    MKD,
    MMK,
    MNT,
    MOP,
    MRU,
    MUR,
    MVR,
    MWK,
    MXN,
    MYR,
    MZN,
    NAD,
    NGN,
    NIO,
    NOK,
    NPR,
    NZD,
    OMR,
    PAB,
    PEN,
    PGK,
    PHP,
    PKR,
    PLN,
    PYG,
    QAR,
    RON,
    RSD,
    RUB,
    RWF,
    SAR,
    SBD,
    SCR,
    SEK,
    SGD,
    SHP,
    SLE,
    SLL,
    SOS,
    SRD,
    SSP,
    STN,
    SVC,
    SZL,
    THB,
    TND,
    TOP,
    TRY,
    TTD,
    TWD,
    TZS,
    UAH,
    UGX,
    USD,
    UYU,
    UZS,
    VES,
    VND,
    VUV,
    WST,
    XAF,
    XCD,
    XOF,
    XPF,
    YER,
    ZAR,
    ZMW,
}

#[derive(Serialize, Deserialize)]
pub enum PaymentMethod {
    Card,
    Token,
    PaymentProfile,
    Cash,
    Cheque,
    Interac,
    ApplePay,
    AndroidPay,
    #[serde(rename = "3d_secure")]
    ThreeDSecure,
    ProcessorToken,
}

#[derive(Serialize, Deserialize)]
pub enum CaptureMethod {
    /// Post the payment authorization, the capture will be executed on the full amount immediately
    Automatic,
    /// The capture will happen only if the merchant triggers a Capture API request
    Manual,
    /// The capture will happen only if the merchant triggers a Capture API request
    ManualMultiple,
    /// The capture can be scheduled to automatically get triggered at a specific date & time
    Scheduled,
}

#[derive(Serialize, Deserialize)]
pub enum AuthenticationType {
    /// If the card is enrolled for 3DS authentication, the 3DS based authentication will be activated. The liability of chargeback shift to the issuer
    ThreeDs,
    /// 3DS based authentication will not be activated. The liability of chargeback stays with the merchant.
    NoThreeDs,
}

#[derive(Serialize, Deserialize)]
pub enum PaymentExperience {
    /// The URL to which the customer needs to be redirected for completing the payment.
    RedirectToUrl,
    /// Contains the data for invoking the sdk client for completing the payment.
    InvokeSdkClient,
    /// The QR code data to be displayed to the customer.
    DisplayQrCode,
    /// Contains data to finish one click payment.
    OneClick,
    /// Redirect customer to link wallet
    LinkWallet,
    /// Contains the data for invoking the sdk client for completing the payment.
    InvokePaymentApp,
    /// Contains the data for displaying wait screen
    DisplayWaitScreen,
}

#[derive(Serialize, Deserialize)]
pub enum PaymentMethodType {
    Ach,
    Affirm,
    AfterpayClearpay,
    Alfamart,
    AliPay,
    AliPayHk,
    Alma,
    ApplePay,
    Atome,
    Bacs,
    BancontactCard,
    Becs,
    Benefit,
    Bizum,
    Blik,
    Boleto,
    BcaBankTransfer,
    BniVa,
    BriVa,
    CardRedirect,
    CimbVa,
    #[serde(rename = "classic")]
    ClassicReward,
    Credit,
    CryptoCurrency,
    Cashapp,
    Dana,
    DanamonVa,
    Debit,
    DuitNow,
    Efecty,
    Eps,
    Fps,
    Evoucher,
    Giropay,
    Givex,
    GooglePay,
    GoPay,
    Gcash,
    Ideal,
    Interac,
    Indomaret,
    Klarna,
    KakaoPay,
    LocalBankRedirect,
    MandiriVa,
    Knet,
    MbWay,
    MobilePay,
    Momo,
    MomoAtm,
    Multibanco,
    OnlineBankingThailand,
    OnlineBankingCzechRepublic,
    OnlineBankingFinland,
    OnlineBankingFpx,
    OnlineBankingPoland,
    OnlineBankingSlovakia,
    Oxxo,
    PagoEfectivo,
    PermataBankTransfer,
    OpenBankingUk,
    PayBright,
    Paypal,
    Pix,
    PaySafeCard,
    Przelewy24,
    PromptPay,
    Pse,
    RedCompra,
    RedPagos,
    SamsungPay,
    Sepa,
    Sofort,
    Swish,
    TouchNGo,
    Trustly,
    Twint,
    UpiCollect,
    UpiIntent,
    Vipps,
    VietQr,
    Venmo,
    Walley,
    WeChatPay,
    SevenEleven,
    Lawson,
    MiniStop,
    FamilyMart,
    Seicomart,
    PayEasy,
    LocalBankTransfer,
    Mifinity,
}

#[derive(Serialize, Deserialize)]
pub enum MandateDataType {
    SingleUse(MandateAmountData),
    MultiUse(Option<MandateAmountData>),
}

#[derive(Serialize, Deserialize)]
pub struct MandateAmountData {
    pub amount: i64,
    pub currency: Currency,
    pub start_date: Option<PrimitiveDateTime>,
    pub end_date: Option<PrimitiveDateTime>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct MandateDetails {
    pub update_mandate_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[charybdis_model(
    table_name = merchant_accounts,
    partition_keys = [merchant_id],
    clustering_keys = [],
    global_secondary_indexes = [],
    local_secondary_indexes = [],
    static_columns = []
)]
pub struct MerchantAccount {
    pub merchant_id: String,
    pub return_url: Option<String>,
    pub enable_payment_response_hash: bool,
    pub payment_response_hash_key: Option<String>,
    pub redirect_to_merchant_with_http_post: bool,
    pub merchant_name: Option<String>,
    pub merchant_details: Option<String>,
    pub webhook_details: Option<String>,
    pub sub_merchants_enabled: Option<bool>,
    pub parent_merchant_id: Option<String>,
    pub publishable_key: Option<String>,
    pub storage_scheme: String,
    pub locker_id: Option<String>,
    pub metadata: Option<String>,
    pub routing_algorithm: Option<String>,
    pub primary_business_details: String,
    pub intent_fulfillment_time: Option<i64>,
    pub created_at: Timestamp,
    pub modified_at: Timestamp,
    pub frm_routing_algorithm: Option<String>,
    pub payout_routing_algorithm: Option<String>,
    pub organization_id: String,
    pub is_recon_enabled: bool,
    pub default_profile: Option<String>,
    pub recon_status: String,
    pub payment_link_config: Option<String>,
    pub pm_collect_link_config: Option<String>,
}

impl MerchantAccount {
    pub fn new(merchant_id : String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { merchant_id,
            return_url: Some("www.google.co.in".into()),
            enable_payment_response_hash: false,
            payment_response_hash_key: Some("hash".into()),
            redirect_to_merchant_with_http_post: true,
            merchant_name: Some("kaps".into()),
            merchant_details: None,
            webhook_details: Some("callback_url".into()),
            sub_merchants_enabled: None,
            parent_merchant_id: None,
            publishable_key: Some("AAAAB3NzaC1yc2EAAAADAQABAAACAQC+gCy875BbJDjy/KDczr84xswL1edCE82IkOCBeYuOPbmhR251K9r7UFjlXioa5UnMpRals/pMSkz9MF7yUTzPDg+NjfmsZ8rgHqTKJ7z/yUWEyxd3TcUh0xkkYMCfrA+9rLqgolBiasAOApBDYTi0BsBlfAgNIaTgg7xTX7PHUzceAvujJel1Q6V+bABnFvlDu6kWUhXlrPafWRPUSQz2wEsO7vqrE9UfP+CtuXrJ+t6pMbkVDGc0+JWaPJjBXMjxljBfZHw7UVbHmPlYYTwOGD/IWOgisFfvnutR4JvDZA5elWqkXj+ZEsOw4QFXw71o+b2YWrRBa8l+AFn//zPQvLB753wQVuhzmsidToLss2DfGLdrYKVuCTX7a7OhxKDYRYeyZgeqWK8xVqiyayXgvuxcZV2g+mHi2WuUGGJ6Ycj+JZ9Vh67EnplDmJAKCFXCenS4ou9rMCHqD6i9UVgzakzxy/wd5Cj6R26uKqKo8rZDw9D6zKDzF45NbVh+obAFh/9MuzSCaaL5pXWPUI0kI7iZ8lU7rC8HAj5HhynLZd+rfQazVo+qcoQMxO+A9+fFubru41Aku6siQgv6oXNiGSOcb4bgEDlCBQ/uQgNCn9Vdq3f1yWqC1eAQtwoB4YTE2DZrY1TVZiN202JQNweIIOQUANyKRVV2ITZketmeZQ==".into()),
            storage_scheme: "cassandra".into(),
            locker_id: None,
            metadata: None,
            routing_algorithm: Some(serde_json::to_string(&get_large_value())?.into()),
            primary_business_details: serde_json::to_string(&get_large_value())?.into(),
            intent_fulfillment_time: None,
            created_at: chrono::DateTime::default(),
            modified_at: chrono::DateTime::default(),
            frm_routing_algorithm: Some(serde_json::to_string(&get_large_value())?.into()),
            payout_routing_algorithm: Some(serde_json::to_string(&get_large_value())?.into()),
            organization_id: "kaps".into(),
            is_recon_enabled: false,
            default_profile: None,
            recon_status: "NONE".into(),
            payment_link_config: Some("link_config".into()),
            pm_collect_link_config: None
        })
    }
}

#[derive(Serialize, Deserialize)]
#[charybdis_model(
    table_name = customers,
    partition_keys = [customer_id],
    clustering_keys = [],
    global_secondary_indexes = [],
    local_secondary_indexes = [],
    static_columns = []
)]
pub struct Customer {
    pub id: i32,
    pub customer_id: String,
    pub merchant_id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub phone_country_code: Option<String>,
    pub description: Option<String>,
    pub created_at: Timestamp,
    pub metadata: Option<String>,
    pub connector_customer: Option<String>,
    pub modified_at: Timestamp,
    pub address_id: Option<String>,
    pub default_payment_method_id: Option<String>,
    pub updated_by: Option<String>,
}

// impl Customer{
//     pub fn new(customer_id : String) -> Result<(), Box>
// }


#[derive(Serialize)]
pub struct PaymentAttemptResponse {
    pub pa: String
}

#[derive(Serialize)]
pub struct PaymentIntentResponse {
    pub pi : String
}