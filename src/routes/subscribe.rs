use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum_macros::debug_handler;

use chrono::Utc;
use uuid::Uuid;
use std::sync::Arc;
use sqlx::{Postgres, Transaction};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use crate::app;
use crate::models;
use crate::email_client::{ValidEmail, EmailClient};

#[debug_handler]
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, payload),
    fields(
        // could not find a way to get this automatically with tower_http like in the book.
        request_id = %Uuid::new_v4(), 
        subscriber_email = %payload.get_email(),
        subscriber_name= %payload.get_name()
    )
)]
pub async fn subscribe(
    State(state): State<Arc<app::AppState>>,
    Json(payload): Json<models::NewSubscriber>,
) -> StatusCode {
    tracing::info!("Processing request: {:?}", payload);

    let mut transaction = match state.pg_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };


    // Insert the email into the database
    let response_insertion = insert_subscriber(
        &mut transaction, 
        payload.get_email(), 
        payload.get_name()).await;
    
    let subscriber_id: Uuid;
    match response_insertion {
            Ok(id) => {
                tracing::info!("Subscriber nsertion succeeded.");
                subscriber_id = id;
            },
            Err(err) => {
                tracing::error!("Insertion failed with error: {:?}", err);
                return StatusCode::BAD_REQUEST
            }
        }
    
    // generate the token and store it in the db
    let token = generate_subscription_token();
    let response_store_token = store_token(&mut transaction, subscriber_id, &token).await;
    match response_store_token {
        Ok(_) => {
            tracing::info!("Token insertion succeeded.");
        },
        Err(err) => {
            tracing::error!("Insertion failed with error: {:?}", err);
            return StatusCode::BAD_REQUEST
        }
    }

    if transaction.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    // directly return the statuscode returned by send_confirmation_email.
    send_confirmation_email(&state.base_url, &state.email_client, payload.get_email(), &token).await

}

#[tracing::instrument(
    name = "Insert subscriber in the database",
    skip(transaction)
)]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>, 
    email: &str, 
    name: &str
) -> Result<Uuid, sqlx::Error> {

    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'pending_confirmation')
            "#,
        subscriber_id,
        email,
        name,
        Utc::now()
    )
    // complex way to expose the database connection in transaction
    // that implements the executor trait.
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
    
}

/// Generate a random 25-characters-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    // complex way to expose the database connection in transaction
    // that implements the executor trait.
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
    tracing::error!("Failed to execute query: {:?}", e);
    e
    })?;
    Ok(())
    }


async fn send_confirmation_email(base_url: &app::ApplicationBaseUrl, email_client: &EmailClient, email_address: &str, token: &str) -> StatusCode {
// Send a confirmation email:
    
    // The validation is superfluous, since the validity is also checked
    // during derialisation of the request, but I need a ValidEmail for the
    // email_client.
    // let email_address = ValidEmail::new(email_address)?;
    let email_address = match ValidEmail::new(email_address) {
        Ok(x) => x,
        Err(err) => {
            tracing::error!("Email is not valid: {:?}", err);
            return StatusCode::BAD_REQUEST
        }
    };

    let confirmation_link = format!("{}/confirm?token={}", base_url.0, token);

    let res_conf = email_client
        .send_email(
            &email_address,
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
                confirmation_link
            )
        )
        .await;

        match res_conf {
            Ok(_) => {
                tracing::info!("Confirmation email sent!.");
            },
            Err(err) => {
                tracing::error!("Confirmation email failed with error: {:?}", err);
                // tracing::debug!("email_client.url =  {:?}", state.email_client.base_url);
                return StatusCode::INTERNAL_SERVER_ERROR
            }
        };
    
        StatusCode::OK

}