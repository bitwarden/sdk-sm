use axum::{
    Router,
    routing::{get, post},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub mod routes;

pub fn create_app() -> Router {
    Router::new()
        .route("/identity/connect/token", post(routes::auth::token))
        // secrets
        .route(
            "/api/secrets/{id}",
            get(routes::secrets::get_secret).put(routes::secrets::create_secret),
        )
        .route(
            "/api/organizations/{org_id}/secrets",
            get(routes::secrets::list_secrets).post(routes::secrets::create_secret),
        )
        .route(
            "/api/secrets/get-by-ids",
            post(routes::secrets::get_secrets_by_ids),
        )
        .route(
            "/api/organizations/{org_id}/secrets/sync",
            get(routes::secrets::sync_secrets),
        )
        .route("/api/secrets/delete", post(routes::secrets::delete_secrets))
        // projects
        .route(
            "/api/projects/{id}",
            get(routes::projects::get_project).put(routes::projects::create_project),
        )
        .route(
            "/api/organizations/{org_id}/projects",
            get(routes::projects::list_projects).post(routes::projects::create_project),
        )
        .route(
            "/api/projects/delete",
            post(routes::projects::delete_projects),
        )
        // access policies - project
        .route(
            "/api/projects/{id}/access-policies/people",
            get(routes::access_policies::get_project_people_access_policies)
                .put(routes::access_policies::put_project_people_access_policies),
        )
        .route(
            "/api/projects/{id}/access-policies/service-accounts",
            get(routes::access_policies::get_project_service_accounts_access_policies)
                .put(routes::access_policies::put_project_service_accounts_access_policies),
        )
        // access policies - secrets
        .route(
            "/api/secrets/{id}/access-policies",
            get(routes::access_policies::get_secret_access_policies)
                .put(routes::access_policies::put_secret_access_policies),
        )
        // access policies - service accounts
        .route(
            "/api/service-accounts/{id}/granted-policies",
            get(routes::access_policies::get_service_account_granted_policies)
                .put(routes::access_policies::put_service_account_granted_policies),
        )
        // access policies - potential grantees
        .route(
            "/api/organizations/{org_id}/access-policies/people/potential-grantees",
            get(routes::access_policies::get_people_potential_grantees),
        )
        .route(
            "/api/organizations/{org_id}/access-policies/projects/potential-grantees",
            get(routes::access_policies::get_project_potential_grantees),
        )
        .route(
            "/api/organizations/{org_id}/access-policies/service-accounts/potential-grantees",
            get(routes::access_policies::get_service_accounts_potential_grantees),
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
    body: Option<axum::extract::Json<serde_json::Value>>,
) -> (axum::http::StatusCode, String) {
    println!("Endpoint was hit but not implemented: {}", uri);
    if let Some(axum::extract::Json(body)) = body {
        println!("Endpoint body: {}", body);
    }
    (axum::http::StatusCode::NOT_FOUND, "No route".to_string())
}
