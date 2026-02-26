//! T4 / T6 / T7 / T8 / T10 — Integration tests against a real Bitwarden server.
//!
//! ## Running integration tests
//!
//! ```bash
//! # Set required env vars (see list below), then:
//! SM_INTEGRATION_TESTS=1 cargo test -p bitwarden --test access_policies -- --ignored
//!
//! # Run a specific test:
//! SM_INTEGRATION_TESTS=1 cargo test -p bitwarden --test access_policies \
//!   integration::t4_sa_write_only_cannot_put_project_policies -- --ignored --nocapture
//! ```
//!
//! ## Required manual setup (via Bitwarden web UI)
//!
//! Before running these tests, prepare the following in your test organization:
//!
//! 1. **Organization**: SM-enabled org. Note the org ID → `SM_ORG_ID`.
//!
//! 2. **Service accounts** — create 3 (optionally 4):
//!    - `SA_READ_ONLY`: has Read-only on `PROJECT_ID_WRITE_ONLY`.
//!    - `SA_WRITE_ONLY`: has Read+Write (but NOT Manage) on `PROJECT_ID_WRITE_ONLY`.
//!    - `SA_MANAGE`: has Manage on `PROJECT_ID_MANAGE`, `PROJECT_ID_LOCKOUT`, `SECRET_ID_MANAGE`.
//!      **SA_MANAGE must be the creator of `PROJECT_ID_MANAGE`** (create via SDK or ensure
//!      `CreatedByServiceAccountId` is set to this SA).
//!    - `SA_MANAGE_NON_CREATOR` (optional): has Manage on a project it did NOT create. Used for
//!      delegation constraint tests T7.
//!
//! 3. **Projects**:
//!    - `PROJECT_ID_MANAGE`: SA_MANAGE has Manage and is creator. At least one human user has
//!      Manage.
//!    - `PROJECT_ID_WRITE_ONLY`: SA_WRITE_ONLY has Write. No Manage for any SA.
//!    - `PROJECT_ID_LOCKOUT`: exactly one human user has Manage. SA_MANAGE has Manage. Used for
//!      lockout protection test T6.
//!
//! 4. **Secret**: `SECRET_ID_MANAGE` belongs to a project where SA_MANAGE has Manage.
//!
//! 5. **Grantee identifiers**:
//!    - `GRANTEE_USER_ID`: org-user ID of a test user in the org.
//!    - `GRANTEE_GROUP_ID` (optional): group ID of a test group.
//!    - `GRANTEE_SA_ID`: service account ID of a second SA (used for granted-policies tests).
//!
//! ## Environment variables
//!
//! | Variable | Required | Description |
//! |---|---|---|
//! | `SM_INTEGRATION_TESTS` | yes | Set to `1` to enable integration tests |
//! | `SM_BASE_URL` | no | Base URL (default: `http://localhost:8080`) |
//! | `SM_ORG_ID` | yes | Organization UUID |
//! | `PROJECT_ID_MANAGE` | yes | Project where SA_MANAGE has Manage (is creator) |
//! | `PROJECT_ID_WRITE_ONLY` | yes | Project where SA_WRITE_ONLY has Write, not Manage |
//! | `PROJECT_ID_LOCKOUT` | yes | Project with exactly one human Manage grantee |
//! | `SECRET_ID_MANAGE` | yes | Secret where SA_MANAGE has Manage |
//! | `SA_READ_ONLY_TOKEN` | yes | SA with Read-only permission |
//! | `SA_WRITE_ONLY_TOKEN` | yes | SA with Write (not Manage) permission |
//! | `SA_MANAGE_TOKEN` | yes | SA with Manage permission (is project creator) |
//! | `SA_MANAGE_NON_CREATOR_TOKEN` | no | SA with Manage but not creator (for T7) |
//! | `GRANTEE_USER_ID` | yes | Org-user UUID for grantee tests |
//! | `GRANTEE_GROUP_ID` | no | Group UUID for grantee tests |
//! | `GRANTEE_SA_ID` | yes | Second SA UUID for granted-policies tests |
//!
//! ## Cleanup
//!
//! Tests that mutate policies restore them after completion by re-applying the original state.
//! If a test panics mid-way, manual cleanup of policies may be needed.

use bitwarden::secrets_manager::{
    AccessPoliciesClientExt,
    access_policies::{
        AccessPolicyEntry, GetProjectAccessPoliciesRequest, PutProjectAccessPoliciesRequest,
    },
    projects::{ProjectCreateRequest, ProjectsDeleteRequest},
};

use crate::helpers::{integration_config, real_server_client};

// ──────────────────────────────────────────────────────────────────────────────
// T4: Permission gate — SA with Write but not Manage is denied policy access
// ──────────────────────────────────────────────────────────────────────────────

/// T4: SA with Write permission is denied GET of project access policies.
///
/// The server must return 404 (resource enumeration prevention) or 403 when the SA
/// does not have Manage. The SDK must surface this as an error.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t4_sa_write_only_cannot_get_project_policies() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_write_only_token).await;

    let result = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest {
            project_id: config.project_id_write_only,
        })
        .await;

    assert!(
        result.is_err(),
        "SA with Write-only permission must be denied GET of project policies. \
         Server should return 404 (to avoid resource enumeration) or 403."
    );
    let err_str = result.unwrap_err().to_string().to_lowercase();
    assert!(
        err_str.contains("403")
            || err_str.contains("404")
            || err_str.contains("not found")
            || err_str.contains("forbidden"),
        "denial error must indicate 403 or 404, got: {err_str}"
    );
}

/// T4b: SA with Write permission is denied PUT of project access policies.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t4b_sa_write_only_cannot_put_project_policies() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_write_only_token).await;

    let result = client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id: config.project_id_write_only,
            user_access_policies: Some(vec![]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(vec![]),
        })
        .await;

    assert!(
        result.is_err(),
        "SA with Write-only permission must be denied PUT of project policies"
    );
    let err_str = result.unwrap_err().to_string().to_lowercase();
    assert!(
        err_str.contains("403")
            || err_str.contains("404")
            || err_str.contains("not found")
            || err_str.contains("forbidden"),
        "denial error must indicate 403 or 404, got: {err_str}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T4c: Permission gate — SA with Read-only is denied policy access
// ──────────────────────────────────────────────────────────────────────────────

/// T4c: SA with Read-only is denied GET of project access policies.
///
/// Read-only grants the SA access to read secrets, not to manage policies.
/// The server must return 404 or 403.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t4c_sa_read_only_cannot_get_project_policies() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_read_only_token).await;

    let result = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest {
            project_id: config.project_id_write_only,
        })
        .await;

    assert!(
        result.is_err(),
        "SA with Read-only permission must be denied GET of project policies"
    );
    let err_str = result.unwrap_err().to_string().to_lowercase();
    assert!(
        err_str.contains("403")
            || err_str.contains("404")
            || err_str.contains("not found")
            || err_str.contains("forbidden"),
        "denial error must indicate 403 or 404, got: {err_str}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T5 (integration): SA with Manage can GET/PUT project policies on a real server
// ──────────────────────────────────────────────────────────────────────────────

/// T5 (integration): SA with Manage can GET project access policies from real server.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t5_integration_sa_manage_can_get_project_policies() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_manage_token).await;

    let response = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest {
            project_id: config.project_id_manage,
        })
        .await
        .expect("SA with Manage must be able to GET project policies");

    // SA_MANAGE should appear in service_account_access_policies with manage: true
    let sa_policy = response
        .service_account_access_policies
        .iter()
        .find(|p| p.policy.manage);

    assert!(
        sa_policy.is_some(),
        "at least one SA with manage:true must be in the policy list"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T6: Lockout protection — cannot remove last human Manage grantee
// ──────────────────────────────────────────────────────────────────────────────

/// T6: Server rejects PUT that would remove the last human Manage grantee.
///
/// Precondition: `PROJECT_ID_LOCKOUT` has exactly one human user with Manage.
/// The test attempts to clear all user policies, which must be rejected with HTTP 400.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t6_lockout_protection_prevents_removing_last_human_manage() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_manage_token).await;

    // GET current state so we can restore it if needed
    let _current = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest {
            project_id: config.project_id_lockout,
        })
        .await
        .expect("SA_MANAGE must be able to GET policies on PROJECT_ID_LOCKOUT");

    // Attempt to clear all human policies (users + groups)
    let result = client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id: config.project_id_lockout,
            user_access_policies: Some(vec![]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: None,
        })
        .await;

    let err = result.expect_err(
        "removing the last human Manage grantee must be rejected by the server (HTTP 400)",
    );
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("400") || err_msg.to_lowercase().contains("manage"),
        "lockout error must indicate HTTP 400 or mention 'manage', got: {err_msg}"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T7: Delegation constraint — non-creator SA cannot grant Manage
// ──────────────────────────────────────────────────────────────────────────────

/// T7: SA with human-granted Manage cannot grant Manage to another principal.
///
/// SKIP REASON if `SA_MANAGE_NON_CREATOR_TOKEN` is not set: the fixture requires
/// a SA that has Manage but did NOT create the project (CreatedByServiceAccountId != SA ID).
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1, SA_MANAGE_NON_CREATOR_TOKEN, and a configured server"]
async fn t7_non_creator_sa_cannot_grant_manage() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let Some(non_creator_token) = &config.sa_manage_non_creator_token else {
        println!(
            "SKIPPED: SA_MANAGE_NON_CREATOR_TOKEN not set. \
             Set up a SA with human-granted Manage on a project it did not create."
        );
        return;
    };

    let client = real_server_client(&config, non_creator_token).await;

    // Attempt to grant Manage to another user
    let result = client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id: config.project_id_manage,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: config.grantee_user_id,
                read: true,
                write: true,
                manage: true, // attempting to delegate Manage
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: None,
        })
        .await;

    assert!(
        result.is_err(),
        "SA with human-granted Manage must NOT be able to grant Manage to others. \
         The server enforces this via CreatedByServiceAccountId check."
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// T8: Delegation constraint — creator SA CAN grant Manage
// ──────────────────────────────────────────────────────────────────────────────

/// T8: SA that created the project can grant Manage to another principal.
///
/// Creates a fresh project via SA_MANAGE (so CreatedByServiceAccountId is set),
/// grants Manage to GRANTEE_USER_ID, verifies success, then deletes the project.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and SA_MANAGE to be project creator"]
async fn t8_creator_sa_can_grant_manage() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_manage_token).await;

    // Create a project owned by SA_MANAGE so CreatedByServiceAccountId is set
    let project = client
        .projects()
        .create(&ProjectCreateRequest {
            organization_id: config.org_id,
            name: "t8-creator-sa-test".into(),
        })
        .await
        .expect("SA_MANAGE must be able to create a project");
    let project_id = project.id;

    // Grant Manage to GRANTEE_USER_ID — should succeed because SA_MANAGE is creator.
    // SA policies are None so the SA PUT is skipped, preserving the SA's own access.
    let result = client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: config.grantee_user_id,
                read: true,
                write: true,
                manage: true,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: None,
        })
        .await;

    assert!(
        result.is_ok(),
        "SA_MANAGE (project creator) must be able to grant Manage. Error: {:?}",
        result.err()
    );

    // Verify grantee now has Manage
    let updated = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("must be able to GET after successful PUT");

    let granted = updated
        .user_access_policies
        .iter()
        .find(|p| p.organization_user_id == config.grantee_user_id);

    assert!(
        granted.is_some(),
        "grantee must appear in policies after PUT"
    );
    assert!(
        granted.unwrap().policy.manage,
        "grantee must have manage:true after being granted Manage by project creator"
    );
    assert!(
        granted.unwrap().policy.read,
        "grantee must have read:true after PUT with read:true"
    );

    // Cleanup: delete the project
    client
        .projects()
        .delete(ProjectsDeleteRequest {
            ids: vec![project_id],
        })
        .await
        .expect("cleanup: delete test project");
}

// ──────────────────────────────────────────────────────────────────────────────
// T10: SA with Write cannot modify secret access policies (real server)
// ──────────────────────────────────────────────────────────────────────────────

/// T10: SA with Write-only on a secret cannot GET or modify secret access policies.
///
/// SKIP REASON: `put_secret_access_policies` is currently `NotImplemented` in the SDK.
/// This test verifies the GET denial and documents the PUT skip.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t10_write_only_sa_cannot_get_secret_policies() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_write_only_token).await;

    let result = client
        .access_policies()
        .get_secret_policies(
            &bitwarden::secrets_manager::access_policies::GetSecretAccessPoliciesRequest {
                secret_id: config.secret_id_manage,
            },
        )
        .await;

    assert!(
        result.is_err(),
        "SA with Write-only permission must not be able to GET secret access policies"
    );

    // PUT is currently NotImplemented in the SDK — document the skip
    println!(
        "SKIP (T10 PUT): put_secret_access_policies is NotImplemented in the SDK. \
         Test will be enabled once the endpoint is implemented."
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Manage round-trip on real server
// ──────────────────────────────────────────────────────────────────────────────

/// T_RT: GET-PUT round-trip preserves `manage: true` on the real server.
///
/// Creates a fresh project via SA_MANAGE, seeds a user with manage:true, then
/// verifies that a GET-PUT round-trip preserves the value. Deletes the project after.
///
/// This catches bugs where:
///   - BUG-1: GET response hardcodes `manage: false` even when the server returns `true`.
///   - BUG-2: PUT request omits or drops the `manage` field, causing an implicit downgrade of
///     manage from true to false.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t_rt_manage_true_survives_get_put_round_trip() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_manage_token).await;

    // Create a project owned by SA_MANAGE so CreatedByServiceAccountId is set
    let project = client
        .projects()
        .create(&ProjectCreateRequest {
            organization_id: config.org_id,
            name: "t_rt-round-trip-test".into(),
        })
        .await
        .expect("SA_MANAGE must be able to create a project");
    let project_id = project.id;

    // Seed a user with manage:true so we have something to round-trip.
    // SA policies are None so the SA PUT is skipped, preserving the SA's own access.
    client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: config.grantee_user_id,
                read: true,
                write: true,
                manage: true,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: None,
        })
        .await
        .expect("seed PUT must succeed");

    // Step 1: GET current policies
    let before = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("initial GET must succeed");

    // Step 2: PUT the same policies back unchanged
    let user_policies: Vec<AccessPolicyEntry> = before
        .user_access_policies
        .iter()
        .map(|p| AccessPolicyEntry {
            grantee_id: p.organization_user_id,
            read: p.policy.read,
            write: p.policy.write,
            manage: p.policy.manage,
        })
        .collect();

    let sa_policies: Vec<AccessPolicyEntry> = before
        .service_account_access_policies
        .iter()
        .map(|p| AccessPolicyEntry {
            grantee_id: p.service_account_id,
            read: p.policy.read,
            write: p.policy.write,
            manage: p.policy.manage,
        })
        .collect();

    client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(user_policies),
            group_access_policies: Some(vec![]),
            service_account_access_policies: Some(sa_policies),
        })
        .await
        .expect("PUT with same data must succeed");

    // Step 3: GET again and compare
    let after = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("post-PUT GET must succeed");

    // Compare sets by ID (order-independent)
    let before_user_ids: std::collections::HashSet<_> = before
        .user_access_policies
        .iter()
        .map(|p| p.organization_user_id)
        .collect();
    let after_user_ids: std::collections::HashSet<_> = after
        .user_access_policies
        .iter()
        .map(|p| p.organization_user_id)
        .collect();
    assert_eq!(
        before_user_ids, after_user_ids,
        "user policy set must be identical after round-trip"
    );

    // Verify no manage value was silently changed
    for before_policy in &before.user_access_policies {
        let after_policy = after
            .user_access_policies
            .iter()
            .find(|p| p.organization_user_id == before_policy.organization_user_id)
            .expect("user must still be present after round-trip");

        assert_eq!(
            before_policy.policy.manage, after_policy.policy.manage,
            "manage value for user {} must not change after GET-PUT round-trip. \
             BUG-1: manage is hardcoded false on GET; BUG-2: manage is dropped on PUT.",
            before_policy.organization_user_id
        );
    }

    // Cleanup: delete the project
    client
        .projects()
        .delete(ProjectsDeleteRequest {
            ids: vec![project_id],
        })
        .await
        .expect("cleanup: delete test project");
}

// ──────────────────────────────────────────────────────────────────────────────
// T_SELECTIVE: Selective PUT — updating people with SA `None` preserves SA policies
// ──────────────────────────────────────────────────────────────────────────────

/// T_SELECTIVE: PUT with `service_account_access_policies: None` does not remove SA policies.
///
/// Creates a project via SA_MANAGE, verifies the SA has Manage, then PUTs only people
/// policies (SA = None). A subsequent GET must show SA policies unchanged.
#[tokio::test]
#[ignore = "requires SM_INTEGRATION_TESTS=1 and a configured real Bitwarden server"]
async fn t_selective_put_people_only_preserves_sa_policies() {
    let Some(config) = integration_config() else {
        println!("SKIPPED: set SM_INTEGRATION_TESTS=1 to run integration tests");
        return;
    };

    let client = real_server_client(&config, &config.sa_manage_token).await;

    // Create a project owned by SA_MANAGE
    let project = client
        .projects()
        .create(&ProjectCreateRequest {
            organization_id: config.org_id,
            name: "t_selective-sa-none-test".into(),
        })
        .await
        .expect("SA_MANAGE must be able to create a project");
    let project_id = project.id;

    // GET initial state — SA_MANAGE should have Manage
    let before = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("must be able to GET policies on newly created project");

    assert!(
        !before.service_account_access_policies.is_empty(),
        "SA_MANAGE must have an SA policy on the project it created"
    );

    // PUT only people policies, leave SA policies untouched (None)
    client
        .access_policies()
        .put_project_policies(&PutProjectAccessPoliciesRequest {
            project_id,
            user_access_policies: Some(vec![AccessPolicyEntry {
                grantee_id: config.grantee_user_id,
                read: true,
                write: true,
                manage: false,
            }]),
            group_access_policies: Some(vec![]),
            service_account_access_policies: None, // must NOT remove SA policies
        })
        .await
        .expect("selective PUT (people only) must succeed");

    // GET again — SA policies must be unchanged
    let after = client
        .access_policies()
        .get_project_policies(&GetProjectAccessPoliciesRequest { project_id })
        .await
        .expect("must be able to GET after selective PUT");

    assert_eq!(
        before.service_account_access_policies.len(),
        after.service_account_access_policies.len(),
        "SA policy count must be unchanged after PUT with SA: None"
    );

    for before_sa in &before.service_account_access_policies {
        let after_sa = after
            .service_account_access_policies
            .iter()
            .find(|p| p.service_account_id == before_sa.service_account_id)
            .expect("each SA must still be present after selective PUT");

        assert_eq!(
            before_sa.policy.manage, after_sa.policy.manage,
            "SA manage value must be unchanged after PUT with SA: None"
        );
    }

    // Verify the user policy WAS applied
    assert!(
        after
            .user_access_policies
            .iter()
            .any(|p| p.organization_user_id == config.grantee_user_id),
        "grantee user must be present after people PUT"
    );

    // Cleanup: delete the project
    client
        .projects()
        .delete(ProjectsDeleteRequest {
            ids: vec![project_id],
        })
        .await
        .expect("cleanup: delete test project");
}
