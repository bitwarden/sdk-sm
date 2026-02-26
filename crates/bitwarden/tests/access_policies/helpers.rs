//! Shared test helpers: mock server setup, common fixtures, and integration config loading.

#![allow(dead_code)]

use bitwarden::secrets_manager::{
    AccessTokenLoginRequest, ClientSettings, DeviceType, SecretsManagerClient,
};
use uuid::Uuid;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

/// Organization ID embedded in the test JWT (`organization` claim).
pub const TEST_ORG_ID: &str = "f4e44a7f-1190-432a-9d4a-af96013127cb";

/// Service account access token.
///
/// This value is cryptographically tied to `TOKEN_RESPONSE_BODY` — the encrypted_payload in
/// that response was encrypted with a key derived from this access token. Both values are taken
/// from the existing wiremock fixture in `client_secrets.rs` and must not be changed independently.
pub const TEST_ACCESS_TOKEN: &str = "0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ==";

/// Returns the JSON body for a successful identity token response.
pub fn token_response() -> serde_json::Value {
    serde_json::json!({
        "access_token": "eyJhbGciOiJSUzI1NiIsImtpZCI6IjMwMURENkE1MEU4NEUxRDA5MUM4MUQzQjAwQkY5MDEwQzg1REJEOUFSUzI1NiIsInR5cCI6ImF0K2p3dCIsIng1dCI6Ik1CM1dwUTZFNGRDUnlCMDdBTC1RRU1oZHZabyJ9.eyJuYmYiOjE2NzUxMDM3ODEsImV4cCI6MTY3NTEwNzM4MSwiaXNzIjoiaHR0cDovL2xvY2FsaG9zdCIsImNsaWVudF9pZCI6ImVjMmMxZDQ2LTZhNGItNDc1MS1hMzEwLWFmOTYwMTMxN2YyZCIsInN1YiI6ImQzNDgwNGNhLTRmNmMtNDM5Mi04NmI3LWFmOTYwMTMxNzVkMCIsIm9yZ2FuaXphdGlvbiI6ImY0ZTQ0YTdmLTExOTAtNDMyYS05ZDRhLWFmOTYwMTMxMjdjYiIsImp0aSI6IjU3QUU0NzQ0MzIwNzk1RThGQkQ4MUIxNDA2RDQyNTQyIiwiaWF0IjoxNjc1MTAzNzgxLCJzY29wZSI6WyJhcGkuc2VjcmV0cyJdfQ.GRKYzqgJZHEEZHsJkhVZH8zjYhY3hUvM4rhdV3FU10WlCteZdKHrPIadCUh-Oz9DxIAA2HfALLhj1chL4JgwPmZgPcVS2G8gk8XeBmZXowpVWJ11TXS1gYrM9syXbv9j0JUCdpeshH7e56WnlpVynyUwIum9hmYGZ_XJUfmGtlKLuNjYnawTwLEeR005uEjxq3qI1kti-WFnw8ciL4a6HLNulgiFw1dAvs4c7J0souShMfrnFO3gSOHff5kKD3hBB9ynDBnJQSFYJ7dFWHIjhqs0Vj-9h0yXXCcHvu7dVGpaiNjNPxbh6YeXnY6UWcmHLDtFYsG2BWcNvVD4-VgGxXt3cMhrn7l3fSYuo32ZYk4Wop73XuxqF2fmfmBdZqGI1BafhENCcZw_bpPSfK2uHipfztrgYnrzwvzedz0rjFKbhDyrjzuRauX5dqVJ4ntPeT9g_I5n71gLxiP7eClyAx5RxdF6He87NwC8i-hLBhugIvLTiDj-Sk9HvMth6zaD0ebxd56wDjq8-CMG_WcgusDqNzKFHqWNDHBXt8MLeTgZAR2rQMIMFZqFgsJlRflbig8YewmNUA9wAU74TfxLY1foO7Xpg49vceB7C-PlvGi1VtX6F2i0tc_67lA5kWXnnKBPBUyspoIrmAUCwfms5nTTqA9xXAojMhRHAos_OdM",
        "expires_in": 3600,
        "token_type": "Bearer",
        "scope": "api.secrets",
        "encrypted_payload": "2.E9fE8+M/VWMfhhim1KlCbQ==|eLsHR484S/tJbIkM6spnG/HP65tj9A6Tba7kAAvUp+rYuQmGLixiOCfMsqt5OvBctDfvvr/AesBu7cZimPLyOEhqEAjn52jF0eaI38XZfeOG2VJl0LOf60Wkfh3ryAMvfvLj3G4ZCNYU8sNgoC2+IQ==|lNApuCQ4Pyakfo/wwuuajWNaEX/2MW8/3rjXB/V7n+k="
    })
}

/// Returns a `Mock` for the identity token endpoint.
pub fn token_mock() -> Mock {
    Mock::given(matchers::method("POST"))
        .and(matchers::path("/identity/connect/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(token_response()))
}

/// Starts a wiremock server, registers `mocks`, and returns an unauthenticated SDK client.
///
/// The caller must include `token_mock()` in `mocks` before calling `login_access_token`.
pub async fn mock_server_with(mocks: Vec<Mock>) -> (MockServer, SecretsManagerClient) {
    let server = MockServer::start().await;
    for mock in mocks {
        server.register(mock).await;
    }
    let client = SecretsManagerClient::new(Some(ClientSettings {
        identity_url: format!("http://{}/identity", server.address()),
        api_url: format!("http://{}/api", server.address()),
        user_agent: "Bitwarden Rust-SDK [TEST]".into(),
        device_type: DeviceType::SDK,
        device_identifier: None,
        bitwarden_client_version: None,
        bitwarden_package_type: None,
    }));
    (server, client)
}

/// Starts a wiremock server with `extra_mocks`, authenticates the SDK client with the test token,
/// and returns both the server handle and the authenticated client.
pub async fn authenticated_client(extra_mocks: Vec<Mock>) -> (MockServer, SecretsManagerClient) {
    let mut mocks = vec![token_mock()];
    mocks.extend(extra_mocks);
    let (server, client) = mock_server_with(mocks).await;
    client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token: TEST_ACCESS_TOKEN.into(),
            state_file: None,
        })
        .await
        .expect("test authentication failed — token_response() and TEST_ACCESS_TOKEN must match");
    (server, client)
}

// ──────────────────────────────────────────────────────────────────────────────
// JSON fixtures for mock responses
// ──────────────────────────────────────────────────────────────────────────────

/// People access policies response with one user who has `manage: true`.
pub fn people_response_manage_true(user_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "userAccessPolicies": [{
            "organizationUserId": user_id,
            "organizationUserName": "Admin User",
            "currentUser": false,
            "read": true,
            "write": true,
            "manage": true,
            "id": Uuid::new_v4(),
            "creationDate": "2024-01-01T00:00:00Z",
            "revisionDate": "2024-01-01T00:00:00Z"
        }],
        "groupAccessPolicies": []
    })
}

/// People access policies response with one user who has `manage: false`.
pub fn people_response_manage_false(user_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "userAccessPolicies": [{
            "organizationUserId": user_id,
            "organizationUserName": "Admin User",
            "currentUser": false,
            "read": true,
            "write": true,
            "manage": false,
            "id": Uuid::new_v4(),
            "creationDate": "2024-01-01T00:00:00Z",
            "revisionDate": "2024-01-01T00:00:00Z"
        }],
        "groupAccessPolicies": []
    })
}

/// Empty people access policies (no users, no groups).
pub fn empty_people_response() -> serde_json::Value {
    serde_json::json!({
        "userAccessPolicies": [],
        "groupAccessPolicies": []
    })
}

/// Empty service account access policies.
pub fn empty_sa_response() -> serde_json::Value {
    serde_json::json!({ "serviceAccountAccessPolicies": [] })
}

/// Service account access policies response with one SA that has `manage: true`.
/// SA name is encrypted (same as the real server returns).
pub fn sa_response_manage_true(sa_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "serviceAccountAccessPolicies": [{
            "serviceAccountId": sa_id,
            "serviceAccountName": ENCRYPTED_TEST,
            "read": true,
            "write": true,
            "manage": true,
            "id": Uuid::new_v4(),
            "creationDate": "2024-01-01T00:00:00Z",
            "revisionDate": "2024-01-01T00:00:00Z"
        }]
    })
}

/// Combined access policies response for secrets (users + groups + SAs).
/// SA name is encrypted (same as the real server returns).
pub fn secret_policies_response_manage_true(user_id: Uuid, sa_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "userAccessPolicies": [{
            "organizationUserId": user_id,
            "organizationUserName": "Admin User",
            "currentUser": false,
            "read": true,
            "write": true,
            "manage": true,
            "id": Uuid::new_v4(),
            "creationDate": "2024-01-01T00:00:00Z",
            "revisionDate": "2024-01-01T00:00:00Z"
        }],
        "groupAccessPolicies": [],
        "serviceAccountAccessPolicies": [{
            "serviceAccountId": sa_id,
            "serviceAccountName": ENCRYPTED_TEST,
            "read": true,
            "write": true,
            "manage": true,
            "id": Uuid::new_v4(),
            "creationDate": "2024-01-01T00:00:00Z",
            "revisionDate": "2024-01-01T00:00:00Z"
        }]
    })
}

/// Granted policies response with one project that has `manage: true` and `has_permission: true`.
/// Project name is encrypted (same as the real server returns).
pub fn granted_policies_response_manage_true(project_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "grantedProjectPolicies": [{
            "accessPolicy": {
                "grantedProjectId": project_id,
                "grantedProjectName": ENCRYPTED_TEST,
                "read": true,
                "write": true,
                "manage": true,
                "id": Uuid::new_v4(),
                "creationDate": "2024-01-01T00:00:00Z",
                "revisionDate": "2024-01-01T00:00:00Z"
            },
            "hasPermission": true
        }]
    })
}

/// Empty granted policies.
pub fn empty_granted_response() -> serde_json::Value {
    serde_json::json!({ "grantedProjectPolicies": [] })
}

/// Potential grantees response with one user and one group.
pub fn people_potential_grantees_response(user_id: Uuid, group_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "data": [
            {
                "id": user_id,
                "name": "Admin User",
                "type": "user",
                "email": "admin@example.com"
            },
            {
                "id": group_id,
                "name": "Developers",
                "type": "group",
                "email": null
            }
        ]
    })
}

/// Encrypted form of "TEST", encrypted with the org key derived from TEST_ACCESS_TOKEN.
/// Use this for project and service account names in mock responses (they are stored encrypted).
pub const ENCRYPTED_TEST: &str = "2.pMS6/icTQABtulw52pq2lg==|XXbxKxDTh+mWiN1HjH2N1w==|Q6PkuT+KX/axrgN9ubD5Ajk2YNwxQkgs3WJM0S0wtG8=";

/// Potential grantees response with one service account (name is encrypted).
pub fn sa_potential_grantees_response(sa_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "data": [{
            "id": sa_id,
            "name": ENCRYPTED_TEST,
            "type": "serviceAccount",
            "email": null
        }]
    })
}

/// Potential grantees response with one project (name is encrypted).
pub fn project_potential_grantees_response(project_id: Uuid) -> serde_json::Value {
    serde_json::json!({
        "data": [{
            "id": project_id,
            "name": ENCRYPTED_TEST,
            "type": "project",
            "email": null
        }]
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// Integration test configuration
// ──────────────────────────────────────────────────────────────────────────────

/// Configuration loaded from environment variables for integration tests.
///
/// Set `SM_INTEGRATION_TESTS=1` to enable integration tests.
/// All other variables are required when `SM_INTEGRATION_TESTS` is set.
#[derive(Clone)]
pub struct IntegrationConfig {
    /// Base URL of the Bitwarden server (e.g. `http://localhost:8080`).
    pub base_url: String,
    /// Organization ID (UUID).
    pub org_id: Uuid,
    /// Project where `SA_MANAGE_TOKEN` has Manage permission (is creator).
    pub project_id_manage: Uuid,
    /// Project where `SA_WRITE_ONLY_TOKEN` has Write but not Manage.
    pub project_id_write_only: Uuid,
    /// Project with exactly one human grantee having Manage (for lockout tests).
    pub project_id_lockout: Uuid,
    /// Secret where `SA_MANAGE_TOKEN` has Manage permission.
    pub secret_id_manage: Uuid,
    /// Service account access token with Read-only on test resources.
    pub sa_read_only_token: String,
    /// Service account access token with Write but not Manage on test resources.
    pub sa_write_only_token: String,
    /// Service account access token with Manage on test resources (is project creator).
    pub sa_manage_token: String,
    /// Optional: SA access token with Manage but not creator (for delegation constraint tests).
    pub sa_manage_non_creator_token: Option<String>,
    /// Organization user ID of a test user (for grantee tests).
    pub grantee_user_id: Uuid,
    /// Optional: group ID of a test group.
    pub grantee_group_id: Option<Uuid>,
    /// Service account ID of a second SA (for granted-policies tests).
    pub grantee_sa_id: Uuid,
}

/// Returns `Some(IntegrationConfig)` if `SM_INTEGRATION_TESTS=1` and all required env vars are
/// set. Returns `None` if `SM_INTEGRATION_TESTS` is unset.
///
/// Panics with a descriptive message if `SM_INTEGRATION_TESTS=1` but a required variable is
/// missing.
pub fn integration_config() -> Option<IntegrationConfig> {
    if std::env::var("SM_INTEGRATION_TESTS").is_err() {
        return None;
    }

    fn require(key: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| {
            panic!("Integration test requires env var {key}. Set SM_INTEGRATION_TESTS=1 only after all required variables are set. See tests/access_policies/integration.rs for the full list.")
        })
    }

    fn require_uuid(key: &str) -> Uuid {
        require(key)
            .parse()
            .unwrap_or_else(|_| panic!("{key} must be a valid UUID"))
    }

    fn optional_uuid(key: &str) -> Option<Uuid> {
        std::env::var(key).ok().map(|v| {
            v.parse()
                .unwrap_or_else(|_| panic!("{key} must be a valid UUID"))
        })
    }

    Some(IntegrationConfig {
        base_url: std::env::var("SM_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
        org_id: require_uuid("SM_ORG_ID"),
        project_id_manage: require_uuid("PROJECT_ID_MANAGE"),
        project_id_write_only: require_uuid("PROJECT_ID_WRITE_ONLY"),
        project_id_lockout: require_uuid("PROJECT_ID_LOCKOUT"),
        secret_id_manage: require_uuid("SECRET_ID_MANAGE"),
        sa_read_only_token: require("SA_READ_ONLY_TOKEN"),
        sa_write_only_token: require("SA_WRITE_ONLY_TOKEN"),
        sa_manage_token: require("SA_MANAGE_TOKEN"),
        sa_manage_non_creator_token: std::env::var("SA_MANAGE_NON_CREATOR_TOKEN").ok(),
        grantee_user_id: require_uuid("GRANTEE_USER_ID"),
        grantee_group_id: optional_uuid("GRANTEE_GROUP_ID"),
        grantee_sa_id: require_uuid("GRANTEE_SA_ID"),
    })
}

/// Creates an authenticated SDK client pointing at the real server, using the given access token.
pub async fn real_server_client(config: &IntegrationConfig, token: &str) -> SecretsManagerClient {
    let client = SecretsManagerClient::new(Some(ClientSettings {
        identity_url: format!("{}/identity", config.base_url),
        api_url: format!("{}/api", config.base_url),
        user_agent: "Bitwarden Rust-SDK [TEST]".into(),
        device_type: DeviceType::SDK,
        device_identifier: None,
        bitwarden_client_version: None,
        bitwarden_package_type: None,
    }));
    client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token: token.into(),
            state_file: None,
        })
        .await
        .expect("integration test authentication failed");
    client
}
