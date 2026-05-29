package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"runtime"
	"runtime/debug"
	"strings"
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
	Language        string          `json:"language"`
	SDKVersion      string          `json:"sdk_version"`
	Operations      []TestOperation `json:"operations"`
	TotalDurationMs int64           `json:"total_duration_ms"`
	OS              string          `json:"os"`
	Architecture    string          `json:"architecture"`
	Timestamp       string          `json:"timestamp"`
}

// GoSDKTestSuite manages test execution
type GoSDKTestSuite struct {
	client         sdk.BitwardenClientInterface
	organizationID string
	jsonOutput     bool
	verbose        bool
	operations     []TestOperation
	startTime      time.Time
	testMode       string
}

// NewTestSuite creates a new test suite instance
func NewTestSuite(jsonOutput, verbose bool) *GoSDKTestSuite {
	return &GoSDKTestSuite{
		jsonOutput:     jsonOutput,
		verbose:        verbose,
		organizationID: os.Getenv("ORGANIZATION_ID"),
		operations:     []TestOperation{},
		testMode:       os.Getenv("TEST_MODE"),
	}
}

// SetupClient initializes the Bitwarden client
func (s *GoSDKTestSuite) SetupClient() error {
	apiURL := os.Getenv("API_URL")
	identityURL := os.Getenv("IDENTITY_URL")

	if apiURL == "" || identityURL == "" {
		return fmt.Errorf("API_URL and IDENTITY_URL environment variables must be set")
	}

	if s.verbose {
		fmt.Fprintf(os.Stderr, "Setting up client with API: %s, Identity: %s\n", apiURL, identityURL)
	}

	client, err := sdk.NewBitwardenClient(&apiURL, &identityURL)
	if err != nil {
		return fmt.Errorf("failed to create client: %w", err)
	}

	s.client = client
	return nil
}

// createTestProject creates a project for testing and returns the project ID
func (s *GoSDKTestSuite) createTestProject(purpose string) (string, error) {
	projectName := fmt.Sprintf("test-project-%s-%s", purpose, uuid.Must(uuid.NewV4()).String()[:8])
	project, err := s.client.Projects().Create(s.organizationID, projectName)
	if err != nil {
		return "", fmt.Errorf("failed to create project: %w", err)
	}
	return project.ID, nil
}

// cleanupProject deletes a project and verifies cleanup on real server
func (s *GoSDKTestSuite) cleanupProject(projectID string) error {
	_, err := s.client.Projects().Delete([]string{projectID})
	if err != nil {
		return fmt.Errorf("failed to delete project: %w", err)
	}

	if s.testMode == "real-server" {
		projectsList, err := s.client.Projects().List(s.organizationID)
		if err != nil {
			return fmt.Errorf("failed to list projects for verification: %w", err)
		}
		for _, p := range projectsList.Data {
			if p.ID == projectID {
				return fmt.Errorf("project %s still exists after deletion", projectID)
			}
		}
	}
	return nil
}

// verifySecretDeleted verifies a secret is deleted on real server
func (s *GoSDKTestSuite) verifySecretDeleted(secretID string) error {
	if s.testMode == "real-server" {
		_, err := s.client.Secrets().Get(secretID)
		if err == nil {
			return fmt.Errorf("secret still exists after deletion")
		}
		if !strings.Contains(strings.ToLower(err.Error()), "not found") {
			return fmt.Errorf("unexpected error when verifying deletion: %w", err)
		}
	}
	return nil
}

// RunOperation executes a test operation and tracks the result
func (s *GoSDKTestSuite) RunOperation(name string, testFunc func() (bool, map[string]interface{}, error), displayName string) bool {
	if displayName == "" {
		displayName = name
	}

	start := time.Now()
	operation := TestOperation{
		Operation: name,
		Success:   false,
	}

	if s.verbose {
		fmt.Fprintf(os.Stderr, "Running: %s\n", displayName)
	}

	success, details, err := testFunc()
	operation.Success = success
	operation.Details = details
	operation.DurationMs = time.Since(start).Milliseconds()

	if err != nil {
		errStr := err.Error()
		operation.Error = &errStr
		if !s.jsonOutput {
			fmt.Printf("❌ %s: %s\n", displayName, err)
		}
	} else if !s.jsonOutput {
		if operation.Success {
			fmt.Printf("✅ %s (%dms)\n", displayName, operation.DurationMs)
		} else {
			fmt.Printf("❌ %s (%dms)\n", displayName, operation.DurationMs)
		}
	}

	s.operations = append(s.operations, operation)
	return operation.Success
}

// ========== Test Operations ==========

// TestAuth tests authentication
func (s *GoSDKTestSuite) TestAuth() (bool, map[string]interface{}, error) {
	accessToken := os.Getenv("ACCESS_TOKEN")
	if accessToken == "" {
		return false, nil, fmt.Errorf("ACCESS_TOKEN not set")
	}

	stateFile := os.Getenv("STATE_FILE")
	var stateFilePtr *string
	if stateFile != "" {
		stateFilePtr = &stateFile
	}
	err := s.client.AccessTokenLogin(accessToken, stateFilePtr)
	if err != nil {
		return false, nil, err
	}

	// Verify authentication worked by trying to sync
	_, err = s.client.Secrets().Sync(s.organizationID, nil)
	if err != nil {
		return false, nil, fmt.Errorf("authentication verification failed: %w", err)
	}

	return true, map[string]interface{}{
		"has_state": stateFile != "",
	}, nil
}

// TestSecretCreate creates a secret
func (s *GoSDKTestSuite) TestSecretCreate() (bool, map[string]interface{}, error) {
	// Create a project for this test
	projectID, err := s.createTestProject("secret-create")
	if err != nil {
		return false, nil, err
	}

	// Create the secret
	secretName := fmt.Sprintf("test-secret-%s", uuid.Must(uuid.NewV4()).String()[:8])
	secret, err := s.client.Secrets().Create(
		secretName,
		"test-value",
		"Created by test suite",
		s.organizationID,
		[]string{projectID},
	)
	if err != nil {
		// Clean up the project on failure
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// Clean up the secret
	s.client.Secrets().Delete([]string{secret.ID})
	if err := s.verifySecretDeleted(secret.ID); err != nil {
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// Clean up the project
	if err := s.cleanupProject(projectID); err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"id":       secret.ID,
		"key":      secret.Key,
		"verified": s.testMode == "real-server",
	}, nil
}

// TestSecretGet gets a secret
func (s *GoSDKTestSuite) TestSecretGet() (bool, map[string]interface{}, error) {
	// Create a project for this test
	projectID, err := s.createTestProject("secret-get")
	if err != nil {
		return false, nil, err
	}

	// Create a secret to get
	secretName := fmt.Sprintf("test-secret-%s", uuid.Must(uuid.NewV4()).String()[:8])
	secret, err := s.client.Secrets().Create(
		secretName,
		"test-value",
		"Created by test suite",
		s.organizationID,
		[]string{projectID},
	)
	if err != nil {
		// Clean up the project on failure
		s.cleanupProject(projectID)
		return false, nil, err
	}
	expectedID := secret.ID

	// Get the secret
	retrievedSecret, err := s.client.Secrets().Get(expectedID)
	if err != nil {
		// Clean up before returning error
		s.client.Secrets().Delete([]string{expectedID})
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// On real server, verify we got the correct secret
	if s.testMode == "real-server" {
		if retrievedSecret.ID != expectedID {
			// Clean up before returning error
			s.client.Secrets().Delete([]string{expectedID})
			s.cleanupProject(projectID)
			return false, nil, fmt.Errorf("got wrong secret: expected %s, got %s", expectedID, retrievedSecret.ID)
		}
	}

	// Clean up: delete secret and project
	s.client.Secrets().Delete([]string{expectedID})
	if err := s.verifySecretDeleted(expectedID); err != nil {
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// Clean up the project
	if err := s.cleanupProject(projectID); err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"id":       retrievedSecret.ID,
		"key":      retrievedSecret.Key,
		"verified": s.testMode == "real-server",
	}, nil
}

// TestSecretUpdate updates a secret
func (s *GoSDKTestSuite) TestSecretUpdate() (bool, map[string]interface{}, error) {
	// Create a project for this test
	projectID, err := s.createTestProject("secret-update")
	if err != nil {
		return false, nil, err
	}

	// Create a secret to update
	secretName := fmt.Sprintf("test-secret-%s", uuid.Must(uuid.NewV4()).String()[:8])
	secret, err := s.client.Secrets().Create(
		secretName,
		"test-value",
		"Created by test suite",
		s.organizationID,
		[]string{projectID},
	)
	if err != nil {
		// Clean up the project on failure
		s.cleanupProject(projectID)
		return false, nil, err
	}
	secretID := secret.ID

	// Update the secret
	updated, err := s.client.Secrets().Update(
		secretID,
		"updated-key",
		"updated-value",
		"Updated by test",
		s.organizationID,
		[]string{projectID},
	)
	if err != nil {
		// Clean up before returning error
		s.client.Secrets().Delete([]string{secretID})
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// Clean up: delete secret and project
	s.client.Secrets().Delete([]string{secretID})
	if err := s.verifySecretDeleted(secretID); err != nil {
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// Clean up the project
	if err := s.cleanupProject(projectID); err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"id":       secretID,
		"key":      updated.Key,
		"verified": s.testMode == "real-server",
	}, nil
}

// TestSecretSync tests sync functionality
func (s *GoSDKTestSuite) TestSecretSync() (bool, map[string]interface{}, error) {
	// Initial sync with nil date - should return all secrets
	syncResponse, err := s.client.Secrets().Sync(s.organizationID, nil)
	if err != nil {
		return false, nil, err
	}

	// Verify initial sync returns data (has_changes should be true for first sync)
	if !syncResponse.HasChanges {
		return false, nil, fmt.Errorf("initial sync should return has_changes=true")
	}

	// Sync with current date - should return no changes (nothing changed since now)
	now := time.Now()
	syncResponseWithDate, err := s.client.Secrets().Sync(s.organizationID, &now)
	if err != nil {
		return false, nil, err
	}

	// For fake-server, the behavior is currently inverted due to implementation
	// For real-server, this should properly return false for no changes
	expectedNoChanges := false
	if s.testMode == "fake-server" {
		// Fake server incorrectly returns false for any past date
		expectedNoChanges = syncResponseWithDate.HasChanges == false
	} else {
		// Real server should return false when no changes since the given date
		expectedNoChanges = syncResponseWithDate.HasChanges == false
	}

	return expectedNoChanges, map[string]interface{}{
		"sync_succeeded":  true,
		"initial_secrets": len(syncResponse.Secrets),
	}, nil
}

// TestSecretDelete deletes secrets
func (s *GoSDKTestSuite) TestSecretDelete() (bool, map[string]interface{}, error) {
	// Create a project for this test
	projectID, err := s.createTestProject("secret-delete")
	if err != nil {
		return false, nil, err
	}

	// Create a secret to delete
	secretName := fmt.Sprintf("test-secret-%s", uuid.Must(uuid.NewV4()).String()[:8])
	secret, err := s.client.Secrets().Create(
		secretName,
		"test-value",
		"Created by test suite",
		s.organizationID,
		[]string{projectID},
	)
	if err != nil {
		// Clean up the project on failure
		s.cleanupProject(projectID)
		return false, nil, err
	}
	secretID := secret.ID

	// Delete the secret
	_, err = s.client.Secrets().Delete([]string{secretID})
	if err != nil {
		// Clean up the project before returning error
		s.cleanupProject(projectID)
		return false, nil, fmt.Errorf("failed to delete secret: %w", err)
	}

	// Verify the secret is actually deleted
	if err := s.verifySecretDeleted(secretID); err != nil {
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// Clean up the project
	if err := s.cleanupProject(projectID); err != nil {
		return false, nil, err
	}

	// If we got here, deletion succeeded and was verified (for real-server)
	return true, map[string]interface{}{
		"deletion_succeeded": true,
	}, nil
}

// TestProjectList lists projects
func (s *GoSDKTestSuite) TestProjectList() (bool, map[string]interface{}, error) {
	// Create a project for this test
	projectID, err := s.createTestProject("project-list")
	if err != nil {
		return false, nil, err
	}

	// List projects
	projectsList, err := s.client.Projects().List(s.organizationID)
	if err != nil {
		// Clean up before returning error
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// On real server, verify our created project is in the list
	verified := false
	if s.testMode == "real-server" {
		projectIDs := make(map[string]bool)
		for _, p := range projectsList.Data {
			projectIDs[p.ID] = true
		}
		if !projectIDs[projectID] {
			// Clean up before returning error
			s.cleanupProject(projectID)
			return false, nil, fmt.Errorf("created project %s not found in list", projectID)
		}
		verified = true
	}

	// Clean up the project
	if err := s.cleanupProject(projectID); err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"count":    len(projectsList.Data),
		"verified": verified,
	}, nil
}

// TestProjectUpdate updates a project
func (s *GoSDKTestSuite) TestProjectUpdate() (bool, map[string]interface{}, error) {
	// Create a project for this test
	projectID, err := s.createTestProject("project-update")
	if err != nil {
		return false, nil, err
	}

	// Update the project
	newName := fmt.Sprintf("updated-project-%s", uuid.Must(uuid.NewV4()).String()[:8])
	updated, err := s.client.Projects().Update(projectID, s.organizationID, newName)
	if err != nil {
		// Clean up before returning error
		s.cleanupProject(projectID)
		return false, nil, err
	}

	// Verify the update worked
	success := strings.Contains(updated.Name, newName)

	// Clean up the project
	if err := s.cleanupProject(projectID); err != nil {
		return false, nil, err
	}

	return success, map[string]interface{}{
		"name":     updated.Name,
		"verified": s.testMode == "real-server",
	}, nil
}

// TestGeneratorDefault tests password generator with default parameters
func (s *GoSDKTestSuite) TestGeneratorDefault() (bool, map[string]interface{}, error) {
	// Define character sets
	const (
		lowercaseChars = "abcdefghijklmnopqrstuvwxyz"
		uppercaseChars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
		numericChars   = "0123456789"
		specialChars   = "!@#$%^&*()-_=+[]{};:'\",.<>/?\\|`~"
	)

	generated := s.client.Generators().Generate(sdk.GenerateRequest{})

	// Should be exactly 24 chars
	if len(generated) != 24 {
		return false, nil, fmt.Errorf("expected length 24, got %d", len(generated))
	}

	// Should contain lowercase chars
	if !containsAnyChar(generated, lowercaseChars) {
		return false, nil, fmt.Errorf("generated secret missing lowercase characters")
	}

	// Should contain uppercase chars
	if !containsAnyChar(generated, uppercaseChars) {
		return false, nil, fmt.Errorf("generated secret missing uppercase characters")
	}

	// Should contain numeric chars
	if !containsAnyChar(generated, numericChars) {
		return false, nil, fmt.Errorf("generated secret missing numeric characters")
	}

	// Should contain special chars
	if !containsAnyChar(generated, specialChars) {
		return false, nil, fmt.Errorf("generated secret missing special characters")
	}

	return true, map[string]interface{}{
		"length":        len(generated),
		"has_all_types": true,
	}, nil
}

// TestGeneratorCustom tests password generator with custom parameters
func (s *GoSDKTestSuite) TestGeneratorCustom() (bool, map[string]interface{}, error) {
	// Define character sets
	const (
		lowercaseChars  = "abcdefghijklmnopqrstuvwxyz"
		uppercaseChars  = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
		numericChars    = "0123456789"
		specialChars    = "!@#$%^&*()-_=+[]{};:'\",.<>/?\\|`~"
		ambiguousChars  = "0O1lI"
	)

	length := uint8(128)
	avoidAmbiguous := false
	lowercase := true
	uppercase := true
	numbers := true
	special := true
	minLowercase := uint8(2)
	minUppercase := uint8(2)
	minNumber := uint8(4)
	minSpecial := uint8(4)

	generated := s.client.Generators().Generate(sdk.GenerateRequest{
		Length:         &length,
		AvoidAmbiguous: &avoidAmbiguous,
		Lowercase:      &lowercase,
		Uppercase:      &uppercase,
		Numbers:        &numbers,
		Special:        &special,
		MinLowercase:   &minLowercase,
		MinUppercase:   &minUppercase,
		MinNumber:      &minNumber,
		MinSpecial:     &minSpecial,
	})

	// Should be exactly 128 chars
	if len(generated) != 128 {
		return false, nil, fmt.Errorf("expected length 128, got %d", len(generated))
	}

	// Should contain ambiguous chars
	if !containsAnyChar(generated, ambiguousChars) {
		return false, nil, fmt.Errorf("generated secret missing ambiguous characters")
	}

	// Should contain lowercase chars
	if !containsAnyChar(generated, lowercaseChars) {
		return false, nil, fmt.Errorf("generated secret missing lowercase characters")
	}

	// Should contain uppercase chars
	if !containsAnyChar(generated, uppercaseChars) {
		return false, nil, fmt.Errorf("generated secret missing uppercase characters")
	}

	// Should contain special chars
	if !containsAnyChar(generated, specialChars) {
		return false, nil, fmt.Errorf("generated secret missing special characters")
	}

	// Count character types
	lowercaseCount := countCharsInSet(generated, lowercaseChars)
	uppercaseCount := countCharsInSet(generated, uppercaseChars)
	numericCount := countCharsInSet(generated, numericChars)
	specialCount := countCharsInSet(generated, specialChars)

	// Should contain at least 2 lowercase chars
	if lowercaseCount < 2 {
		return false, nil, fmt.Errorf("expected at least 2 lowercase, got %d", lowercaseCount)
	}

	// Should contain at least 2 uppercase chars
	if uppercaseCount < 2 {
		return false, nil, fmt.Errorf("expected at least 2 uppercase, got %d", uppercaseCount)
	}

	// Should contain at least 4 numeric chars
	if numericCount < 4 {
		return false, nil, fmt.Errorf("expected at least 4 numeric, got %d", numericCount)
	}

	// Should contain at least 4 special chars
	if specialCount < 4 {
		return false, nil, fmt.Errorf("expected at least 4 special, got %d", specialCount)
	}

	return true, map[string]interface{}{
		"length":          len(generated),
		"lowercase_count": lowercaseCount,
		"uppercase_count": uppercaseCount,
		"numeric_count":   numericCount,
		"special_count":   specialCount,
	}, nil
}

// Helper function to check if string contains any character from a set
func containsAnyChar(str, charset string) bool {
	for _, c := range str {
		if strings.ContainsRune(charset, c) {
			return true
		}
	}
	return false
}

// Helper function to count characters in a set
func countCharsInSet(str, charset string) int {
	count := 0
	for _, c := range str {
		if strings.ContainsRune(charset, c) {
			count++
		}
	}
	return count
}

// TestDefinition holds test metadata
type TestDefinition struct {
	name        string
	testFunc    func() (bool, map[string]interface{}, error)
	displayName string
}

// GetTests returns all test definitions
func (s *GoSDKTestSuite) GetTests() []TestDefinition {
	return []TestDefinition{
		{"test_auth", s.TestAuth, "Authentication"},
		{"test_secret_create", s.TestSecretCreate, "Create Secret"},
		{"test_secret_get", s.TestSecretGet, "Get Secret"},
		{"test_secret_update", s.TestSecretUpdate, "Update Secret"},
		{"test_secret_sync", s.TestSecretSync, "Sync Secrets"},
		{"test_secret_delete", s.TestSecretDelete, "Delete Secrets"},
		{"test_project_list", s.TestProjectList, "List Projects"},
		{"test_project_update", s.TestProjectUpdate, "Update Project"},
		{"test_generator_default", s.TestGeneratorDefault, "Generator Default"},
		{"test_generator_custom", s.TestGeneratorCustom, "Generator Custom"},
	}
}

// GenerateReport outputs the test report
func (s *GoSDKTestSuite) GenerateReport(totalDuration int64) {
	if s.jsonOutput {
		report := TestResult{
			Language:        "go",
			SDKVersion:      getSDKVersion(),
			Operations:      s.operations,
			TotalDurationMs: totalDuration,
			OS:              runtime.GOOS,
			Architecture:    runtime.GOARCH,
			Timestamp:       time.Now().UTC().Format(time.RFC3339),
		}

		jsonData, _ := json.MarshalIndent(report, "", "  ")
		fmt.Println(string(jsonData))
	} else {
		// Print summary
		passed := 0
		failed := 0
		for _, op := range s.operations {
			if op.Success {
				passed++
			} else {
				failed++
			}
		}

		fmt.Println()
		fmt.Println(strings.Repeat("=", 60))
		fmt.Printf("Results: %d/%d passed (%dms)\n", passed, len(s.operations), totalDuration)
		if failed > 0 {
			fmt.Println("Failed operations:")
			for _, op := range s.operations {
				if !op.Success {
					errMsg := "Failed"
					if op.Error != nil {
						errMsg = *op.Error
					}
					fmt.Printf("  - %s: %s\n", op.Operation, errMsg)
				}
			}
		}
		fmt.Println(strings.Repeat("=", 60))
	}
}

// RunAllTests executes all test operations
func (s *GoSDKTestSuite) RunAllTests() int {
	s.startTime = time.Now()

	// Validate required environment variables
	if s.organizationID == "" {
		fmt.Fprintln(os.Stderr, "ORGANIZATION_ID environment variable must be set")
		return 1
	}

	if err := s.SetupClient(); err != nil {
		fmt.Fprintf(os.Stderr, "Failed to setup client: %v\n", err)
		return 1
	}

	// Print header for text mode
	if !s.jsonOutput {
		fmt.Println(strings.Repeat("=", 60))
		fmt.Println("Go SDK Test Suite")
		fmt.Println(strings.Repeat("=", 60))
		fmt.Println()
	}

	// Get and run all tests
	tests := s.GetTests()
	for _, test := range tests {
		s.RunOperation(test.name, test.testFunc, test.displayName)
	}

	totalDuration := time.Since(s.startTime).Milliseconds()

	// Generate report
	s.GenerateReport(totalDuration)

	// Return appropriate exit code
	allPassed := true
	for _, op := range s.operations {
		if !op.Success {
			allPassed = false
			break
		}
	}

	if allPassed {
		return 0
	}
	return 1
}

// Helper functions

func getSDKVersion() string {
	// Try to get version from Go modules
	bi, ok := debug.ReadBuildInfo()
	if ok {
		for _, dep := range bi.Deps {
			if strings.Contains(dep.Path, "github.com/bitwarden/sdk-go") {
				return dep.Version
			}
		}
	}
	return "unknown"
}

func main() {
	jsonOutput := flag.Bool("json", false, "Output JSON format")
	verbose := flag.Bool("verbose", false, "Verbose output")
	flag.Parse()

	suite := NewTestSuite(*jsonOutput, *verbose)
	os.Exit(suite.RunAllTests())
}
