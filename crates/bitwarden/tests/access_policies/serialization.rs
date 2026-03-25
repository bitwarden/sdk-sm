use bitwarden::secrets_manager::{
    AccessPoliciesClientExt,
    access_policies::{
        AccessPolicyEntry, GetProjectAccessPoliciesRequest, PutProjectAccessPoliciesRequest,
    },
};
use uuid::Uuid;
use wiremock::{Mock, ResponseTemplate, matchers};

use crate::helpers;

// ──────────────────────────────────────────────────────────────────────────────
// T1: manage field is present in serialized JSON (pure unit tests, always pass)
// ──────────────────────────────────────────────────────────────────────────────

/// T1a: `manage: false` must not be omitted from serialized JSON.
///
/// If `manage` were `Option<bool>` serialized with `skip_serializing_if = "Option::is_none"`,
/// a `false` value would be silently dropped, causing the server to default it (potentially
/// to `true`). The field is `bool` + always-serialize to prevent this.
#[test]
fn t1a_manage_false_present_in_serialized_json() {
    let entry = AccessPolicyEntry {
        grantee_id: Uuid::new_v4(),
        read: true,
        write: true,
        manage: false,
    };
    let json = serde_json::to_string(&entry).expect("serialization must not fail");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("round-trip parse must not fail");

    assert_eq!(
        parsed["manage"],
        serde_json::Value::Bool(false),
        "manage:false must be explicitly present in the JSON payload"
    );
    assert_eq!(parsed["read"], serde_json::Value::Bool(true));
    assert_eq!(parsed["write"], serde_json::Value::Bool(true));
    assert!(
        parsed.get("granteeId").is_some(),
        "granteeId must be present (camelCase rename)"
    );
}

/// T1b: `manage: true` must also be serialized explicitly.
#[test]
fn t1b_manage_true_present_in_serialized_json() {
    let entry = AccessPolicyEntry {
        grantee_id: Uuid::new_v4(),
        read: true,
        write: true,
        manage: true,
    };
    let json = serde_json::to_string(&entry).expect("serialization must not fail");
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("round-trip parse must not fail");

    assert_eq!(
        parsed["manage"],
        serde_json::Value::Bool(true),
        "manage:true must be explicitly present in the JSON payload"
    );
}

/// T1c: Round-trip of `AccessPolicyEntry` preserves all fields including `manage`.
#[test]
fn t1c_access_policy_entry_round_trips() {
    let grantee_id = Uuid::new_v4();
    let entry = AccessPolicyEntry {
        grantee_id,
        read: true,
        write: false,
        manage: true,
    };
    let json = serde_json::to_string(&entry).unwrap();
    let restored: AccessPolicyEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.grantee_id, grantee_id);
    assert_eq!(restored.read, true);
    assert_eq!(restored.write, false);
    assert_eq!(
        restored.manage, true,
        "manage must survive a JSON round-trip"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T2: GET response with manage:true is correctly returned by the SDK.
// ──────────────────────────────────────────────────────────────────────────────

/// T2: SDK correctly surfaces `manage: true` returned by the server in GET project policies.
///
/// The server returns a user access policy with `manage: true`. The SDK must not silently
/// downgrade it to `false`.
#[tokio::test]
async fn t2_get_project_policies_preserves_manage_true() {
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(helpers::people_response_manage_true(user_id)),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(helpers::empty_sa_response())),
    ])
    .await;

    let response = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("get_project_policies must succeed");

    assert_eq!(
        response.user_access_policies.len(),
        1,
        "response must contain the one user access policy"
    );
    let policy = &response.user_access_policies[0];
    assert_eq!(policy.organization_user_id, user_id);
    assert!(policy.policy.read, "read must be true");
    assert!(policy.policy.write, "write must be true");
    assert!(
        policy.policy.manage,
        "manage must be true when server returns manage:true"
    );
}

/// T2b: SDK correctly surfaces `manage: false` — not an accidental `true`.
///
/// Verifies that `manage: false` from the server is not incorrectly promoted to `true`.
/// (This currently passes due to the hardcoded false, but verifies no over-promotion bug.)
#[tokio::test]
async fn t2b_get_project_policies_preserves_manage_false() {
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(helpers::people_response_manage_false(user_id)),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(helpers::empty_sa_response())),
    ])
    .await;

    let response = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("get_project_policies must succeed");

    let policy = &response.user_access_policies[0];
    assert!(
        !policy.policy.manage,
        "manage must be false when server returns manage:false"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T3: PUT request body contains the `manage` field.
// ──────────────────────────────────────────────────────────────────────────────

/// T3: SDK sends `manage: true` in the PUT request body for project people policies.
///
/// The SDK must include the `manage` field in the HTTP request body sent to the server.
#[tokio::test]
async fn t3_put_project_policies_sends_manage_field() {
    let project_id = Uuid::new_v4();
    let grantee_id = Uuid::new_v4();

    let (server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_people_response()),
            ),
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(helpers::empty_sa_response())),
        // put_project_access_policies re-fetches with GET after both PUTs
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_people_response()),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(helpers::empty_sa_response())),
    ])
    .await;

    client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id,
                read: true,
                write: true,
                manage: true,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(vec![]),
        })
        .await
        .expect("put_project_policies must succeed");

    // Inspect what was actually sent to the people PUT endpoint.
    let all_requests = server
        .received_requests()
        .await
        .expect("wiremock must track requests");
    let put_people = all_requests
        .iter()
        .find(|r| r.method.as_str() == "PUT" && r.url.path().contains("access-policies/people"))
        .expect("SDK must send PUT to .../access-policies/people");

    let body: serde_json::Value =
        serde_json::from_slice(&put_people.body).expect("PUT request body must be valid JSON");

    let user_requests = &body["userAccessPolicyRequests"];
    assert!(
        user_requests.is_array() && !user_requests.as_array().unwrap().is_empty(),
        "userAccessPolicyRequests must not be empty in the PUT body"
    );

    let first = &user_requests[0];
    assert_eq!(
        first["manage"],
        serde_json::Value::Bool(true),
        "manage:true must be present in the PUT request body"
    );
}

/// T3b: Manages `manage: false` is also sent explicitly (not omitted).
///
/// This test ensures that `manage: false` is not omitted from the PUT body via
/// a `skip_serializing_if` or `Option<bool>` pattern.
#[tokio::test]
async fn t3b_put_project_policies_sends_manage_false_explicitly() {
    let project_id = Uuid::new_v4();
    let grantee_id = Uuid::new_v4();

    let (server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_people_response()),
            ),
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(helpers::empty_sa_response())),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_people_response()),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(helpers::empty_sa_response())),
    ])
    .await;

    client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id,
                read: true,
                write: false,
                manage: false,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(vec![]),
        })
        .await
        .expect("put_project_policies must succeed");

    let all_requests = server.received_requests().await.unwrap();
    let put_people = all_requests
        .iter()
        .find(|r| r.method.as_str() == "PUT" && r.url.path().contains("access-policies/people"))
        .expect("SDK must send PUT to .../access-policies/people");

    let body: serde_json::Value = serde_json::from_slice(&put_people.body).unwrap();
    let first = &body["userAccessPolicyRequests"][0];

    assert_eq!(
        first["manage"],
        serde_json::Value::Bool(false),
        "manage:false must be explicitly present in PUT body — must not be omitted by SDK"
    );
}
