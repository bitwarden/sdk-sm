//! T5 — Project access policy GET/PUT happy-path tests using wiremock.
//!
//! These tests verify that SDK calls for project people and service-account access policies
//! succeed under normal conditions when the caller has Manage permission.

use bitwarden::secrets_manager::{
    AccessPoliciesClientExt,
    access_policies::{
        AccessPolicyEntry, GetProjectAccessPoliciesRequest, PutProjectAccessPoliciesRequest,
    },
};
use uuid::Uuid;
use wiremock::{Mock, ResponseTemplate, matchers};

use crate::helpers;

/// T5a: SA with Manage can GET project people access policies.
///
/// Verifies the SDK correctly calls both the people and service-account sub-endpoints
/// and assembles them into a single `AccessPoliciesResponse`.
#[tokio::test]
async fn t5a_sa_manage_can_get_project_people_policies() {
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let sa_id = Uuid::new_v4();

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
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::sa_response_manage_true(sa_id)),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("get_project_policies must succeed when server returns 200");

    assert_eq!(response.user_access_policies.len(), 1);
    assert_eq!(
        response.user_access_policies[0].organization_user_id,
        user_id
    );
    assert!(
        !response.user_access_policies[0].policy.manage,
        "user manage must be false per fixture"
    );

    assert_eq!(response.service_account_access_policies.len(), 1);
    assert_eq!(
        response.service_account_access_policies[0].service_account_id,
        sa_id
    );
    assert!(
        response.service_account_access_policies[0].policy.manage,
        "SA manage must be true per fixture"
    );
    assert!(
        response.service_account_access_policies[0].policy.read,
        "SA read must be true per fixture"
    );

    assert!(
        response.group_access_policies.is_empty(),
        "no group policies were configured"
    );
}

/// T5b: SA with Manage can GET project policies — group access policies are included.
#[tokio::test]
async fn t5b_get_project_policies_includes_group_policies() {
    let project_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    let people_response = serde_json::json!({
        "userAccessPolicies": [],
        "groupAccessPolicies": [{
            "groupId": group_id,
            "groupName": "Developers",
            "currentUserInGroup": false,
            "read": true,
            "write": true,
            "manage": false,
            "id": Uuid::new_v4(),
            "creationDate": "2024-01-01T00:00:00Z",
            "revisionDate": "2024-01-01T00:00:00Z"
        }]
    });

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(people_response)),
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

    assert_eq!(response.group_access_policies.len(), 1);
    assert_eq!(response.group_access_policies[0].group_id, group_id);
    assert_eq!(
        response.group_access_policies[0].group_name,
        Some("Developers".into())
    );
    assert!(response.group_access_policies[0].policy.read);
    assert!(response.group_access_policies[0].policy.write);
}

/// T5c: SA with Manage can PUT project people and service-account access policies.
///
/// Verifies the SDK sends PUT requests to both sub-endpoints, then re-fetches.
/// The returned `AccessPoliciesResponse` reflects the post-PUT state.
/// TC-PSA-02: SA policy is present in re-fetch and manage is preserved.
#[tokio::test]
async fn t5c_sa_manage_can_put_project_people_policies() {
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let sa_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
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
                ResponseTemplate::new(200)
                    .set_body_json(helpers::people_response_manage_false(user_id)),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::sa_response_manage_true(sa_id)),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: user_id,
                read: true,
                write: true,
                manage: false,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: sa_id,
                read: true,
                write: true,
                manage: true,
            }]),
        })
        .await
        .expect("put_project_policies must succeed when server returns 200");

    assert_eq!(
        response.user_access_policies.len(),
        1,
        "re-fetched response must reflect the applied user policies"
    );
    assert_eq!(
        response.user_access_policies[0].organization_user_id,
        user_id
    );

    // TC-PSA-02: SA policy must be present in re-fetch with manage preserved
    assert_eq!(
        response.service_account_access_policies.len(),
        1,
        "SA policy must appear in re-fetched response"
    );
    assert_eq!(
        response.service_account_access_policies[0].service_account_id, sa_id,
        "correct SA must be returned in re-fetch"
    );
    assert!(
        response.service_account_access_policies[0].policy.manage,
        "SA manage:true must be preserved in re-fetched response"
    );
}

/// T5d: PUT with empty user and group lists succeeds (removes all people policies).
///
/// Full-replace semantics: an empty PUT clears all policies on the people endpoint.
/// This does NOT trigger lockout protection in the mock (server logic is not emulated here).
#[tokio::test]
async fn t5d_put_project_policies_with_empty_people_succeeds() {
    let project_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
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

    let response = client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(vec![]),
        })
        .await
        .expect("PUT with empty lists must succeed (server enforces lockout protection, not SDK)");

    assert!(response.user_access_policies.is_empty());
    assert!(response.group_access_policies.is_empty());
    assert!(response.service_account_access_policies.is_empty());
}

/// T5e: PUT request body separates user/group entries (people endpoint) from SA entries.
///
/// Verifies the SDK correctly routes users and groups to the people endpoint, and
/// service accounts to the service-accounts endpoint.
#[tokio::test]
async fn t5e_put_routes_sa_policies_to_correct_endpoint() {
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let sa_id = Uuid::new_v4();

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
                grantee_id: user_id,
                read: true,
                write: false,
                manage: false,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: sa_id,
                read: true,
                write: true,
                manage: false,
            }]),
        })
        .await
        .expect("put_project_policies must succeed");

    let all_requests = server.received_requests().await.unwrap();

    let put_people = all_requests
        .iter()
        .find(|r| r.method.as_str() == "PUT" && r.url.path().contains("access-policies/people"))
        .expect("people PUT must be present");
    let put_sa = all_requests
        .iter()
        .find(|r| {
            r.method.as_str() == "PUT" && r.url.path().contains("access-policies/service-accounts")
        })
        .expect("service-accounts PUT must be present");

    let people_body: serde_json::Value = serde_json::from_slice(&put_people.body).unwrap();
    let sa_body: serde_json::Value = serde_json::from_slice(&put_sa.body).unwrap();

    let user_requests = people_body["userAccessPolicyRequests"]
        .as_array()
        .expect("userAccessPolicyRequests must be an array");
    assert_eq!(user_requests.len(), 1, "one user entry in people PUT");

    let sa_requests = sa_body["serviceAccountAccessPolicyRequests"]
        .as_array()
        .expect("serviceAccountAccessPolicyRequests must be an array");
    assert_eq!(sa_requests.len(), 1, "one SA entry in SA PUT");
}

/// T5f: PUT with people `Some` and SA `None` skips the SA PUT entirely.
///
/// When `service_account_access_policies` is `None`, the SDK must not send
/// a PUT to the service-accounts endpoint, preserving existing SA policies.
#[tokio::test]
async fn t5f_put_with_sa_none_skips_sa_put() {
    let project_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let (server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("PUT"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/people"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::empty_people_response()),
            ),
        // No SA PUT mock — if the SDK sends one, wiremock returns 404
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

    client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: user_id,
                read: true,
                write: true,
                manage: false,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: None, // skip SA PUT
        })
        .await
        .expect("put_project_policies must succeed when SA is None");

    let all_requests = server.received_requests().await.unwrap();
    let sa_puts: Vec<_> = all_requests
        .iter()
        .filter(|r| {
            r.method.as_str() == "PUT" && r.url.path().contains("access-policies/service-accounts")
        })
        .collect();

    assert!(
        sa_puts.is_empty(),
        "no PUT to service-accounts endpoint when service_account_access_policies is None"
    );

    let people_puts: Vec<_> = all_requests
        .iter()
        .filter(|r| r.method.as_str() == "PUT" && r.url.path().contains("access-policies/people"))
        .collect();

    assert_eq!(people_puts.len(), 1, "people PUT must still be sent");
}

/// T5g: PUT with SA `Some` and people `None` skips the people PUT entirely.
///
/// When both `user_access_policies` and `group_access_policies` are `None`,
/// the SDK must not send a PUT to the people endpoint.
#[tokio::test]
async fn t5g_put_with_people_none_skips_people_put() {
    let project_id = Uuid::new_v4();
    let sa_id = Uuid::new_v4();

    let (server, client) = helpers::authenticated_client(vec![
        // No people PUT mock — if the SDK sends one, wiremock returns 404
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
            .respond_with(
                ResponseTemplate::new(200).set_body_json(helpers::sa_response_manage_true(sa_id)),
            ),
    ])
    .await;

    client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: None, // skip people PUT
            group_access_policies: None,
            service_account_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: sa_id,
                read: true,
                write: true,
                manage: true,
            }]),
        })
        .await
        .expect("put_project_policies must succeed when people is None");

    let all_requests = server.received_requests().await.unwrap();
    let people_puts: Vec<_> = all_requests
        .iter()
        .filter(|r| r.method.as_str() == "PUT" && r.url.path().contains("access-policies/people"))
        .collect();

    assert!(
        people_puts.is_empty(),
        "no PUT to people endpoint when user and group are both None"
    );

    let sa_puts: Vec<_> = all_requests
        .iter()
        .filter(|r| {
            r.method.as_str() == "PUT" && r.url.path().contains("access-policies/service-accounts")
        })
        .collect();

    assert_eq!(sa_puts.len(), 1, "SA PUT must still be sent");
}

/// TC-PP-03: Appending a user entry — policy count increases from 1 to 2.
///
/// Verifies that a re-GET after PUT with 2 users reflects both entries.
#[tokio::test]
async fn t5h_append_user_entry_count_increases() {
    let project_id = Uuid::new_v4();
    let user_id_1 = Uuid::new_v4();
    let user_id_2 = Uuid::new_v4();

    let two_user_response = serde_json::json!({
        "userAccessPolicies": [
            {
                "organizationUserId": user_id_1,
                "organizationUserName": "User One",
                "currentUser": false,
                "read": true,
                "write": false,
                "manage": false,
                "id": Uuid::new_v4(),
                "creationDate": "2024-01-01T00:00:00Z",
                "revisionDate": "2024-01-01T00:00:00Z"
            },
            {
                "organizationUserId": user_id_2,
                "organizationUserName": "User Two",
                "currentUser": false,
                "read": true,
                "write": true,
                "manage": false,
                "id": Uuid::new_v4(),
                "creationDate": "2024-01-01T00:00:00Z",
                "revisionDate": "2024-01-01T00:00:00Z"
            }
        ],
        "groupAccessPolicies": []
    });

    let (_server, client) = helpers::authenticated_client(vec![
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
            .respond_with(ResponseTemplate::new(200).set_body_json(two_user_response)),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/projects/{project_id}/access-policies/service-accounts"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(helpers::empty_sa_response())),
    ])
    .await;

    let response = client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![
                AccessPolicyEntry {
                    grantee_id: user_id_1,
                    read: true,
                    write: false,
                    manage: false,
                },
                AccessPolicyEntry {
                    grantee_id: user_id_2,
                    read: true,
                    write: true,
                    manage: false,
                },
            ]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(vec![]),
        })
        .await
        .expect("PUT with two user policies must succeed");

    assert_eq!(
        response.user_access_policies.len(),
        2,
        "re-fetched response must contain 2 user policies after append"
    );

    let ids: Vec<_> = response
        .user_access_policies
        .iter()
        .map(|p| p.organization_user_id)
        .collect();
    assert!(ids.contains(&user_id_1), "user 1 must be present");
    assert!(
        ids.contains(&user_id_2),
        "user 2 must be present after append"
    );
}
