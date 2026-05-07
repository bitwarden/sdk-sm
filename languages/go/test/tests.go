package main

import (
	"encoding/json"
	"fmt"
	"os"
	"runtime"
	"time"

	sdk "github.com/bitwarden/sdk-go/v2"
	"github.com/gofrs/uuid"
)

// TestOperation represents a single test operation result
type TestOperation struct {
	Operation  string                 `json:"operation"`
	Success    bool                   `json:"success"`
	DurationMs int64                  `json:"duration_ms"`
	Error      *string                `json:"error"`
	Details    map[string]interface{} `json:"details,omitempty"`
}

// TestResult contains all test results
type TestResult struct {
	Language       string           `json:"language"`
	SDKVersion     string           `json:"sdk_version"`
	Operations     []TestOperation  `json:"operations"`
	TotalDurationMs int64           `json:"total_duration_ms"`
	OS             string           `json:"os"`
	Architecture   string           `json:"architecture"`
	Timestamp      string           `json:"timestamp"`
}

// GoSDKTester runs Go SDK tests
type GoSDKTester struct {
	client         *sdk.BitwardenClient
	organizationID string
	stateFile      string
	testMode       string
	operations     []TestOperation
	createdSecretID string
}

// NewGoSDKTester creates a new tester instance
func NewGoSDKTester() *GoSDKTester {
	return &GoSDKTester{
		organizationID: os.Getenv("ORGANIZATION_ID"),
		stateFile:     os.Getenv("STATE_FILE"),
		testMode:      getEnvOrDefault("TEST_MODE", "fake-server"),
		operations:    make([]TestOperation, 0),
	}
}

// SetupClient initializes the Bitwarden client
func (t *GoSDKTester) SetupClient() error {
	apiURL := getEnvOrDefault("API_URL", "http://localhost:4000")
	identityURL := getEnvOrDefault("IDENTITY_URL", "http://localhost:33656")

	client, err := sdk.NewBitwardenClient(&apiURL, &identityURL)
	if err != nil {
		return fmt.Errorf("failed to create client: %v", err)
	}

	t.client = client
	return nil
}

// TestAuth tests authentication with access token
func (t *GoSDKTester) TestAuth() TestOperation {
	op := TestOperation{Operation: "auth"}
	start := time.Now()

	accessToken := os.Getenv("ACCESS_TOKEN")
	if accessToken == "" {
		err := "ACCESS_TOKEN not set"
		op.Error = &err
	} else {
		err := t.client.AccessTokenLogin(accessToken, &t.stateFile)
		if err != nil {
			errStr := err.Error()
			op.Error = &errStr
		} else {
			op.Success = true
			op.Details = map[string]interface{}{
				"method": "access_token",
			}
		}
	}

	op.DurationMs = time.Since(start).Milliseconds()
	return op
}

// TestCreateSecret tests creating a new secret
func (t *GoSDKTester) TestCreateSecret() TestOperation {
	op := TestOperation{Operation: "create_secret"}
	start := time.Now()

	secretName := fmt.Sprintf("test-secret-%s", uuid.Must(uuid.NewV4()).String()[:8])
	secret, err := t.client.Secrets().Create(
		secretName,
		"test-value",
		"Created by Go SDK test",
		t.organizationID,
		[]string{},
	)

	if err != nil {
		errStr := err.Error()
		op.Error = &errStr
	} else {
		op.Success = true
		t.createdSecretID = secret.ID
		op.Details = map[string]interface{}{
			"secret_id":  secret.ID,
			"secret_key": secret.Key,
		}
	}

	op.DurationMs = time.Since(start).Milliseconds()
	return op
}

// TestListSecrets tests listing secrets
func (t *GoSDKTester) TestListSecrets() TestOperation {
	op := TestOperation{Operation: "list_secrets"}
	start := time.Now()

	secretsList, err := t.client.Secrets().List(t.organizationID)
	if err != nil {
		errStr := err.Error()
		op.Error = &errStr
	} else {
		op.Success = true
		op.Details = map[string]interface{}{
			"count": len(secretsList.Data),
		}
	}

	op.DurationMs = time.Since(start).Milliseconds()
	return op
}

// TestGetSecret tests getting a specific secret
func (t *GoSDKTester) TestGetSecret() TestOperation {
	op := TestOperation{Operation: "get_secret"}
	start := time.Now()

	if t.testMode == "fake-server" {
		// Fake server returns "btw" for any random UUID
		secret, err := t.client.Secrets().Get(uuid.Must(uuid.NewV4()).String())
		if err != nil {
			errStr := err.Error()
			op.Error = &errStr
		} else {
			expectedKey := "btw"
			if secret.Key == expectedKey {
				op.Success = true
			} else {
				errStr := fmt.Sprintf("Unexpected secret key: %s", secret.Key)
				op.Error = &errStr
			}
			op.Details = map[string]interface{}{
				"secret_key": secret.Key,
			}
		}
	} else {
		// Real server - get the created secret
		if t.createdSecretID == "" {
			errStr := "No secret created to retrieve"
			op.Error = &errStr
		} else {
			secret, err := t.client.Secrets().Get(t.createdSecretID)
			if err != nil {
				errStr := err.Error()
				op.Error = &errStr
			} else {
				op.Success = secret.ID == t.createdSecretID
				op.Details = map[string]interface{}{
					"secret_key": secret.Key,
				}
			}
		}
	}

	op.DurationMs = time.Since(start).Milliseconds()
	return op
}

// TestDeleteSecret tests deleting a secret
func (t *GoSDKTester) TestDeleteSecret() TestOperation {
	op := TestOperation{Operation: "delete_secret"}
	start := time.Now()

	// Use created secret ID if available, otherwise use random UUID
	secretID := t.createdSecretID
	if secretID == "" {
		secretID = uuid.Must(uuid.NewV4()).String()
	}

	_, err := t.client.Secrets().Delete([]string{secretID})
	if err != nil {
		errStr := err.Error()
		op.Error = &errStr
	} else {
		op.Success = true
		op.Details = map[string]interface{}{
			"deleted_id": secretID,
		}
	}

	op.DurationMs = time.Since(start).Milliseconds()
	return op
}

// TestSync tests sync functionality
func (t *GoSDKTester) TestSync() TestOperation {
	op := TestOperation{Operation: "sync"}
	start := time.Now()

	// First sync without date (should have changes)
	syncResponse, err := t.client.Secrets().Sync(t.organizationID, nil)
	if err != nil {
		errStr := err.Error()
		op.Error = &errStr
		op.DurationMs = time.Since(start).Milliseconds()
		return op
	}

	hasChangesInitial := syncResponse.HasChanges

	// Second sync with current date (should have no changes)
	now := time.Now()
	syncResponseWithDate, err := t.client.Secrets().Sync(t.organizationID, &now)
	if err != nil {
		errStr := err.Error()
		op.Error = &errStr
		op.DurationMs = time.Since(start).Milliseconds()
		return op
	}

	hasChangesAfter := syncResponseWithDate.HasChanges

	// Success if initial has changes and subsequent doesn't
	op.Success = hasChangesInitial && !hasChangesAfter
	op.Details = map[string]interface{}{
		"initial_has_changes": hasChangesInitial,
		"after_has_changes":   hasChangesAfter,
	}

	op.DurationMs = time.Since(start).Milliseconds()
	return op
}

// RunAllTests runs all test operations and returns results
func (t *GoSDKTester) RunAllTests() TestResult {
	startTime := time.Now()

	result := TestResult{
		Language:     "go",
		SDKVersion:   "unknown", // TODO: Get actual SDK version
		Operations:   []TestOperation{},
		OS:           runtime.GOOS,
		Architecture: runtime.GOARCH,
		Timestamp:    time.Now().UTC().Format(time.RFC3339),
	}

	// Setup client
	if err := t.SetupClient(); err != nil {
		fmt.Fprintf(os.Stderr, "Failed to setup client: %v\n", err)
		return result
	}

	// Run tests in sequence
	tests := []func() TestOperation{
		t.TestAuth,
		t.TestCreateSecret,
		t.TestListSecrets,
		t.TestGetSecret,
		t.TestDeleteSecret,
		t.TestSync,
	}

	for _, test := range tests {
		op := test()
		t.operations = append(t.operations, op)
		result.Operations = append(result.Operations, op)
	}

	// Clean up client
	if t.client != nil {
		t.client.Close()
	}

	result.TotalDurationMs = time.Since(startTime).Milliseconds()
	return result
}

func getEnvOrDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

func main() {
	tester := NewGoSDKTester()
	results := tester.RunAllTests()

	// Output JSON results
	jsonData, err := json.MarshalIndent(results, "", "  ")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to marshal results: %v\n", err)
		os.Exit(1)
	}

	fmt.Println(string(jsonData))

	// Exit with appropriate code
	allPassed := true
	for _, op := range results.Operations {
		if !op.Success {
			allPassed = false
			break
		}
	}

	if allPassed {
		os.Exit(0)
	} else {
		os.Exit(1)
	}
}