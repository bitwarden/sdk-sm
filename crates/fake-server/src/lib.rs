use axum::{
    response::Json,
    routing::{get, post, put},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub mod routes;

pub fn create_app() -> Router {
    Router::new()
        .route("/identity/connect/token", post(routes::auth::token))
        // secrets
        .route("/api/secrets/{id}", get(routes::secrets::get_secret))
        .route(
            "/api/organizations/{org_id}/secrets",
            get(routes::secrets::list_secrets).post(routes::secrets::create_secret),
        )
        .route(
            "/api/secrets/get-by-ids",
            post(routes::secrets::get_secrets_by_ids),
        )
        .route("/api/secrets/{id}", put(routes::secrets::create_secret)) // we don't really have data to edit, so just treat it as create
        .route(
            "/api/organizations/{org_id}/secrets/sync",
            get(routes::secrets::sync_secrets),
        )
        .route("/api/secrets/delete", post(routes::secrets::delete_secrets))
        // projects
        .route("/api/projects/{id}", get(routes::projects::get_project))
        .route(
            "/api/organizations/{org_id}/projects",
            get(routes::projects::list_projects).post(routes::projects::create_project),
        )
        .route("/api/projects/{id}", put(routes::projects::create_project)) // we don't really have data to edit, so just treat it as create
        .route(
            "/api/projects/delete",
            post(routes::projects::delete_projects),
        )
        // misc
        .route("/help", get(routes::misc::help))
        .route("/health", get(routes::misc::health_check))
        .route("/echo", post(routes::misc::echo))
        .fallback(fallback)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn fallback(
    uri: axum::http::Uri,
    Json(body): Json<serde_json::Value>,
) -> (axum::http::StatusCode, String) {
    println!("Endpoint was hit but not implemented: {}", uri);
    println!("Endpoint body: {}", body);
    (axum::http::StatusCode::NOT_FOUND, "No route".to_string())
}
