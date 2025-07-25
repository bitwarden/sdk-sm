// This module organizes routes into separate modules by feature

use axum::{
    extract::{Path, Query},
    response::Json,
    Form,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;
use uuid::Uuid;

pub mod auth {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct IdentityTokenPayloadResponse {
        pub access_token: String,
        pub expires_in: u64,
        pub refresh_token: Option<String>,
        pub token_type: String,
        pub scope: String,
        pub encrypted_payload: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct TokenRequest {
        pub grant_type: String,
        pub client_id: Option<String>,
        pub client_secret: Option<String>,
        pub username: Option<String>,
        pub password: Option<String>,
        pub scope: Option<String>,
    }
    pub async fn token(Form(payload): Form<TokenRequest>) -> Json<IdentityTokenPayloadResponse> {
        info!("Token request: {:?}", payload);

        // Hard-coded token response for testing
        let token_response = IdentityTokenPayloadResponse {
            access_token: "eyJhbGciOiJSUzI1NiIsImtpZCI6IjMwMURENkE1MEU4NEUxRDA5MUM4MUQzQjAwQkY5MDEwQzg1REJEOUFSUzI1NiIsInR5cCI6\
                    ImF0K2p3dCIsIng1dCI6Ik1CM1dwUTZFNGRDUnlCMDdBTC1RRU1oZHZabyJ9.eyJuYmYiOjE2NzUxMDM3ODEsImV4cCI6MTY3NTEwNzM4MSwiaXNzIjo\
                    iaHR0cDovL2xvY2FsaG9zdCIsImNsaWVudF9pZCI6ImVjMmMxZDQ2LTZhNGItNDc1MS1hMzEwLWFmOTYwMTMxN2YyZCIsInN1YiI6ImQzNDgwNGNhLTR\
                    mNmMtNDM5Mi04NmI3LWFmOTYwMTMxNzVkMCIsIm9yZ2FuaXphdGlvbiI6ImY0ZTQ0YTdmLTExOTAtNDMyYS05ZDRhLWFmOTYwMTMxMjdjYiIsImp0aSI\
                    6IjU3QUU0NzQ0MzIwNzk1RThGQkQ4MUIxNDA2RDQyNTQyIiwiaWF0IjoxNjc1MTAzNzgxLCJzY29wZSI6WyJhcGkuc2VjcmV0cyJdfQ.GRKYzqgJZHEE\
                    ZHsJkhVZH8zjYhY3hUvM4rhdV3FU10WlCteZdKHrPIadCUh-Oz9DxIAA2HfALLhj1chL4JgwPmZgPcVS2G8gk8XeBmZXowpVWJ11TXS1gYrM9syXbv9j\
                    0JUCdpeshH7e56WnlpVynyUwIum9hmYGZ_XJUfmGtlKLuNjYnawTwLEeR005uEjxq3qI1kti-WFnw8ciL4a6HLNulgiFw1dAvs4c7J0souShMfrnFO3g\
                    SOHff5kKD3hBB9ynDBnJQSFYJ7dFWHIjhqs0Vj-9h0yXXCcHvu7dVGpaiNjNPxbh6YeXnY6UWcmHLDtFYsG2BWcNvVD4-VgGxXt3cMhrn7l3fSYuo32Z\
                    Yk4Wop73XuxqF2fmfmBdZqGI1BafhENCcZw_bpPSfK2uHipfztrgYnrzwvzedz0rjFKbhDyrjzuRauX5dqVJ4ntPeT9g_I5n71gLxiP7eClyAx5RxdF6\
                    He87NwC8i-hLBhugIvLTiDj-Sk9HvMth6zaD0ebxd56wDjq8-CMG_WcgusDqNzKFHqWNDHBXt8MLeTgZAR2rQMIMFZqFgsJlRflbig8YewmNUA9wAU74\
                    TfxLY1foO7Xpg49vceB7C-PlvGi1VtX6F2i0tc_67lA5kWXnnKBPBUyspoIrmAUCwfms5nTTqA9xXAojMhRHAos_OdM".to_string(),
            expires_in: 3600, // 1 hour
            refresh_token: Some("fake_refresh_token_67890".to_string()),
            token_type: "Bearer".to_string(),
            scope: "api.secrets".to_string(),
            encrypted_payload: "2.E9fE8+M/VWMfhhim1KlCbQ==|eLsHR484S/tJbIkM6spnG/HP65tj9A6Tba7kAAvUp+rYuQmGLixiOCfMsqt5OvBctDfvvr/Aes\
                    Bu7cZimPLyOEhqEAjn52jF0eaI38XZfeOG2VJl0LOf60Wkfh3ryAMvfvLj3G4ZCNYU8sNgoC2+IQ==|lNApuCQ4Pyakfo/wwuuajWNaEX/2MW8/3rjXB/V7n+k=".to_string(),
        };

        Json(token_response)
    }
}

pub mod secrets {
    use bitwarden_sm::secrets::SecretResponse;
    use chrono::DateTime;

    use super::*;
    #[derive(Debug, Serialize)]
    pub struct SecretsSyncResponse {
        #[serde(rename = "hasChanges")]
        pub has_changes: bool,
        pub secrets: Option<serde_json::Value>,
    }

    #[derive(Debug, Deserialize)]
    pub struct SyncQueryParams {
        #[serde(rename = "lastSyncedDate")]
        pub last_synced_date: Option<DateTime<chrono::Utc>>,
    }

    // bitwarden_sm::secrets::SecretCreateRequest has deny_unknown_fields
    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateSecretRequest {
        pub key: String,
        pub value: String,
        pub note: String,
        pub project_ids: Option<Vec<Uuid>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct GetByIdsBody {
        ids: Vec<Uuid>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct GetByIdsResponse {
        data: Vec<SecretResponse>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct SecretListResponse {
        secrets: Vec<SecretResponse>,
    }

    pub async fn list_secrets(Path(org_id): Path<String>) -> Json<SecretListResponse> {
        info!("Listing secrets for organization: {}", org_id);

        let secrets = vec![
            SecretResponse {
                id: uuid::Uuid::new_v4(),
                organization_id: Uuid::parse_str(&org_id).unwrap_or_else(|_| Uuid::new_v4()),
                project_id: Some(uuid::Uuid::new_v4()),
                key: "2.pMS6/icTQABtulw52pq2lg==|XXbxKxDTh+mWiN1HjH2N1w==|Q6PkuT+KX/axrgN9ubD5Ajk2YNwxQkgs3WJM0S0wtG8=".to_string(),
                value: "2.i189LsTzTnYi00heXfe5fw==|xRFAsQGm1qbpasRBw0i9cg==|oNaTecIpkIFITxcI/pNHF8FOyuBMGgHIyS4PoLiJ34Y=".to_string(),
                note: "2.i189LsTzTnYi00heXfe5fw==|xRFAsQGm1qbpasRBw0i9cg==|oNaTecIpkIFITxcI/pNHF8FOyuBMGgHIyS4PoLiJ34Y=".to_string(),
                creation_date: chrono::Utc::now(),
                revision_date: chrono::Utc::now(),
            },
        ];

        Json(SecretListResponse { secrets })
    }

    pub async fn create_secret(
        Path(org_id): Path<String>,
        Json(payload): Json<CreateSecretRequest>,
    ) -> Json<SecretResponse> {
        info!("Creating secret for organization {}: {:?}", org_id, payload);

        let secret = SecretResponse {
            id: Uuid::new_v4(),
            organization_id: Uuid::parse_str(&org_id).unwrap_or_else(|_| Uuid::new_v4()),
            project_id: payload.project_ids.and_then(|ids| ids.first().cloned()),
            key: payload.key,
            value: payload.value,
            note: payload.note,
            creation_date: chrono::Utc::now(),
            revision_date: chrono::Utc::now(),
        };

        Json(secret)
    }

    pub async fn get_secret(Path(id): Path<Uuid>) -> Json<SecretResponse> {
        info!("Getting secret with id: {}", id);

        let secret = SecretResponse {
            id: id.clone(),
            organization_id: Uuid::new_v4(),
            project_id: Some(uuid::Uuid::new_v4()),
            key: "2.WYqmVCB2wZc08tkzNOCmTw==|FAsVol/nJnnDk3/mp7z6QQ==|uPJOCC8iAbMzz4t60c35iZm8KzWKMn0ueCVJZlfmTdY=".to_string(),
            value: "2.IYOGfBMSOI5qOxfYGHd6Rg==|0PBFivy/Qtp4lg4vv1+yPn/sDeRsNWmRnUYgwmAgPzUqZA9ZojvuggVSp/isPPc2mYO5UQfb/co/81fDhQqopHrwat0l8SRB+sv/uEuomDdMkjaYl+jqblXebIDN42ZCy1wbERZgFmCMm3k1OIj1z5WHdRFGTWDLFlP316SgkAKOwaZF0eNmcQ90Py5Mrq9rKeVozsPWIL3aAXNchID6kJnqxbx717BxKQ9Vj/dMAaBlQoGrl/cYA6hoUBq7wOSMWkZ8PAorLhc3OSDwGT/iamlAfePbkbjVqlTK2WrQ5ZHIo5Qzwpd/cvn6a0rSW5cPQ6DLrrOBdgDU3ELJ3eB+vZ/IWl9jXsCQ3re6Pv4pOToAMYDYEkC7DlwbSiCWLegqbexwPNLRLa2hM9n+V8nVPgNic+LyakfsLqx1ReDFY0A7qRs7pE/EabYyj1O44HwZT3sSFKGYPlTBmQh6S21T7eGJ4+OV+dhnFSpjiJ7IjOhfAzwq8cUiAeIEvKECBD++C+TsGwNAYK57F8Dd2gEwSaDhkiEPssa/c9ZBQnarNWzmZN1gj4udXRmsXqAY6GcrZiLhBIpW2Yap8VVdgbQ9vwN77NzLfFW/FsdlAPB22dvjR1SzszgweG2QstGi9PcKY0Mp1zSvswWdGjdBpbfuExXBD62Fp+DWOmFzWPo2MyqSQLaegvO4G+v8DRlf7VHA34Yvcbzv9Jtq4+H+Z7SkglRcQvKrn9uv7qOlZPvGJs1Ri86BAopXIGsj/5XfQTdtQdhs4c0vviMSrNWtNvIgfg==|Z6BNqVlCknATGieykii0vF9xKu+JT3u2WqtbDhSYvkY=".to_string(),
            note: "2.S57kOfi1kIHjToxwR6sEuQ==|lTop/7iWWUveCGWXrHbHwg==|YrtUfrlRRN+ff8Re7txi2dTT9Ul0cwmiFWDgVpdWGlc=".to_string(),
            creation_date: chrono::Utc::now(),
            revision_date: chrono::Utc::now(),
        };

        Json(secret)
    }

    pub async fn get_secrets_by_ids(Json(payload): Json<GetByIdsBody>) -> Json<GetByIdsResponse> {
        let id1 = payload.ids.get(0).cloned().unwrap_or(Uuid::new_v4());
        let id2 = payload.ids.get(1).cloned().unwrap_or(Uuid::new_v4());

        info!("Getting secrets with ids: {}, {}", id1, id2);

        let mut secrets: Vec<SecretResponse> = payload.ids.iter().map(|_| SecretResponse {
            // FERRIS, the crab
            id: id1,
            organization_id: Uuid::parse_str("ec2c1d46-6a4b-4751-a310-af9601317f2d").unwrap_or_else(|_| Uuid::new_v4()),
            project_id: Some(uuid::Uuid::new_v4()),
            key: "2.N2aCz0PU6Ga9YfJlvisnwQ==|3M8dF2PFub9FP/SbgdenSQ==|KbaUQSb5IjVwhSbXDbCJbKXGBaCHEKDArrhvQDr9/QM=".to_string(), //
            value: "2./mtIg5EsCJbesKfmbWAx+A==|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".to_string(),
            note: "2.pMS6/icTQABtulw52pq2lg==|XXbxKxDTh+mWiN1HjH2N1w==|Q6PkuT+KX/axrgN9ubD5Ajk2YNwxQkgs3WJM0S0wtG8=".to_string(),
            creation_date: chrono::Utc::now(),
            revision_date: chrono::Utc::now(),
        }).collect::<Vec<_>>();

        secrets.push(SecretResponse {
            // TUX, the penguin
            id: id2,
            organization_id: Uuid::parse_str("ec2c1d46-6a4b-4751-a310-af9601317f2d").unwrap_or_else(|_| Uuid::new_v4()),
            project_id: Some(uuid::Uuid::new_v4()),
            key: "2.OldQj0RJKww0WN7RSxI1wQ==|TpxAbmdx6zIVo37YJ5n1aQ==|06Imyx7jqaZ5J5amrBboCVPwvPoDKB8REJdToQwp3dA=".to_string(),
            value: "2.oEDp566lC9VYHn6XmusxfA==|Gj23w5q2NZ4z9PNne1d0ug==|y7K5TgMJFI0T0yFwLXzAMf9OBANNT567hLQ+z7G2rac=".to_string(),
            note: "2.owktgGRm4r+ho4WY4U9zvA==|6Up5NQHyZ65SL3vbNI1GhQ==|vdvWvPpoB/J3aWXKBiruqOr1SK/ndkCCTjHf2vphhu4=".to_string(),
            creation_date: chrono::Utc::now(),
            revision_date: chrono::Utc::now(),
        });
        Json(GetByIdsResponse { data: secrets })
    }

    pub async fn delete_secrets(Json(ids): Json<Vec<Uuid>>) -> Json<Value> {
        info!("Deleting secrets with ids: {:?}", ids);

        for id in &ids {
            info!("Deleted secret with id: {}", id);
        }

        Json(json!({
            "message": "Secrets deleted successfully",
            "deleted_ids": ids
        }))
    }

    pub async fn sync_secrets(
        Path(org_id): Path<Uuid>,
        Query(params): Query<SyncQueryParams>,
    ) -> Json<SecretsSyncResponse> {
        info!("Syncing secrets for organization: {}", org_id);

        if let Some(date) = params.last_synced_date {
            if date < chrono::Utc::now() {
                return Json(SecretsSyncResponse {
                    has_changes: false,
                    secrets: None,
                });
            }
        }

        let secrets = vec![
            SecretResponse {
                id: uuid::Uuid::new_v4(),
                organization_id: org_id,
                project_id: Some(uuid::Uuid::new_v4()),
                key: "2.4MPPDSh3oQPfK0Sxny925g==|Rj4P0P6M3l/WD99KiwtRiQ==|6wcq4L8ZJ45h549RbiVd1sI26Q6QxaUPmQB2dAXVO1c=".to_string(),
                value: "2.78LSmLCl2XW8za0HyZUV2Q==|rwLKE0FGfyAZT9CUR9yUVg==|xZIthCfi1fzJOZ/riBQ3FXfuICBUv8jeE3Qrbg1f5D8=".to_string(),
                note: "2.M/Sa8H76DNUT1RFT082VFg==|ILTxxg3P4w4aqkY1q18I4w==|mdHLGBghl+apZirhuBfbAWkTENKBJV4GnM03fhbHxy0=".to_string(),
                creation_date: chrono::Utc::now(),
                revision_date: chrono::Utc::now(),
            },
        ];

        let secrets_response = serde_json::json!({
            "data": secrets
        });

        Json(SecretsSyncResponse {
            has_changes: true,
            secrets: Some(secrets_response),
        })
    }
}

pub mod projects {
    use bitwarden_sm::projects::{ProjectResponse, ProjectsResponse};

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CreateProjectRequest {
        pub name: String,
    }

    pub async fn list_projects(Path(org_id): Path<Uuid>) -> Json<ProjectsResponse> {
        info!("Listing projects for organization: {}", org_id);

        let projects =
        vec![
            ProjectResponse {
                id: uuid::Uuid::new_v4(),
                organization_id: org_id.clone(),
                name: "2.DmcNJqtzi+nPWY9gJR4nMw==|IEkn2x+C0YLmnQ/qm0EfOcMGcRZDkexFkDW9BPw3wRQ=|TxxTeBKqL0QYLT+89F0KfI81BbBryXnNNAjU9DGKuuY=".to_string(),
                creation_date: chrono::Utc::now(),
                revision_date: chrono::Utc::now(),
            },
            ProjectResponse {
                id: uuid::Uuid::new_v4(),
                organization_id: org_id,
                name: "2.4hWxQC9O5KpHcyCyI/xBsQ==|RLkyV/QbEMpxPnO91E/jPURCvsDIjI1ZIh6eMvGIuEg=|FXnie8Z9OtaBElnzF0v4Iut0fmy7IAI2IedEKJKuSp0=".to_string(),
                creation_date: chrono::Utc::now(),
                revision_date: chrono::Utc::now(),
            }
        ];

        Json(ProjectsResponse { data: projects })
    }

    pub async fn create_project(
        Path(org_id): Path<Uuid>,
        Json(payload): Json<CreateProjectRequest>,
    ) -> Json<ProjectResponse> {
        info!(
            "Creating project for organization {}: {:?}",
            org_id, payload
        );

        let project = bitwarden_sm::projects::ProjectResponse {
            id: uuid::Uuid::new_v4(),
            organization_id: org_id,
            name: payload.name,
            creation_date: chrono::Utc::now(),
            revision_date: chrono::Utc::now(),
        };

        Json(project)
    }

    pub async fn get_project(Path(id): Path<String>) -> Json<ProjectResponse> {
        info!("Getting project with id: {}", id);

        let project = ProjectResponse {
            id: Uuid::new_v4(),
            organization_id: Uuid::parse_str("ec2c1d46-6a4b-4751-a310-af9601317f2d").unwrap_or_else(|_| Uuid::new_v4()),
            name: "2.DmcNJqtzi+nPWY9gJR4nMw==|IEkn2x+C0YLmnQ/qm0EfOcMGcRZDkexFkDW9BPw3wRQ=|TxxTeBKqL0QYLT+89F0KfI81BbBryXnNNAjU9DGKuuY=".to_string(),
            creation_date: chrono::Utc::now(),
            revision_date: chrono::Utc::now(),
        };

        Json(project)
    }

    pub async fn delete_projects(Json(ids): Json<Vec<String>>) -> Json<Value> {
        info!("Deleting projects with ids: {:?}", ids);

        for id in &ids {
            info!("Deleted project with id: {}", id);
        }

        Json(json!({
            "message": "Projects deleted successfully",
            "deleted_ids": ids
        }))
    }
}

pub mod misc {
    use axum::response::Json;

    use super::*;

    pub async fn health_check() -> Json<Value> {
        Json(json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now()
        }))
    }

    pub async fn echo(Json(payload): Json<Value>) -> Json<Value> {
        info!("Echo request: {:?}", payload);
        Json(payload)
    }

    pub async fn help() -> Json<Value> {
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
                    "method": "GET",
                    "path": "/api/organizations/:org_id/secrets/sync",
                    "description": "Sync secrets for an organization."
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
}
