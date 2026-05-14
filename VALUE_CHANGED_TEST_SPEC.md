# value_changed Field - Test Specification

## Critical Requirement
The `value_changed` field powers version history and must be **100% accurate**. There are no fallbacks or approximations.

## Test Cases for All Implementations

### Test 1: Value Actually Changed
**Scenario:** User updates secret with a different value
- **Setup:** Secret exists with value="old_value"
- **Action:** Update with value="new_value"
- **Expected:** `value_changed = true`
- **Assertion:** Version history records this as a value change

### Test 2: Value Unchanged
**Scenario:** User updates secret but keeps the same value
- **Setup:** Secret exists with value="same_value"
- **Action:** Update with value="same_value" (no change)
- **Expected:** `value_changed = false`
- **Assertion:** Version history records this as no value change

### Test 3: Other Fields Changed, Value Unchanged
**Scenario:** User updates key/note/projects but NOT the value
- **Setup:** Secret with key="old_key", value="same_value"
- **Action:** Update with key="new_key", value="same_value"
- **Expected:** `value_changed = false`
- **Assertion:** Only tracks that value didn't change (other fields don't affect flag)

### Test 4: Fetch Fails - Update Must Fail
**Scenario:** Old secret cannot be fetched (deleted, permission denied, network error)
- **Setup:** Secret ID exists but user lacks read permission
- **Action:** Attempt update
- **Expected:** Update fails with clear error message
- **Assertion:** Does NOT proceed with fallback value_changed=false
- **Error Message Must Include:** "failed to fetch current value for version history"

### Test 5: Fetch Fails - Different Error Types
**Test each error case:**
- Secret doesn't exist (404)
- Permission denied (403)
- Network timeout
- Server error (500)

**Expected:** All fail with same error handling pattern

### Test 6: Concurrent Modifications
**Scenario:** Another user modifies the secret between fetch and update
- **Setup:** User A fetches secret (value="A")
- **Concurrent:** User B updates same secret (value="B")
- **Action:** User A updates with value="C"
- **Expected:** `value_changed = true` (compares to "A", not "B")
- **Note:** This is acceptable - version history is based on what the user calculated, not race conditions

---

## Per-Language Test Implementation

### CLI (Rust - crates/bws/src/command/secret.rs)
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_edit_value_changed_true() {
        // Old value: "original"
        // New value: "modified"
        // Expected: value_changed = true
    }

    #[tokio::test]
    async fn test_edit_value_changed_false() {
        // Old value: "same"
        // New value: "same"
        // Expected: value_changed = false
    }

    #[tokio::test]
    async fn test_edit_value_changed_with_optional_values() {
        // key: Some("new_key")
        // value: None (keeping old value)
        // Expected: value_changed = false
    }

    #[tokio::test]
    async fn test_edit_fails_if_secret_not_found() {
        // Attempt to edit secret that doesn't exist
        // Expected: Err with message about fetch failure
    }
}
```

### Go (languages/go/secrets.go)
```go
func TestUpdateValueChangedTrue(t *testing.T) {
    // Old value: "original"
    // New value: "modified"
    // Expected: valueChanged = true
}

func TestUpdateValueChangedFalse(t *testing.T) {
    // Old value: "same"
    // New value: "same"
    // Expected: valueChanged = false
}

func TestUpdateFailsIfFetchFails(t *testing.T) {
    // Mock Get() to return error
    // Expected: Update() returns error with "failed to fetch" message
}
```

### Python (languages/python/bitwarden_sdk/bitwarden_client.py)
```python
def test_update_value_changed_true(self):
    # Old value: "original"
    # New value: "modified"
    # Assert: value_changed = True

def test_update_value_changed_false(self):
    # Old value: "same"
    # New value: "same"
    # Assert: value_changed = False

def test_update_fails_if_get_fails(self):
    # Mock get() to raise exception
    # Assert: update() raises with "failed to fetch" message
```

### C#, Java, PHP, Ruby, TypeScript, C++
Similar pattern for each language.

---

## Pre-Commit Checklist

Before committing these changes:

- [ ] All new test cases added
- [ ] All tests pass: `cargo test --all` / `npm test --workspaces`
- [ ] No clippy warnings: `cargo clippy --all-features`
- [ ] Compile checks pass: `cargo build --all`
- [ ] Language bindings compile: `npm run build --workspaces`
- [ ] Error messages are clear and mention "version history"
- [ ] Comments marked as "CRITICAL" present in all implementations
- [ ] No fallback values (no hardcoded `false` as default)

---

## Integration Testing with Fake Server

```bash
# Start fake server
cargo run -p fake-server &

# Test CLI
bws secret update <id> --value "new_value"

# Verify version history records the change
# (If version history API exists, query it)
```

---

## Regression Testing

Ensure existing functionality still works:
- [ ] Creating secrets still works
- [ ] Getting secrets still works
- [ ] Listing secrets still works
- [ ] Deleting secrets still works
- [ ] All error cases still handled properly

---

## Performance Testing

Document the impact:
- **Before:** 1 API call per update
- **After:** 2 API calls per update (fetch + update)
- **Latency Impact:** ~100% increase per update operation
- **Rate Limit Impact:** 2x API calls used

This is acceptable because version history accuracy is critical.

