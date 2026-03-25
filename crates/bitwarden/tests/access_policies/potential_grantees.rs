//! T12 — Potential grantees GET tests using wiremock.
//!
//! Verifies that the SDK correctly calls the three potential-grantee discovery endpoints
//! (People, Projects, ServiceAccounts) and returns the correct data shapes.

use bitwarden::secrets_manager::{
    AccessPoliciesClientExt,
    access_policies::{GetPotentialGranteesRequest, GranteeType},
};
use uuid::Uuid;
use wiremock::{Mock, ResponseTemplate, matchers};

use crate::helpers;

/// T12a: GET people potential grantees returns users and groups.
///
/// Uses `GranteeType::People` to fetch users and groups that can be granted access to projects.
#[tokio::test]
async fn t12a_get_people_potential_grantees() {
    let org_id: uuid::Uuid = helpers::TEST_ORG_ID.parse().unwrap();
    let user_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/people/potential-grantees"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                helpers::people_potential_grantees_response(user_id, group_id),
            )),
    ])
    .await;

    let response = client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::People,
        })
        .await
        .expect("get_potential_grantees(People) must succeed");

    assert_eq!(response.data.len(), 2, "must return user and group");

    let ids: Vec<_> = response.data.iter().map(|g| g.id).collect();
    assert!(ids.contains(&user_id), "user must be present");
    assert!(ids.contains(&group_id), "group must be present");

    let user = response.data.iter().find(|g| g.id == user_id).unwrap();
    assert_eq!(user.name, Some("Admin User".into()));
    assert_eq!(user.r#type, Some("user".into()));
    assert_eq!(user.email, Some("admin@example.com".into()));

    let group = response.data.iter().find(|g| g.id == group_id).unwrap();
    assert_eq!(group.name, Some("Developers".into()));
    assert_eq!(group.r#type, Some("group".into()));
}

/// T12b: GET service account potential grantees returns service accounts.
#[tokio::test]
async fn t12b_get_sa_potential_grantees() {
    let org_id: uuid::Uuid = helpers::TEST_ORG_ID.parse().unwrap();
    let sa_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/service-accounts/potential-grantees"
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(helpers::sa_potential_grantees_response(sa_id)),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::ServiceAccounts,
        })
        .await
        .expect("get_potential_grantees(ServiceAccounts) must succeed");

    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].id, sa_id);
    assert_eq!(response.data[0].name, Some("TEST".into()), "SA name must be decrypted");
    assert_eq!(response.data[0].r#type, Some("serviceAccount".into()));
}

/// T12c: GET project potential grantees returns projects.
///
/// Used to discover which projects can be assigned to a service account via granted policies.
#[tokio::test]
async fn t12c_get_project_potential_grantees() {
    let org_id: uuid::Uuid = helpers::TEST_ORG_ID.parse().unwrap();
    let project_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/projects/potential-grantees"
            )))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(helpers::project_potential_grantees_response(project_id)),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::Projects,
        })
        .await
        .expect("get_potential_grantees(Projects) must succeed");

    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].id, project_id);
    assert_eq!(response.data[0].name, Some("TEST".into()), "project name must be decrypted");
}

/// T12d: GET potential grantees returns empty list gracefully.
#[tokio::test]
async fn t12d_get_potential_grantees_empty_returns_empty_list() {
    let org_id: uuid::Uuid = helpers::TEST_ORG_ID.parse().unwrap();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/people/potential-grantees"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })),
            ),
    ])
    .await;

    let response = client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::People,
        })
        .await
        .expect("get_potential_grantees must succeed with empty list");

    assert!(response.data.is_empty());
}

/// T12e: Each grantee type routes to a distinct API endpoint.
///
/// Verifies that the SDK sends requests to different URL paths depending on `GranteeType`.
#[tokio::test]
async fn t12e_grantee_type_routes_to_correct_endpoint() {
    let org_id: uuid::Uuid = helpers::TEST_ORG_ID.parse().unwrap();

    let (server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/people/potential-grantees"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/service-accounts/potential-grantees"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })),
            ),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/projects/potential-grantees"
            )))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })),
            ),
    ])
    .await;

    client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::People,
        })
        .await
        .expect("People grantees must succeed");

    client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::ServiceAccounts,
        })
        .await
        .expect("ServiceAccounts grantees must succeed");

    client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::Projects,
        })
        .await
        .expect("Projects grantees must succeed");

    let all_requests = server.received_requests().await.unwrap();
    // Exclude POST /identity/connect/token (login) from the count
    let get_requests: Vec<_> = all_requests
        .iter()
        .filter(|r| r.method.as_str() == "GET")
        .collect();

    assert_eq!(
        get_requests.len(),
        3,
        "must send exactly 3 GET requests, one per GranteeType"
    );

    let paths: Vec<&str> = get_requests.iter().map(|r| r.url.path()).collect();
    assert!(
        paths
            .iter()
            .any(|p| p.contains("/people/potential-grantees")),
        "must call people potential-grantees endpoint"
    );
    assert!(
        paths
            .iter()
            .any(|p| p.contains("/service-accounts/potential-grantees")),
        "must call service-accounts potential-grantees endpoint"
    );
    assert!(
        paths
            .iter()
            .any(|p| p.contains("/projects/potential-grantees")),
        "must call projects potential-grantees endpoint"
    );
}

/// TC-PG-03: GET service account potential grantees returns multiple entries.
///
/// Verifies that when the server returns multiple SAs, the SDK correctly surfaces
/// all entries with correct IDs. Used when setting up access policies for secrets.
#[tokio::test]
async fn t12f_get_sa_potential_grantees_multiple_entries() {
    let org_id: uuid::Uuid = helpers::TEST_ORG_ID.parse().unwrap();
    let sa_id_1 = Uuid::new_v4();
    let sa_id_2 = Uuid::new_v4();

    let two_sa_response = serde_json::json!({
        "data": [
            {
                "id": sa_id_1,
                "name": helpers::ENCRYPTED_TEST,
                "type": "serviceAccount",
                "email": null
            },
            {
                "id": sa_id_2,
                "name": helpers::ENCRYPTED_TEST,
                "type": "serviceAccount",
                "email": null
            }
        ]
    });

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id}/access-policies/service-accounts/potential-grantees"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(two_sa_response)),
    ])
    .await;

    let response = client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id,
            grantee_type: GranteeType::ServiceAccounts,
        })
        .await
        .expect("get_potential_grantees(ServiceAccounts) must succeed");

    assert_eq!(response.data.len(), 2, "must return both SA entries");

    let ids: Vec<_> = response.data.iter().map(|g| g.id).collect();
    assert!(ids.contains(&sa_id_1), "SA-Deploy must be present");
    assert!(ids.contains(&sa_id_2), "SA-CI must be present");

    let sa1 = response.data.iter().find(|g| g.id == sa_id_1).unwrap();
    assert_eq!(sa1.name, Some("TEST".into()), "SA name must be decrypted");
    assert_eq!(sa1.r#type, Some("serviceAccount".into()));
}

/// TC-ISO-01: Requests for different organizations return isolated potential grantee sets.
///
/// Verifies that the SDK routes correctly by org_id and responses do not leak across contexts.
/// Uses People grantees because those don't require decryption keys (names are plaintext),
/// allowing us to test routing isolation with a second org that has no loaded crypto keys.
#[tokio::test]
async fn t12g_potential_grantees_isolated_per_organization() {
    let org_id_a: uuid::Uuid = helpers::TEST_ORG_ID.parse().unwrap();
    let org_id_b = Uuid::new_v4();
    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();

    let (server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id_a}/access-policies/people/potential-grantees"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{
                    "id": user_a,
                    "name": "User A",
                    "type": "user",
                    "email": "a@example.com"
                }]
            }))),
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/organizations/{org_id_b}/access-policies/people/potential-grantees"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{
                    "id": user_b,
                    "name": "User B",
                    "type": "user",
                    "email": "b@example.com"
                }]
            }))),
    ])
    .await;

    let response_a = client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id_a,
            grantee_type: GranteeType::People,
        })
        .await
        .expect("get_potential_grantees for org A must succeed");

    let response_b = client
        .access_policies()
        .get_potential_grantees(&GetPotentialGranteesRequest {
            organization_id: org_id_b,
            grantee_type: GranteeType::People,
        })
        .await
        .expect("get_potential_grantees for org B must succeed");

    assert_eq!(response_a.data.len(), 1, "org A must return 1 user");
    assert_eq!(response_b.data.len(), 1, "org B must return 1 user");

    let ids_a: Vec<_> = response_a.data.iter().map(|g| g.id).collect();
    let ids_b: Vec<_> = response_b.data.iter().map(|g| g.id).collect();

    assert!(ids_a.contains(&user_a), "org A must return user_a");
    assert!(!ids_a.contains(&user_b), "org A must not contain user_b");
    assert!(ids_b.contains(&user_b), "org B must return user_b");
    assert!(!ids_b.contains(&user_a), "org B must not contain user_a");

    let all_requests = server.received_requests().await.unwrap();
    let get_requests: Vec<_> = all_requests
        .iter()
        .filter(|r| r.method.as_str() == "GET")
        .collect();
    assert_eq!(
        get_requests.len(),
        2,
        "must send exactly 2 GET requests, one per organization"
    );
}
