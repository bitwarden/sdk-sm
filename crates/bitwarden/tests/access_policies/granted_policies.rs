//! T11 — Service account granted policies GET/PUT tests using wiremock.
//!
//! These tests verify that the SDK correctly calls the granted-policies endpoints and
//! assembles responses, including correct preservation of the `manage` field.

use bitwarden::secrets_manager::{
    AccessPoliciesClientExt,
    access_policies::{GetGrantedPoliciesRequest, GrantedProjectEntry, PutGrantedPoliciesRequest},
};
use uuid::Uuid;
use wiremock::{Mock, ResponseTemplate, matchers};

use crate::helpers;

/// T11a: SA can GET its own granted project policies.
///
/// The SDK calls `GET /api/service-accounts/{id}/granted-policies` and returns
/// the list of projects this SA has been granted access to.
#[tokio::test]
async fn t11a_get_granted_policies_returns_project_list() {
    let sa_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(helpers::granted_policies_response_manage_true(project_id)),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .get_granted_policies(&GetGrantedPoliciesRequest {
            service_account_id: sa_id,
        })
        .await
        .expect("get_granted_policies must succeed when server returns 200");

    assert_eq!(response.granted_project_policies.len(), 1);
    let policy = &response.granted_project_policies[0];
    assert_eq!(policy.project_id, project_id);
    assert_eq!(policy.project_name, Some("TEST".into()), "project name must be decrypted");
    assert!(policy.has_permission, "has_permission must be true");
    assert!(policy.policy.read, "read must be true");
    assert!(policy.policy.write, "write must be true");
    assert!(
        policy.policy.manage,
        "manage:true must be preserved from fixture"
    );
}

/// T11b: GET granted policies — `manage: true` is preserved from server response.
#[tokio::test]
async fn t11b_get_granted_policies_preserves_manage_true() {
    let sa_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(helpers::granted_policies_response_manage_true(project_id)),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .get_granted_policies(&GetGrantedPoliciesRequest {
            service_account_id: sa_id,
        })
        .await
        .expect("get_granted_policies must succeed");

    assert!(
        response.granted_project_policies[0].policy.manage,
        "manage:true must be preserved from server response"
    );
}

/// T11c: GET granted policies — empty list returns correctly.
#[tokio::test]
async fn t11c_get_granted_policies_empty_list() {
    let sa_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_granted_response()),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .get_granted_policies(&GetGrantedPoliciesRequest {
            service_account_id: sa_id,
        })
        .await
        .expect("get_granted_policies must succeed");

    assert!(response.granted_project_policies.is_empty());
}

/// T11d: SA can PUT (assign) granted project policies.
///
/// The SDK calls `PUT /api/service-accounts/{id}/granted-policies` and then re-fetches.
/// The returned response reflects the updated state.
#[tokio::test]
async fn t11d_put_granted_policies_assigns_project_access() {
    let sa_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_granted_response()),
            ),
        // put_granted_policies re-fetches after the PUT
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(helpers::granted_policies_response_manage_true(project_id)),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .put_granted_policies(&PutGrantedPoliciesRequest {
            service_account_id: sa_id,
            projects: vec![GrantedProjectEntry {
                project_id,
                read: true,
                write: true,
                manage: true,
            }],
        })
        .await
        .expect("put_granted_policies must succeed when server returns 200");

    assert_eq!(response.granted_project_policies.len(), 1);
    assert_eq!(response.granted_project_policies[0].project_id, project_id);
    assert!(
        response.granted_project_policies[0].policy.manage,
        "manage:true must be preserved in re-fetch after PUT"
    );
}

/// T11e: PUT granted policies — request body includes `manage` field.
///
/// The `GrantedProjectEntry` struct uses `bool` for `manage`, which must be included
/// in the PUT request body. Verify the body sent to the server.
#[tokio::test]
async fn t11e_put_granted_policies_sends_manage_field() {
    let sa_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();

    let (server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_granted_response()),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_granted_response()),
            ),
    ])
    .await;

    client
        .access_policies()
        .put_granted_policies(&PutGrantedPoliciesRequest {
            service_account_id: sa_id,
            projects: vec![GrantedProjectEntry {
                project_id,
                read: true,
                write: true,
                manage: true,
            }],
        })
        .await
        .expect("put_granted_policies must succeed");

    let all_requests = server.received_requests().await.unwrap();
    let put_req = all_requests
        .iter()
        .find(|r| r.method.as_str() == "PUT")
        .expect("SDK must send PUT request");

    let body: serde_json::Value = serde_json::from_slice(&put_req.body).unwrap();
    let entries = body["projectGrantedPolicyRequests"]
        .as_array()
        .expect("projectGrantedPolicyRequests must be an array in PUT body");

    assert_eq!(entries.len(), 1);
    // Note: the `manage` field in GrantedAccessPolicyRequest (from bitwarden-api-api) may also
    // be missing, analogous to BUG-2 for project policies. If manage is absent here,
    // this assertion will fail and document the same category of bug.
    assert_eq!(
        entries[0]["manage"],
        serde_json::Value::Bool(true),
        "manage:true must be present in PUT granted-policies request body"
    );
}

/// T11f: PUT granted policies — empty list removes all project access (full-replace semantics).
#[tokio::test]
async fn t11f_put_granted_policies_empty_list_revokes_all_access() {
    let sa_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_granted_response()),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/service-accounts/{sa_id}/granted-policies"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_granted_response()),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .put_granted_policies(&PutGrantedPoliciesRequest {
            service_account_id: sa_id,
            projects: vec![],
        })
        .await
        .expect("PUT with empty projects must succeed");

    assert!(response.granted_project_policies.is_empty());
}
