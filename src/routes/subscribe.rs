use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum_macros::debug_handler;
use chrono::Utc;
use uuid::Uuid;

use std::sync::Arc;

use crate::app;

use crate::error::Error;
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

    // Insert the email into the database
    let res = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at, status)
            VALUES ($1, $2, $3, $4, 'confirmed')
            "#,
        Uuid::new_v4(),
        payload.get_email(),
        payload.get_name(),
        Utc::now()
    )
    .execute(&state.pg_pool)
    .await;

    match res {
        Ok(_) => {
            tracing::info!("Insertion succeeded.");
        },
        Err(err) => {
            tracing::error!("Insertion failed with error: {:?}", err);
            return StatusCode::BAD_REQUEST
        }
    };
    
    let status = send_confirmation_email(&state.email_client, payload.get_email()).await;
    status
   

}


async fn send_confirmation_email(email_client: &EmailClient, email_address: &str) -> StatusCode {
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

    let confirmation_link = "https://my-api.com/subscriptions/confirm";

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