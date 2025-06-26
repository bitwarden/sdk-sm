use axum::{
    extract::Path,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

pub mod routes;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Option<String>,
    pub key: String,
    pub value: Option<String>,
    pub note: Option<String>,
    pub creation_date: String,
    pub revision_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub creation_date: String,
    pub revision_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSecretRequest {
    pub key: String,
    pub value: String,
    pub note: String,
    pub project_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetByIdsBody {
    ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GetByIdsResponse {
    data: Vec<Secret>,
}

#[derive(Serialize, Deserialize)]
struct SecretResponse {
    secrets: Vec<Secret>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectsResponse {
    object: String,
    data: Vec<Project>,
}

pub fn create_app() -> Router {
    Router::new()
        .route("/identity/connect/token", post(routes::auth::token))
        // secrets
        .route("/api/secrets/{id}", get(get_secret))
        .route(
            "/api/organizations/{org_id}/secrets",
            get(list_secrets).post(create_secret),
        )
        .route("/api/secrets/get-by-ids", post(get_secrets_by_ids))
        .route("/api/secrets/{id}", put(create_secret)) // we don't really have data to edit, so just treat it as create
        .route("/api/secrets/delete", post(delete_secrets))
        // projects
        .route("/api/projects/{id}", get(get_project))
        .route(
            "/api/organizations/{org_id}/projects",
            get(list_projects).post(create_project),
        )
        .route("/api/projects/{id}", put(create_project)) // we don't really have data to edit, so just treat it as create
        .route("/api/projects/delete", post(delete_projects))
        // misc
        .route("/help", get(help))
        .route("/health", get(health_check))
        .route("/echo", post(echo))
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

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn echo(Json(payload): Json<Value>) -> Json<Value> {
    info!("Echo request: {:?}", payload);
    Json(payload)
}

async fn help() -> Json<Value> {
    // TODO: put something nice here
    info!("Help endpoint was hit, returning fake API documentation.");

    Json(json!({
        "message": "This is a fake server for testing purposes.",
        "endpoints": [
            {
                "method": "POST",
                "path": "/identity/connect/token",
                "description": "Get an identity token."
            },
            {
                "method": "GET",
                "path": "/api/secrets/:id",
                "description": "Get a secret by ID."
            },
            {
                "method": "GET",
                "path": "/api/organizations/:org_id/secrets",
                "description": "List all secrets for an organization."
            },
            {
                "method": "POST",
                "path": "/api/organizations/:org_id/secrets",
                "description": "Create a new secret for an organization."
            },
            {
                "method": "POST",
                "path": "/api/secrets/get-by-ids",
                "description": "Get secrets by their IDs."
            },
            {
                "method": "PUT",
                "path": "/api/secrets/:id",
                "description": "Update a secret by ID."
            },
            {
                "method": "POST",
                "path": "/api/secrets/delete",
                "description": "Delete secrets by their IDs."
            },
            {
                "method": "GET",
                "path": "/api/projects/:id",
                "description": "Get a project by ID."
            },
            {
                "method": "GET",
                "path": "/api/organizations/:org_id/projects",
                "description": "List all projects for an organization."
            },
            {
                "method": "POST",
                "path": "/api/organizations/:org_id/projects",
                "description": "Create a new project for an organization."
            },
            {
                "method": "PUT",
                "path": "/api/projects/:id",
                "description": "Update a project by ID."
            },
            {
                "method": "POST",
                "path": "/api/projects/delete",
                "description": "Delete projects by their IDs."
            }
        ]
    }))
}

async fn list_secrets(Path(org_id): Path<String>) -> Json<SecretResponse> {
    info!("Listing secrets for organization: {}", org_id);

    let secrets = vec![
        Secret {
            id: uuid::Uuid::new_v4(),
            organization_id: Uuid::parse_str(&org_id).unwrap_or_else(|_| Uuid::new_v4()),
            project_id: Some(uuid::Uuid::new_v4().to_string()),
            key: "2.pMS6/icTQABtulw52pq2lg==|XXbxKxDTh+mWiN1HjH2N1w==|Q6PkuT+KX/axrgN9ubD5Ajk2YNwxQkgs3WJM0S0wtG8=".to_string(),
            value: None,
            note: None,
            creation_date: chrono::Utc::now().to_rfc3339(),
            revision_date: chrono::Utc::now().to_rfc3339(),
        },
    ];

    Json(SecretResponse { secrets: secrets })
}

async fn create_secret(
    Path(org_id): Path<String>,
    Json(payload): Json<CreateSecretRequest>,
) -> Json<Secret> {
    info!("Creating secret for organization {}: {:?}", org_id, payload);

    let secret = Secret {
        id: Uuid::new_v4(),
        organization_id: Uuid::parse_str(&org_id).unwrap_or_else(|_| Uuid::new_v4()),
        project_id: payload.project_ids.and_then(|ids| ids.first().cloned()),
        key: payload.key,
        value: Some(payload.value),
        note: Some(payload.note),
        creation_date: chrono::Utc::now().to_rfc3339(),
        revision_date: chrono::Utc::now().to_rfc3339(),
    };

    Json(secret)
}

async fn get_secret(Path(id): Path<Uuid>) -> Json<Secret> {
    info!("Getting secret with id: {}", id);

    let secret = Secret {
        id: id.clone(),
        organization_id: Uuid::new_v4(),
        project_id: Some(uuid::Uuid::new_v4().to_string()),
        key: "2.WYqmVCB2wZc08tkzNOCmTw==|FAsVol/nJnnDk3/mp7z6QQ==|uPJOCC8iAbMzz4t60c35iZm8KzWKMn0ueCVJZlfmTdY=".to_string(),
        value: Some("2.IYOGfBMSOI5qOxfYGHd6Rg==|0PBFivy/Qtp4lg4vv1+yPn/sDeRsNWmRnUYgwmAgPzUqZA9ZojvuggVSp/isPPc2mYO5UQfb/co/81fDhQqopHrwat0l8SRB+sv/uEuomDdMkjaYl+jqblXebIDN42ZCy1wbERZgFmCMm3k1OIj1z5WHdRFGTWDLFlP316SgkAKOwaZF0eNmcQ90Py5Mrq9rKeVozsPWIL3aAXNchID6kJnqxbx717BxKQ9Vj/dMAaBlQoGrl/cYA6hoUBq7wOSMWkZ8PAorLhc3OSDwGT/iamlAfePbkbjVqlTK2WrQ5ZHIo5Qzwpd/cvn6a0rSW5cPQ6DLrrOBdgDU3ELJ3eB+vZ/IWl9jXsCQ3re6Pv4pOToAMYDYEkC7DlwbSiCWLegqbexwPNLRLa2hM9n+V8nVPgNic+LyakfsLqx1ReDFY0A7qRs7pE/EabYyj1O44HwZT3sSFKGYPlTBmQh6S21T7eGJ4+OV+dhnFSpjiJ7IjOhfAzwq8cUiAeIEvKECBD++C+TsGwNAYK57F8Dd2gEwSaDhkiEPssa/c9ZBQnarNWzmZN1gj4udXRmsXqAY6GcrZiLhBIpW2Yap8VVdgbQ9vwN77NzLfFW/FsdlAPB22dvjR1SzszgweG2QstGi9PcKY0Mp1zSvswWdGjdBpbfuExXBD62Fp+DWOmFzWPo2MyqSQLaegvO4G+v8DRlf7VHA34Yvcbzv9Jtq4+H+Z7SkglRcQvKrn9uv7qOlZPvGJs1Ri86BAopXIGsj/5XfQTdtQdhs4c0vviMSrNWtNvIgfg==|Z6BNqVlCknATGieykii0vF9xKu+JT3u2WqtbDhSYvkY=".to_string()),
        note: Some("2.S57kOfi1kIHjToxwR6sEuQ==|lTop/7iWWUveCGWXrHbHwg==|YrtUfrlRRN+ff8Re7txi2dTT9Ul0cwmiFWDgVpdWGlc=".to_string()),
        creation_date: chrono::Utc::now().to_rfc3339(),
        revision_date: chrono::Utc::now().to_rfc3339(),
    };

    Json(secret)
}

async fn get_secrets_by_ids(Json(payload): Json<GetByIdsBody>) -> Json<GetByIdsResponse> {
    let id1 = payload.ids.get(0).cloned().unwrap_or(Uuid::new_v4());
    let id2 = payload.ids.get(1).cloned().unwrap_or(Uuid::new_v4());

    info!("Getting secrets with ids: {}, {}", id1, id2);

    let mut secrets: Vec<Secret> = payload.ids.iter().map(|_| Secret {
        // FERRIS, the crab
        id: id1,
        organization_id: Uuid::parse_str("ec2c1d46-6a4b-4751-a310-af9601317f2d").unwrap_or_else(|_| Uuid::new_v4()),
        project_id: Some(uuid::Uuid::new_v4().to_string()),
        key: "2.N2aCz0PU6Ga9YfJlvisnwQ==|3M8dF2PFub9FP/SbgdenSQ==|KbaUQSb5IjVwhSbXDbCJbKXGBaCHEKDArrhvQDr9/QM=".to_string(), //
        value: Some("2./mtIg5EsCJbesKfmbWAx+A==|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".to_string()),
        note: Some("2.pMS6/icTQABtulw52pq2lg==|XXbxKxDTh+mWiN1HjH2N1w==|Q6PkuT+KX/axrgN9ubD5Ajk2YNwxQkgs3WJM0S0wtG8=".to_string()),
        creation_date: chrono::Utc::now().to_rfc3339(),
        revision_date: chrono::Utc::now().to_rfc3339(),
    }).collect::<Vec<_>>();

    secrets.push(Secret {
        // TUX, the penguin
        id: id2,
        organization_id: Uuid::parse_str("ec2c1d46-6a4b-4751-a310-af9601317f2d").unwrap_or_else(|_| Uuid::new_v4()),
        project_id: Some(uuid::Uuid::new_v4().to_string()),
        key: "2.OldQj0RJKww0WN7RSxI1wQ==|TpxAbmdx6zIVo37YJ5n1aQ==|06Imyx7jqaZ5J5amrBboCVPwvPoDKB8REJdToQwp3dA=".to_string(),
        value: Some("2.oEDp566lC9VYHn6XmusxfA==|Gj23w5q2NZ4z9PNne1d0ug==|y7K5TgMJFI0T0yFwLXzAMf9OBANNT567hLQ+z7G2rac=".to_string()),
        note: Some("2.owktgGRm4r+ho4WY4U9zvA==|6Up5NQHyZ65SL3vbNI1GhQ==|vdvWvPpoB/J3aWXKBiruqOr1SK/ndkCCTjHf2vphhu4=".to_string()),
        creation_date: chrono::Utc::now().to_rfc3339(),
        revision_date: chrono::Utc::now().to_rfc3339(),
    });
    Json(GetByIdsResponse { data: secrets })
}

async fn delete_secrets(Json(ids): Json<Vec<Uuid>>) -> Json<Value> {
    info!("Deleting secrets with ids: {:?}", ids);

    for id in &ids {
        info!("Deleted secret with id: {}", id);
    }

    Json(json!({
        "message": "Secrets deleted successfully",
        "deleted_ids": ids
    }))
}

async fn list_projects(Path(org_id): Path<String>) -> Json<ProjectsResponse> {
    info!("Listing projects for organization: {}", org_id);

    let projects =
    vec![
        Project {
            id: uuid::Uuid::new_v4().to_string(),
            organization_id: org_id.clone(),
            name: "2.DmcNJqtzi+nPWY9gJR4nMw==|IEkn2x+C0YLmnQ/qm0EfOcMGcRZDkexFkDW9BPw3wRQ=|TxxTeBKqL0QYLT+89F0KfI81BbBryXnNNAjU9DGKuuY=".to_string(),
            creation_date: chrono::Utc::now().to_rfc3339(),
            revision_date: chrono::Utc::now().to_rfc3339(),
        },
        Project {
            id: uuid::Uuid::new_v4().to_string(),
            organization_id: org_id,
            name: "2.4hWxQC9O5KpHcyCyI/xBsQ==|RLkyV/QbEMpxPnO91E/jPURCvsDIjI1ZIh6eMvGIuEg=|FXnie8Z9OtaBElnzF0v4Iut0fmy7IAI2IedEKJKuSp0=".to_string(),
            creation_date: chrono::Utc::now().to_rfc3339(),
            revision_date: chrono::Utc::now().to_rfc3339(),
        }
    ];

    Json(ProjectsResponse {
        object: "list".to_string(),
        data: projects,
    })
}

async fn create_project(
    Path(org_id): Path<String>,
    Json(payload): Json<CreateProjectRequest>,
) -> Json<Project> {
    info!(
        "Creating project for organization {}: {:?}",
        org_id, payload
    );

    let project = Project {
        id: uuid::Uuid::new_v4().to_string(),
        organization_id: org_id,
        name: payload.name,
        creation_date: chrono::Utc::now().to_rfc3339(),
        revision_date: chrono::Utc::now().to_rfc3339(),
    };

    Json(project)
}

async fn get_project(Path(id): Path<String>) -> Json<Project> {
    info!("Getting project with id: {}", id);

    let project = Project {
        id: Uuid::new_v4().to_string(),
        organization_id: "ec2c1d46-6a4b-4751-a310-af9601317f2d".to_string(),
        name: "2.DmcNJqtzi+nPWY9gJR4nMw==|IEkn2x+C0YLmnQ/qm0EfOcMGcRZDkexFkDW9BPw3wRQ=|TxxTeBKqL0QYLT+89F0KfI81BbBryXnNNAjU9DGKuuY=".to_string(),
        creation_date: chrono::Utc::now().to_rfc3339(),
        revision_date: chrono::Utc::now().to_rfc3339(),
    };

    Json(project)
}

async fn delete_projects(Json(ids): Json<Vec<String>>) -> Json<Value> {
    info!("Deleting projects with ids: {:?}", ids);

    for id in &ids {
        info!("Deleted project with id: {}", id);
    }

    Json(json!({
        "message": "Projects deleted successfully",
        "deleted_ids": ids
    }))
}
