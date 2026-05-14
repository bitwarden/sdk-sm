# value_changed Field - Update Locations

This document tracks all places where `value_changed` needs to be calculated when updating a secret.

## Overview

The `value_changed` field in `SecretPutRequest` should be `true` if the secret's value has been modified, `false` otherwise.

**Calculation Logic:**
- If user provides a new value: `value_changed = true`
- If value remains unchanged: `value_changed = false`

---

## Language Bindings & Access Points

### 1. **CLI (bws)** ✅ COMPLETE
- **File:** `crates/bws/src/command/secret.rs:186`
- **Status:** ✅ Implemented
- **Logic:** `value_changed: secret.value.is_some()`
- **Details:** Checks if user provided a new value in the edit command (Optional field)

### 2. **Python** 🟡 PLACEHOLDER
- **File:** `languages/python/bitwarden_sdk/bitwarden_client.py:256-260`
- **Status:** 🟡 Needs Implementation
- **Current:** `value_changed = False` (placeholder)
- **TODO:** Either:
  - Fetch old secret first and compare values, OR
  - Add `value_changed` parameter to `update()` method signature

### 3. **Go** 🟡 PLACEHOLDER
- **File:** `languages/go/secrets.go:99-110`
- **Status:** 🟡 Needs Implementation
- **Current:** `ValueChanged: false` (placeholder)
- **TODO:** Either:
  - Fetch old secret first and compare values, OR
  - Add `valueChanged` parameter to `Update()` method signature

### 4. **C#** 🟡 PLACEHOLDER
- **File:** `languages/csharp/Bitwarden.Sdk/SecretsClient.cs:65-81`
- **Status:** 🟡 Needs Implementation
- **Current:** `ValueChanged = false` (placeholder)
- **TODO:** Either:
  - Fetch old secret first and compare values, OR
  - Add `valueChanged` parameter to `UpdateAsync()` method signature

### 5. **Java** 🔴 NEEDS COMMENT
- **File:** `languages/java/src/main/java/com/bitwarden/sdk/SecretsClient.java`
- **Status:** 🔴 Needs Annotation
- **TODO:** Add TODO comment for `valueChanged` calculation

### 6. **PHP** 🔴 NEEDS COMMENT
- **File:** `languages/php/src/SecretsClient.php`
- **Status:** 🔴 Needs Annotation
- **TODO:** Add TODO comment for `valueChanged` calculation

### 7. **Ruby** 🔴 NEEDS COMMENT
- **File:** `languages/ruby/bitwarden_sdk_secrets/lib/secrets.rb`
- **Status:** 🔴 Needs Annotation
- **TODO:** Add TODO comment for `valueChanged` calculation

### 8. **TypeScript/Node** 🔴 NEEDS COMMENT
- **File:** `languages/typescript/src/SecretsClient.ts` (or similar)
- **Status:** 🔴 Needs Annotation
- **TODO:** Add TODO comment for `valueChanged` calculation

### 9. **C++** 🔴 NEEDS COMMENT
- **File:** `languages/cpp/src/Secrets.cpp`
- **Status:** 🔴 Needs Annotation
- **TODO:** Add TODO comment for `valueChanged` calculation

---

## Two Implementation Approaches

### Approach A: Fetch Old Secret (Extra API Call)
```pseudocode
oldSecret = client.get(id)
valueChanged = (newValue != oldSecret.value)
client.update(SecretPutRequest(..., valueChanged))
```
**Pros:** Accurate comparison
**Cons:** Double API calls, race conditions possible

### Approach B: Add Parameter to Method
```pseudocode
client.update(id, key, value, note, projectIds, valueChanged=true)
// User/caller is responsible for calculating it
```
**Pros:** Single API call, no extra overhead
**Cons:** Caller must know to calculate it correctly

---

## Recommended Approach

**Use Approach B** (parameter) because:
1. Avoids extra API calls
2. Avoids race conditions
3. Gives caller control
4. Matches CLI design pattern

### Implementation Steps per Language:
1. Add optional `value_changed` parameter to `update()` method
2. Default to `false` if not provided
3. Update docstring to explain the parameter
4. Document that caller should calculate it based on whether value changed

---

## Next Steps

1. ✅ Add field to SDK core (`sdk-internal/bitwarden-sm/src/secrets/update.rs`)
2. ✅ Add to CLI (`crates/bws/src/command/secret.rs`)
3. ⏳ Run `npm run schemas` to regenerate bindings
4. 🔴 Update each language binding with proper calculation or parameter

