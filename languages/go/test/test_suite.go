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
	client           sdk.BitwardenClientInterface
	organizationID   string
	testMode         string
	jsonOutput       bool
	verbose          bool
	operations       []TestOperation
	startTime        time.Time
	createdSecretIDs []string
	createdProjectIDs []string
}

// NewTestSuite creates a new test suite instance
func NewTestSuite(jsonOutput, verbose bool) *GoSDKTestSuite {
	return &GoSDKTestSuite{
		jsonOutput:        jsonOutput,
		verbose:           verbose,
		organizationID:    os.Getenv("ORGANIZATION_ID"),
		testMode:         os.Getenv("TEST_MODE"),
		operations:       []TestOperation{},
		createdSecretIDs: []string{},
		createdProjectIDs: []string{},
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

// RunOperation executes a test operation and tracks the result
func (s *GoSDKTestSuite) RunOperation(name string, testFunc func() (bool, map[string]interface{}, error), displayName string) bool {
	if displayName == "" {
		displayName = strings.ReplaceAll(name, "_", " ")
		displayName = strings.Title(displayName)
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

	return true, map[string]interface{}{
		"method":    "access_token",
		"has_state": stateFile != "",
	}, nil
}

// TestSecretCreate creates a secret
func (s *GoSDKTestSuite) TestSecretCreate() (bool, map[string]interface{}, error) {
	secretName := fmt.Sprintf("test-secret-%s", uuid.Must(uuid.NewV4()).String()[:8])

	secret, err := s.client.Secrets().Create(
		secretName,
		"test-value",
		"Created by test suite",
		s.organizationID,
		[]string{},
	)
	if err != nil {
		return false, nil, err
	}

	s.createdSecretIDs = append(s.createdSecretIDs, secret.ID)
	return true, map[string]interface{}{
		"id":  secret.ID,
		"key": secret.Key,
	}, nil
}

// TestSecretList lists secrets
func (s *GoSDKTestSuite) TestSecretList() (bool, map[string]interface{}, error) {
	secretsList, err := s.client.Secrets().List(s.organizationID)
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"count": len(secretsList.Data),
	}, nil
}

// TestSecretGet gets a secret
func (s *GoSDKTestSuite) TestSecretGet() (bool, map[string]interface{}, error) {
	if s.testMode == "fake-server" {
		// Fake server returns specific test data
		secret, err := s.client.Secrets().Get(uuid.Must(uuid.NewV4()).String())
		if err != nil {
			return false, nil, err
		}
		return secret.Key == "btw", map[string]interface{}{
			"key": secret.Key,
		}, nil
	}

	// Real server - use created secret
	if len(s.createdSecretIDs) == 0 {
		_, _, err := s.TestSecretCreate()
		if err != nil {
			return false, nil, err
		}
	}

	secret, err := s.client.Secrets().Get(s.createdSecretIDs[0])
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"id":  secret.ID,
		"key": secret.Key,
	}, nil
}

// TestSecretUpdate updates a secret
func (s *GoSDKTestSuite) TestSecretUpdate() (bool, map[string]interface{}, error) {
	if len(s.createdSecretIDs) == 0 {
		_, _, err := s.TestSecretCreate()
		if err != nil {
			return false, nil, err
		}
	}

	secretID := s.createdSecretIDs[0]
	updated, err := s.client.Secrets().Update(
		secretID,
		"updated-key",
		"updated-value",
		"Updated by test",
		s.organizationID,
		[]string{},
	)
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"id":  secretID,
		"key": updated.Key,
	}, nil
}

// TestSecretGetByIDs gets multiple secrets by IDs
func (s *GoSDKTestSuite) TestSecretGetByIDs() (bool, map[string]interface{}, error) {
	ids := []string{
		uuid.Must(uuid.NewV4()).String(),
		uuid.Must(uuid.NewV4()).String(),
		uuid.Must(uuid.NewV4()).String(),
	}

	secrets, err := s.client.Secrets().GetByIDS(ids)
	if err != nil {
		return false, nil, err
	}

	if s.testMode == "fake-server" && len(secrets.Data) > 0 {
		// Fake server returns specific test data
		return secrets.Data[0].Key == "FERRIS", map[string]interface{}{
			"first_key": secrets.Data[0].Key,
		}, nil
	}

	return len(secrets.Data) > 0, map[string]interface{}{
		"count": len(secrets.Data),
	}, nil
}

// TestSecretSync tests sync functionality
func (s *GoSDKTestSuite) TestSecretSync() (bool, map[string]interface{}, error) {
	// Initial sync
	syncResponse, err := s.client.Secrets().Sync(s.organizationID, nil)
	if err != nil {
		return false, nil, err
	}

	// Sync with current date
	now := time.Now()
	syncResponseWithDate, err := s.client.Secrets().Sync(s.organizationID, &now)
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"initial_has_changes": syncResponse.HasChanges,
		"after_has_changes":   syncResponseWithDate.HasChanges,
	}, nil
}

// TestSecretDelete deletes secrets
func (s *GoSDKTestSuite) TestSecretDelete() (bool, map[string]interface{}, error) {
	var idsToDelete []string

	if len(s.createdSecretIDs) > 0 {
		if len(s.createdSecretIDs) > 1 {
			idsToDelete = s.createdSecretIDs[:2]
		} else {
			idsToDelete = s.createdSecretIDs
		}
	} else {
		idsToDelete = []string{
			uuid.Must(uuid.NewV4()).String(),
			uuid.Must(uuid.NewV4()).String(),
		}
	}

	_, err := s.client.Secrets().Delete(idsToDelete)
	if err != nil {
		return false, nil, err
	}

	// Clean tracking
	for _, id := range idsToDelete {
		s.removeCreatedSecretID(id)
	}

	return true, map[string]interface{}{
		"deleted": len(idsToDelete),
	}, nil
}

// TestProjectCreate creates a project
func (s *GoSDKTestSuite) TestProjectCreate() (bool, map[string]interface{}, error) {
	projectName := fmt.Sprintf("test-project-%s", uuid.Must(uuid.NewV4()).String()[:8])

	project, err := s.client.Projects().Create(s.organizationID, projectName)
	if err != nil {
		return false, nil, err
	}

	s.createdProjectIDs = append(s.createdProjectIDs, project.ID)
	return true, map[string]interface{}{
		"id":   project.ID,
		"name": project.Name,
	}, nil
}

// TestProjectList lists projects
func (s *GoSDKTestSuite) TestProjectList() (bool, map[string]interface{}, error) {
	projectsList, err := s.client.Projects().List(s.organizationID)
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"count": len(projectsList.Data),
	}, nil
}

// TestProjectGet gets a project
func (s *GoSDKTestSuite) TestProjectGet() (bool, map[string]interface{}, error) {
	if s.testMode == "fake-server" {
		project, err := s.client.Projects().Get(uuid.Must(uuid.NewV4()).String())
		if err != nil {
			return false, nil, err
		}
		return project.Name == "Production Environment", map[string]interface{}{
			"name": project.Name,
		}, nil
	}

	// Real server - use created project
	if len(s.createdProjectIDs) == 0 {
		_, _, err := s.TestProjectCreate()
		if err != nil {
			return false, nil, err
		}
	}

	project, err := s.client.Projects().Get(s.createdProjectIDs[0])
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"id":   project.ID,
		"name": project.Name,
	}, nil
}

// TestProjectUpdate updates a project
func (s *GoSDKTestSuite) TestProjectUpdate() (bool, map[string]interface{}, error) {
	var projectID string

	if s.testMode == "fake-server" {
		projectID = uuid.Must(uuid.NewV4()).String()
	} else {
		if len(s.createdProjectIDs) == 0 {
			_, _, err := s.TestProjectCreate()
			if err != nil {
				return false, nil, err
			}
		}
		projectID = s.createdProjectIDs[0]
	}

	newName := fmt.Sprintf("updated-project-%s", uuid.Must(uuid.NewV4()).String()[:8])
	updated, err := s.client.Projects().Update(projectID, s.organizationID, newName)
	if err != nil {
		return false, nil, err
	}

	return strings.Contains(updated.Name, newName), map[string]interface{}{
		"name": updated.Name,
	}, nil
}

// TestProjectDelete deletes projects
func (s *GoSDKTestSuite) TestProjectDelete() (bool, map[string]interface{}, error) {
	var idsToDelete []string

	if len(s.createdProjectIDs) > 0 {
		if len(s.createdProjectIDs) > 1 {
			idsToDelete = s.createdProjectIDs[:2]
		} else {
			idsToDelete = s.createdProjectIDs
		}
	} else {
		idsToDelete = []string{
			uuid.Must(uuid.NewV4()).String(),
			uuid.Must(uuid.NewV4()).String(),
		}
	}

	_, err := s.client.Projects().Delete(idsToDelete)
	if err != nil {
		return false, nil, err
	}

	// Clean tracking
	for _, id := range idsToDelete {
		s.removeCreatedProjectID(id)
	}

	return true, map[string]interface{}{
		"deleted": len(idsToDelete),
	}, nil
}

// TestGeneratorDefault tests password generation with defaults
func (s *GoSDKTestSuite) TestGeneratorDefault() (bool, map[string]interface{}, error) {
	password, err := s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Length:    24,
		Lowercase: true,
		Uppercase: true,
		Numbers:   true,
		Special:   true,
	})
	if err != nil {
		return false, nil, err
	}

	// Basic validation
	if password == nil {
		return false, nil, fmt.Errorf("password generation returned nil")
	}

	checks := map[string]bool{
		"length_ok":      len(*password) == 24,
		"has_lowercase":  containsLowercase(*password),
		"has_uppercase":  containsUppercase(*password),
		"has_numbers":    containsNumbers(*password),
		"has_special":    containsSpecial(*password),
	}

	allChecksPass := true
	for _, v := range checks {
		if !v {
			allChecksPass = false
			break
		}
	}

	checksInterface := make(map[string]interface{})
	for k, v := range checks {
		checksInterface[k] = v
	}

	return allChecksPass, checksInterface, nil
}

// TestGeneratorCustom tests password generation with custom params
func (s *GoSDKTestSuite) TestGeneratorCustom() (bool, map[string]interface{}, error) {
	minVal := int64(2)
	password, err := s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Length:       32,
		Lowercase:    true,
		Uppercase:    true,
		Numbers:      true,
		Special:      true,
		MinLowercase: &minVal,
		MinUppercase: &minVal,
		MinNumber:    &minVal,
		MinSpecial:   &minVal,
	})
	if err != nil {
		return false, nil, err
	}

	if password == nil {
		return false, nil, fmt.Errorf("password generation returned nil")
	}

	return len(*password) == 32, map[string]interface{}{
		"length": len(*password),
	}, nil
}

// TestGeneratorValidation tests generator input validation
func (s *GoSDKTestSuite) TestGeneratorValidation() (bool, map[string]interface{}, error) {
	// Should fail - all character types disabled
	_, err := s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Length:    20,
		Lowercase: false,
		Uppercase: false,
		Numbers:   false,
		Special:   false,
	})

	if err == nil {
		return false, map[string]interface{}{
			"error": "Should have raised error",
		}, nil
	}

	return true, map[string]interface{}{
		"validation": "correctly rejected invalid params",
	}, nil
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
		{"test_secret_list", s.TestSecretList, "List Secrets"},
		{"test_secret_get", s.TestSecretGet, "Get Secret"},
		{"test_secret_update", s.TestSecretUpdate, "Update Secret"},
		{"test_secret_get_by_ids", s.TestSecretGetByIDs, "Get Secrets by IDs"},
		{"test_secret_sync", s.TestSecretSync, "Sync Secrets"},
		{"test_secret_delete", s.TestSecretDelete, "Delete Secrets"},
		{"test_project_create", s.TestProjectCreate, "Create Project"},
		{"test_project_list", s.TestProjectList, "List Projects"},
		{"test_project_get", s.TestProjectGet, "Get Project"},
		{"test_project_update", s.TestProjectUpdate, "Update Project"},
		{"test_project_delete", s.TestProjectDelete, "Delete Projects"},
		{"test_generator_default", s.TestGeneratorDefault, "Generate Password (Default)"},
		{"test_generator_custom", s.TestGeneratorCustom, "Generate Password (Custom)"},
		{"test_generator_validation", s.TestGeneratorValidation, "Generator Validation"},
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
		testMode := s.testMode
		if testMode == "" {
			testMode = "default"
		}
		fmt.Printf("Mode: %s\n", testMode)
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

	if allPassed || s.jsonOutput {
		return 0
	}
	return 1
}

// Helper functions

func (s *GoSDKTestSuite) removeCreatedSecretID(id string) {
	for i, sid := range s.createdSecretIDs {
		if sid == id {
			s.createdSecretIDs = append(s.createdSecretIDs[:i], s.createdSecretIDs[i+1:]...)
			break
		}
	}
}

func (s *GoSDKTestSuite) removeCreatedProjectID(id string) {
	for i, pid := range s.createdProjectIDs {
		if pid == id {
			s.createdProjectIDs = append(s.createdProjectIDs[:i], s.createdProjectIDs[i+1:]...)
			break
		}
	}
}


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


func containsLowercase(s string) bool {
	for _, r := range s {
		if r >= 'a' && r <= 'z' {
			return true
		}
	}
	return false
}

func containsUppercase(s string) bool {
	for _, r := range s {
		if r >= 'A' && r <= 'Z' {
			return true
		}
	}
	return false
}

func containsNumbers(s string) bool {
	for _, r := range s {
		if r >= '0' && r <= '9' {
			return true
		}
	}
	return false
}

func containsSpecial(s string) bool {
	for _, r := range s {
		if !((r >= 'a' && r <= 'z') || (r >= 'A' && r <= 'Z') || (r >= '0' && r <= '9')) {
			return true
		}
	}
	return false
}

func main() {
	jsonOutput := flag.Bool("json", false, "Output JSON format")
	verbose := flag.Bool("verbose", false, "Verbose output")
	flag.Parse()

	suite := NewTestSuite(*jsonOutput, *verbose)
	os.Exit(suite.RunAllTests())
}