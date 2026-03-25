# Blocked Test Cases

Test cases that cannot be fully implemented or activated yet, and the reasons why.

---

## Seeder-dependent tests (no SDK `put_secret_access_policies`)

**Affected file:** `put_secret.rs` (in sdk-internal)

**Symptom:** `put_secret_access_policies` in `put_secret.rs` returns `NotImplemented`. There is
no way to seed secret access policies via the SDK in integration tests.

The server endpoint (`PUT /api/secrets/{id}/access-policies`) is fully implemented in the
server's `AccessPoliciesController`, but the OpenAPI spec in `bitwarden-api-api` has not been
regenerated to include this endpoint. The `AccessPoliciesApi` trait is missing a
`put_secret_access_policies` method.

**Blocked tests:**

| Test | Reason |
|------|--------|
| T10 PUT path in `integration.rs` | SDK returns `NotImplemented` before making any HTTP call |
| Any secret policy mutation integration test | Same root cause |

**How to unblock:** Regenerate the OpenAPI spec for `bitwarden-api-api` from the current server,
then replace the stub in `put_secret.rs` with a real API call.

---

## Delegation constraint test (T7) requires manual fixture

**Affected file:** `integration.rs` (in sdk-sm)

**Symptom:** T7 requires a service account that has Manage permission but did NOT create the
project (`CreatedByServiceAccountId` != SA ID). This cannot be set up programmatically through
the current SDK; it requires manual web UI setup or a seeder script.

The server-side authorization logic (`CreatedByServiceAccountId`, `CreateProjectCommand`, and
the authorization handlers) is fully implemented with unit tests. Only the programmatic test
fixture setup is missing.

**Blocked test:** `t7_non_creator_sa_cannot_grant_manage` in `integration.rs` (optional; skips
gracefully if `SA_MANAGE_NON_CREATOR_TOKEN` is not set).

**How to unblock:** Add a seeder scene or API endpoint that allows assigning
`CreatedByServiceAccountId` independently of who calls project creation.

---
