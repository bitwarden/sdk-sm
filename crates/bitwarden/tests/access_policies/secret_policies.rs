//! T9 — Secret access policy tests.
//!
//! T9b and T9c verify GET works and that the `manage` field is correctly preserved.
//! T9d verifies that `AccessPolicyEntry` serialization includes `manage`.

use bitwarden::secrets_manager::{
    AccessPoliciesClientExt,
    access_policies::{AccessPolicyEntry, GetSecretAccessPoliciesRequest},
};
use uuid::Uuid;
use wiremock::{Mock, ResponseTemplate, matchers};

use crate::helpers;

/// T9b: GET secret access policies returns the correct structure.
#[tokio::test]
async fn t9b_get_secret_policies_returns_all_policy_types() {
    let secret_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let sa_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/secrets/{secret_id}/access-policies"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                helpers::secret_policies_response_manage_true(user_id, sa_id),
            )),
    ])
    .await;

    let response = client
        .access_policies()
        .get_secret_policies(&GetSecretAccessPoliciesRequest { secret_id })
        .await
        .expect("get_secret_policies must succeed when server returns 200");

    assert_eq!(
        response.user_access_policies.len(),
        1,
        "must return user policy"
    );
    assert_eq!(
        response.user_access_policies[0].organization_user_id,
        user_id
    );

    assert_eq!(
        response.service_account_access_policies.len(),
        1,
        "must return SA policy"
    );
    assert_eq!(
        response.service_account_access_policies[0].service_account_id,
        sa_id
    );

    assert!(
        response.group_access_policies.is_empty(),
        "no groups were configured"
    );
}

/// T9c: GET secret policies — `manage: true` is preserved from the server response.
#[tokio::test]
async fn t9c_get_secret_policies_preserves_manage_true() {
    let secret_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let sa_id = Uuid::new_v4();

    let (_server, client) = helpers::authenticated_client(vec![
        Mock::given(matchers::method("GET"))
            .and(matchers::path(format!(
                "/api/secrets/{secret_id}/access-policies"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                helpers::secret_policies_response_manage_true(user_id, sa_id),
            )),
    ])
    .await;

    let response = client
        .access_policies()
        .get_secret_policies(&GetSecretAccessPoliciesRequest { secret_id })
        .await
        .expect("get_secret_policies must succeed");

    assert!(
        response.user_access_policies[0].policy.manage,
        "user manage:true must be preserved from server response"
    );
    assert!(
        response.service_account_access_policies[0].policy.manage,
        "SA manage:true must be preserved from server response"
    );
}

/// T9d: `AccessPolicyEntry` serialization includes `manage` for secret policies request.
///
/// When PUT is implemented, this verifies the request body will include `manage`.
/// This is a pure serialization test.
#[test]
fn t9d_secret_put_request_entry_serializes_manage() {
    let entry = AccessPolicyEntry {
        grantee_id: Uuid::new_v4(),
        read: true,
        write: true,
        manage: true,
    };
    let json = serde_json::to_string(&entry).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["manage"], serde_json::Value::Bool(true));
}
