use std::collections::HashMap;
use std::str::FromStr;
use stripe::{
    CreateCustomer, CreateEphemeralKey, Customer, EphemeralKey, PaymentIntent, StripeError,
};
use stripe::{CreatePaymentIntent, CustomerId};

pub use stripe::CreatePaymentIntentShipping;
pub use stripe::CreatePaymentIntentShippingAddress;

use my_macros::make_error;
pub use stripe::Client;

make_error!(StripePaymentError);

pub struct CreatePaymentIntentDto {
    pub amount: i64,
    pub stripe_customer_id: String,
    pub delivery_address: Option<CreatePaymentIntentShipping>,
    pub currency: String,
}

pub struct PaymentIntentDto {
    pub id: String,
    pub ephemeral_secret: String,
    pub client_secret: String,
    pub stripe_customer_id: String,
}

pub struct CreateCustomerDto {
    pub id: String,
}

pub struct CustomerDto {
    pub id: String,
}

pub async fn get_customer(
    stripe_client: &stripe::Client,
    account_id: String,
) -> Result<CustomerDto, StripeError> {
    let url = format!(
        "/v1/customers/search?query=metadata%5B%account_id%27%5D%3A%27{}%27",
        account_id
    );
    stripe_client
        .get::<Customer>(url.as_str())
        .await
        .map(|x| CustomerDto {
            id: x.id.to_string(),
        })
}

pub async fn create_customer(
    stripe_client: &Client,
    dto: &CreateCustomerDto,
) -> Result<CustomerDto, StripePaymentError> {
    let mut meta = HashMap::<String, String>::new();
    meta.insert("id".to_string(), dto.id.clone());
    Customer::create(
        &stripe_client,
        CreateCustomer {
            address: None,
            balance: None,
            cash_balance: None,
            coupon: None,
            description: None,
            email: None,
            expand: &[],
            invoice_prefix: None,
            invoice_settings: None,
            metadata: Some(meta),
            name: None,
            next_invoice_sequence: None,
            payment_method: None,
            phone: None,
            preferred_locales: None,
            promotion_code: None,
            shipping: None,
            source: None,
            tax: None,
            tax_exempt: None,
            tax_id_data: None,
            test_clock: None,
        },
    )
    .await
    .map(|x| CustomerDto {
        id: x.id.to_string(),
    })
    .map_err(StripePaymentError::from_general)
}

pub async fn create_payment_sheet(
    stripe_client: &Client,
    dto: &CreatePaymentIntentDto,
) -> Result<PaymentIntentDto, StripePaymentError> {
    let stripe_customer_id = CustomerId::from_str(dto.stripe_customer_id.as_str())
        .map_err(|x| StripePaymentError::from_general(x.to_string()))?;
    let ephemeral_key = EphemeralKey::create(
        &stripe_client,
        CreateEphemeralKey {
            customer: Some(stripe_customer_id.clone()),
            expand: &[],
            issuing_card: None,
        },
    )
    .await
    .map_err(StripePaymentError::from_general)?;
    let ephemeral_key_secret = ephemeral_key
        .secret
        .ok_or(StripePaymentError::from_general(
            "no ephemeral_key_secret".to_string(),
        ))?;

    let payment_intent = PaymentIntent::create(
        &stripe_client,
        CreatePaymentIntent {
            amount: dto.amount,
            application_fee_amount: None,
            automatic_payment_methods: None,
            capture_method: None,
            confirm: None,
            confirmation_method: None,
            currency: stripe::Currency::from_str(dto.currency.to_lowercase().as_str())
                .map_err(|x| StripePaymentError::from_general(x.to_string()))?,
            customer: Some(stripe_customer_id),
            description: None,
            error_on_requires_action: None,
            expand: &[],
            mandate: None,
            mandate_data: None,
            metadata: None,
            off_session: None,
            on_behalf_of: None,
            payment_method: None,
            payment_method_data: None,
            payment_method_options: None,
            payment_method_types: Some(vec!["card".to_string()]),
            receipt_email: None,
            return_url: None,
            setup_future_usage: None,
            shipping: dto.delivery_address.clone(),
            statement_descriptor: None,
            statement_descriptor_suffix: None,
            transfer_data: None,
            transfer_group: None,
            use_stripe_sdk: None,
        },
    )
    .await
    .map_err(StripePaymentError::from_general)?;

    let payment_client_secret =
        payment_intent
            .client_secret
            .ok_or(StripePaymentError::from_general(
                "no payment_client_secret".to_string(),
            ))?;

    Ok(PaymentIntentDto {
        id: payment_intent.id.to_string(),
        ephemeral_secret: ephemeral_key_secret,
        client_secret: payment_client_secret,
        stripe_customer_id: dto.stripe_customer_id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use stripe::{CreatePaymentIntent, PaymentIntent};

    #[test]
    fn hello() {
        let stripe_client = stripe::Client::new("");

        PaymentIntent::create(
            &stripe_client,
            CreatePaymentIntent {
                amount: 0,
                application_fee_amount: None,
                automatic_payment_methods: None,
                capture_method: None,
                confirm: None,
                confirmation_method: None,
                currency: stripe::Currency::AED,
                customer: None,
                description: None,
                error_on_requires_action: None,
                expand: &[],
                mandate: None,
                mandate_data: None,
                metadata: None,
                off_session: None,
                on_behalf_of: None,
                payment_method: None,
                payment_method_data: None,
                payment_method_options: None,
                payment_method_types: None,
                receipt_email: None,
                return_url: None,
                setup_future_usage: None,
                shipping: None,
                statement_descriptor: None,
                statement_descriptor_suffix: None,
                transfer_data: None,
                transfer_group: None,
                use_stripe_sdk: None,
            },
        );
    }
}
