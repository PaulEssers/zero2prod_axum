use axum::routing::get;
use axum::Router;

use crate::routes::utils::health_check;

pub async fn spawn_app() -> Result<Router, String> {
    // build our application with some routes
    let app = Router::new().route("/health_check", get(health_check));

    Ok(app)
}
