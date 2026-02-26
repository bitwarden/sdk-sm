#[path = "access_policies/granted_policies.rs"]
mod granted_policies;
/// End-to-end tests for SM access policy management via the Rust SDK.
///
/// Tests are split into two categories:
///   1. Mock-based tests: use wiremock to simulate the Bitwarden API. Run unconditionally.
///   2. Integration tests: require a real server. Marked `#[ignore]` and gated by
///      `SM_INTEGRATION_TESTS=1`. Run with `cargo test -p bitwarden --test access_policies --
///      --ignored`.
///
/// See tests/access_policies/integration.rs for required env vars and manual setup steps.
#[path = "access_policies/helpers.rs"]
mod helpers;
#[path = "access_policies/integration.rs"]
mod integration;
#[path = "access_policies/potential_grantees.rs"]
mod potential_grantees;
#[path = "access_policies/project_policies.rs"]
mod project_policies;
#[path = "access_policies/secret_policies.rs"]
mod secret_policies;
#[path = "access_policies/serialization.rs"]
mod serialization;
