package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"runtime"
	"runtime/debug"
	"strings"
	"time"

	sdk "github.com/bitwarden/sdk-go/v2"
	"github.com/gofrs/uuid"
)

// Simple test result structures for CI/CD integration (matching C# framework expectations)

// SimpleTestOperation represents a single test operation result in simple format
type SimpleTestOperation struct {
	Operation  string                 `json:"operation"`
	Success    bool                   `json:"success"`
	DurationMs int64                  `json:"duration_ms"`
	Error      *string                `json:"error"`
	Details    map[string]interface{} `json:"details,omitempty"`
}

// SimpleTestResult contains all test results in simple format
type SimpleTestResult struct {
	Language        string                `json:"language"`
	SDKVersion      string                `json:"sdk_version"`
	Operations      []SimpleTestOperation `json:"operations"`
	TotalDurationMs int64                 `json:"total_duration_ms"`
	OS              string                `json:"os"`
	Architecture    string                `json:"architecture"`
	Timestamp       string                `json:"timestamp"`
}

// Enhanced test result structures matching the comprehensive JSON format

// TestStatus represents the status of a test
type TestStatus string

const (
	StatusPassed  TestStatus = "passed"
	StatusFailed  TestStatus = "failed"
	StatusSkipped TestStatus = "skipped"
	StatusError   TestStatus = "error"
)

// ErrorInfo contains detailed error information
type ErrorInfo struct {
	Type     string `json:"type"`
	Message  string `json:"message"`
	Code     string `json:"code"`
	File     string `json:"file"`
	Line     int    `json:"line"`
	Function string `json:"function"`
}

// FailedAssertion represents a failed assertion
type FailedAssertion struct {
	Expected interface{} `json:"expected"`
	Actual   interface{} `json:"actual"`
	Operator string      `json:"operator"`
	Message  string      `json:"message"`
}

// TestCase represents a single test case result
type TestCase struct {
	Name             string                 `json:"name"`
	DisplayName      string                 `json:"display_name"`
	DurationMs       int64                  `json:"duration_ms"`
	Category         string                 `json:"category"`
	Status           TestStatus             `json:"status"`
	Assertions       int                    `json:"assertions,omitempty"`
	Details          map[string]interface{} `json:"details,omitempty"`
	Error            *ErrorInfo             `json:"error,omitempty"`
	StackTrace       []string               `json:"stack_trace,omitempty"`
	FailedAssertions []FailedAssertion      `json:"failed_assertions,omitempty"`
	RequestDetails   map[string]interface{} `json:"request_details,omitempty"`
	ResponseDetails  map[string]interface{} `json:"response_details,omitempty"`
}

// TestSummary contains summary statistics
type TestSummary struct {
	Total      int        `json:"total"`
	Passed     int        `json:"passed"`
	Failed     int        `json:"failed"`
	Errored    int        `json:"errored"`
	Skipped    int        `json:"skipped"`
	DurationMs int64      `json:"duration_ms"`
	Status     TestStatus `json:"status"`
}

// TestResults contains categorized test results
type TestResults struct {
	Summary      TestSummary `json:"summary"`
	PassedTests  []TestCase  `json:"passed_tests"`
	FailedTests  []TestCase  `json:"failed_tests"`
	ErrorTests   []TestCase  `json:"error_tests"`
	SkippedTests []TestCase  `json:"skipped_tests"`
}

// TestExecution contains test execution information
type TestExecution struct {
	StartTime    string       `json:"start_time"`
	EndTime      string       `json:"end_time"`
	DurationMs   int64        `json:"duration_ms"`
	TestOrder    string       `json:"test_order"`
	Parallelism  int          `json:"parallelism"`
	RetryPolicy  RetryPolicy  `json:"retry_policy"`
}

// RetryPolicy defines retry behavior
type RetryPolicy struct {
	Enabled    bool `json:"enabled"`
	MaxRetries int  `json:"max_retries"`
}

// OSInfo contains operating system information
type OSInfo struct {
	Platform string `json:"platform"`
	Version  string `json:"version"`
	Kernel   string `json:"kernel"`
}

// CPUInfo contains CPU information
type CPUInfo struct {
	Model   string `json:"model,omitempty"`
	Cores   int    `json:"cores"`
	Threads int    `json:"threads,omitempty"`
}

// MemoryInfo contains memory information
type MemoryInfo struct {
	TotalGB     float64 `json:"total_gb,omitempty"`
	AvailableGB float64 `json:"available_gb,omitempty"`
}

// RuntimeInfo contains runtime information
type RuntimeInfo struct {
	Name    string `json:"name"`
	Version string `json:"version"`
	Path    string `json:"path"`
}

// SDKInfo contains SDK information
type SDKInfo struct {
	Source string `json:"source"`
	Path   string `json:"path"`
}

// ServerInfo contains server information
type ServerInfo struct {
	Type    string `json:"type"`
	URL     string `json:"url"`
	Version string `json:"version,omitempty"`
	Healthy bool   `json:"healthy"`
}

// Environment contains environment information
type Environment struct {
	OS            OSInfo                 `json:"os"`
	Arch          string                 `json:"arch"`
	CPU           CPUInfo                `json:"cpu"`
	Memory        MemoryInfo             `json:"memory"`
	Runtime       RuntimeInfo            `json:"runtime"`
	SDK           SDKInfo                `json:"sdk"`
	Server        ServerInfo             `json:"server"`
	EnvVariables  map[string]string      `json:"env_variables"`
}

// BuildInfo contains build information
type BuildInfo struct {
	Commit        string `json:"commit"`
	Branch        string `json:"branch"`
	Tag           string `json:"tag,omitempty"`
	BuiltAt       string `json:"built_at"`
	Builder       string `json:"builder,omitempty"`
	BuildNumber   string `json:"build_number,omitempty"`
	Dirty         bool   `json:"dirty"`
	GoVersion     string `json:"go_version"`
}

// Logs contains categorized logs
type Logs struct {
	Stdout []string `json:"stdout"`
	Stderr []string `json:"stderr"`
	Debug  []string `json:"debug"`
}

// ComprehensiveTestResult is the complete test result structure
type ComprehensiveTestResult struct {
	Language      string        `json:"language"`
	SDKVersion    string        `json:"sdk_version"`
	TestResults   TestResults   `json:"test_results"`
	TestExecution TestExecution `json:"test_execution"`
	Environment   Environment   `json:"environment"`
	BuildInfo     BuildInfo     `json:"build_info"`
	Logs          Logs          `json:"logs"`
	Timestamp     string        `json:"timestamp"`
}

// GoSDKTestSuite runs comprehensive Go SDK tests
type GoSDKTestSuite struct {
	client           sdk.BitwardenClientInterface
	organizationID   string
	stateFile        string
	testMode         string
	sdkSource        string
	outputFormat     string
	verbose          bool

	// Test tracking
	testCases        []TestCase
	logs             Logs
	startTime        time.Time
	endTime          time.Time

	// Test data
	createdSecretIDs []string
	createdProjectIDs []string
}

// NewGoSDKTestSuite creates a new test suite instance
func NewGoSDKTestSuite(outputFormat string, verbose bool) *GoSDKTestSuite {
	return &GoSDKTestSuite{
		organizationID: os.Getenv("ORGANIZATION_ID"),
		stateFile:     os.Getenv("STATE_FILE"),
		testMode:      getEnvOrDefault("TEST_MODE", "fake-server"),
		sdkSource:     getEnvOrDefault("SDK_SOURCE", "local-build"),
		outputFormat:  outputFormat,
		verbose:       verbose,
		testCases:     make([]TestCase, 0),
		logs: Logs{
			Stdout: make([]string, 0),
			Stderr: make([]string, 0),
			Debug:  make([]string, 0),
		},
		createdSecretIDs:  make([]string, 0),
		createdProjectIDs: make([]string, 0),
	}
}

// Logging methods

func (s *GoSDKTestSuite) logStdout(message string) {
	timestamp := time.Now().UTC().Format(time.RFC3339)
	logEntry := fmt.Sprintf("[%s] %s", timestamp, message)
	s.logs.Stdout = append(s.logs.Stdout, logEntry)
	if s.outputFormat == "text" {
		fmt.Println(message)
	}
}

func (s *GoSDKTestSuite) logStderr(message string) {
	timestamp := time.Now().UTC().Format(time.RFC3339)
	logEntry := fmt.Sprintf("[%s] %s", timestamp, message)
	s.logs.Stderr = append(s.logs.Stderr, logEntry)
	if s.outputFormat == "text" {
		fmt.Fprintln(os.Stderr, message)
	}
}

func (s *GoSDKTestSuite) logDebug(message string) {
	if s.verbose {
		timestamp := time.Now().UTC().Format(time.RFC3339)
		logEntry := fmt.Sprintf("[%s] DEBUG: %s", timestamp, message)
		s.logs.Debug = append(s.logs.Debug, logEntry)
		if s.outputFormat == "text" {
			fmt.Fprintf(os.Stderr, "DEBUG: %s\n", message)
		}
	}
}

// Test runner method

func (s *GoSDKTestSuite) runTest(testFunc func() (bool, map[string]interface{}, error),
	name string, displayName string, category string) TestCase {

	tc := TestCase{
		Name:        name,
		DisplayName: displayName,
		Category:    category,
		Status:      StatusPassed,
	}

	start := time.Now()
	s.logDebug(fmt.Sprintf("Running test: %s", displayName))

	// Recover from panics
	defer func() {
		if r := recover(); r != nil {
			tc.Status = StatusError
			tc.Error = &ErrorInfo{
				Type:    "panic",
				Message: fmt.Sprintf("%v", r),
				Code:    "PANIC",
			}
			tc.StackTrace = strings.Split(string(debug.Stack()), "\n")
			tc.DurationMs = time.Since(start).Milliseconds()
		}
	}()

	success, details, err := testFunc()
	tc.DurationMs = time.Since(start).Milliseconds()

	if err != nil {
		tc.Status = StatusFailed
		tc.Error = &ErrorInfo{
			Type:    fmt.Sprintf("%T", err),
			Message: err.Error(),
			Code:    s.getErrorCode(fmt.Sprintf("%T", err)),
		}
		// Capture stack trace if available
		if s.verbose {
			tc.StackTrace = strings.Split(string(debug.Stack()), "\n")
		}
	} else if success {
		tc.Status = StatusPassed
		tc.Details = details
	} else {
		tc.Status = StatusFailed
		if details != nil {
			if msg, ok := details["error"].(string); ok {
				tc.Error = &ErrorInfo{
					Type:    "AssertionError",
					Message: msg,
					Code:    "ASSERTION_FAILED",
				}
			}
		}
	}

	// Log result
	if s.outputFormat == "text" {
		symbol := "✅"
		if tc.Status == StatusFailed {
			symbol = "❌"
		} else if tc.Status == StatusError {
			symbol = "💥"
		} else if tc.Status == StatusSkipped {
			symbol = "⏭️"
		}
		s.logStdout(fmt.Sprintf("%s %s (%dms)", symbol, displayName, tc.DurationMs))
	}

	s.testCases = append(s.testCases, tc)
	return tc
}

func (s *GoSDKTestSuite) getErrorCode(errorType string) string {
	errorCodes := map[string]string{
		"*errors.errorString": "GENERIC_ERROR",
		"error":              "UNKNOWN_ERROR",
	}
	if code, ok := errorCodes[errorType]; ok {
		return code
	}
	return "UNKNOWN_ERROR"
}

// SetupClient initializes the Bitwarden client
func (s *GoSDKTestSuite) SetupClient() error {
	apiURL := getEnvOrDefault("API_URL", "http://localhost:4000")
	identityURL := getEnvOrDefault("IDENTITY_URL", "http://localhost:33656")

	s.logDebug(fmt.Sprintf("Setting up client with API: %s, Identity: %s", apiURL, identityURL))

	client, err := sdk.NewBitwardenClient(&apiURL, &identityURL)
	if err != nil {
		return fmt.Errorf("failed to create client: %v", err)
	}

	s.client = client
	return nil
}

// Test methods

func (s *GoSDKTestSuite) testAuth() (bool, map[string]interface{}, error) {
	accessToken := os.Getenv("ACCESS_TOKEN")
	if accessToken == "" {
		return false, nil, fmt.Errorf("ACCESS_TOKEN not set")
	}

	err := s.client.AccessTokenLogin(accessToken, &s.stateFile)
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"method":     "access_token",
		"state_file": s.stateFile != "",
	}, nil
}

func (s *GoSDKTestSuite) testSecretCreate() (bool, map[string]interface{}, error) {
	secretName := fmt.Sprintf("test-secret-%s", uuid.Must(uuid.NewV4()).String()[:8])
	secret, err := s.client.Secrets().Create(
		secretName,
		"test-value",
		"Created by Go SDK test",
		s.organizationID,
		[]string{},
	)

	if err != nil {
		return false, nil, err
	}

	s.createdSecretIDs = append(s.createdSecretIDs, secret.ID)
	return true, map[string]interface{}{
		"secret_id":  secret.ID,
		"secret_key": secret.Key,
	}, nil
}

func (s *GoSDKTestSuite) testSecretList() (bool, map[string]interface{}, error) {
	secretsList, err := s.client.Secrets().List(s.organizationID)
	if err != nil {
		return false, nil, err
	}

	return true, map[string]interface{}{
		"count": len(secretsList.Data),
	}, nil
}

func (s *GoSDKTestSuite) testSecretGet() (bool, map[string]interface{}, error) {
	if s.testMode == "fake-server" {
		// Fake server returns "btw" for any random UUID
		secret, err := s.client.Secrets().Get(uuid.Must(uuid.NewV4()).String())
		if err != nil {
			return false, nil, err
		}

		success := secret.Key == "btw"
		return success, map[string]interface{}{
			"secret_key": secret.Key,
		}, nil
	}

	// Real server - get the created secret
	if len(s.createdSecretIDs) == 0 {
		return false, nil, fmt.Errorf("no secret created to retrieve")
	}

	secret, err := s.client.Secrets().Get(s.createdSecretIDs[0])
	if err != nil {
		return false, nil, err
	}

	success := secret.ID == s.createdSecretIDs[0]
	return success, map[string]interface{}{
		"secret_id":  secret.ID,
		"secret_key": secret.Key,
	}, nil
}

func (s *GoSDKTestSuite) testSecretUpdate() (bool, map[string]interface{}, error) {
	var secretID string
	if len(s.createdSecretIDs) > 0 {
		secretID = s.createdSecretIDs[0]
	} else {
		// Create a secret first
		secret, err := s.client.Secrets().Create(
			"update-test",
			"original-value",
			"To be updated",
			s.organizationID,
			[]string{},
		)
		if err != nil {
			return false, nil, err
		}
		secretID = secret.ID
		s.createdSecretIDs = append(s.createdSecretIDs, secretID)
	}

	// Update the secret
	updated, err := s.client.Secrets().Update(
		secretID,
		"updated-key",
		"updated-value",
		"Updated note",
		s.organizationID,
		[]string{},
	)

	if err != nil {
		return false, nil, err
	}

	success := strings.Contains(updated.Key, "updated-key")
	return success, map[string]interface{}{
		"secret_id": secretID,
		"new_key":   updated.Key,
	}, nil
}

func (s *GoSDKTestSuite) testSecretGetByIDs() (bool, map[string]interface{}, error) {
	ids := []string{
		uuid.Must(uuid.NewV4()).String(),
		uuid.Must(uuid.NewV4()).String(),
		uuid.Must(uuid.NewV4()).String(),
	}

	secrets, err := s.client.Secrets().GetByIDS(ids)
	if err != nil {
		return false, nil, err
	}

	success := true
	if s.testMode == "fake-server" && len(secrets.Data) > 0 {
		// Fake server returns "FERRIS" for the first secret
		success = secrets.Data[0].Key == "FERRIS"
	}

	return success, map[string]interface{}{
		"requested": len(ids),
		"retrieved": len(secrets.Data),
	}, nil
}

func (s *GoSDKTestSuite) testSecretSync() (bool, map[string]interface{}, error) {
	// First sync without date (should have changes)
	syncResponse, err := s.client.Secrets().Sync(s.organizationID, nil)
	if err != nil {
		return false, nil, err
	}

	hasChangesInitial := syncResponse.HasChanges

	// Second sync with current date (should have no changes)
	now := time.Now()
	syncResponseWithDate, err := s.client.Secrets().Sync(s.organizationID, &now)
	if err != nil {
		return false, nil, err
	}

	hasChangesAfter := syncResponseWithDate.HasChanges

	// For fake-server, expect initial=true, after=false
	var success bool
	if s.testMode == "fake-server" {
		success = hasChangesInitial && !hasChangesAfter
	} else {
		// Real server: just verify the calls succeed
		success = true
	}

	return success, map[string]interface{}{
		"initial_has_changes": hasChangesInitial,
		"after_has_changes":   hasChangesAfter,
	}, nil
}

func (s *GoSDKTestSuite) testSecretDelete() (bool, map[string]interface{}, error) {
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

	// Clean up our tracking
	for _, id := range idsToDelete {
		for i, secretID := range s.createdSecretIDs {
			if secretID == id {
				s.createdSecretIDs = append(s.createdSecretIDs[:i], s.createdSecretIDs[i+1:]...)
				break
			}
		}
	}

	return true, map[string]interface{}{
		"deleted_count": len(idsToDelete),
	}, nil
}

func (s *GoSDKTestSuite) testProjectCreate() (bool, map[string]interface{}, error) {
	projectName := fmt.Sprintf("test-project-%s", uuid.Must(uuid.NewV4()).String()[:8])
	project, err := s.client.Projects().Create(s.organizationID, projectName)

	if err != nil {
		return false, nil, err
	}

	s.createdProjectIDs = append(s.createdProjectIDs, project.ID)
	return true, map[string]interface{}{
		"project_id":   project.ID,
		"project_name": project.Name,
	}, nil
}

func (s *GoSDKTestSuite) testProjectList() (bool, map[string]interface{}, error) {
	projectsList, err := s.client.Projects().List(s.organizationID)
	if err != nil {
		return false, nil, err
	}

	details := map[string]interface{}{
		"count": len(projectsList.Data),
	}

	success := true
	if s.testMode == "fake-server" && len(projectsList.Data) > 0 {
		// Fake server returns "Production Environment" as first project
		success = projectsList.Data[0].Name == "Production Environment"
		details["first_name"] = projectsList.Data[0].Name
	}

	return success, details, nil
}

func (s *GoSDKTestSuite) testProjectGet() (bool, map[string]interface{}, error) {
	if s.testMode == "fake-server" {
		// Fake server returns "Production Environment" for any UUID
		project, err := s.client.Projects().Get(uuid.Must(uuid.NewV4()).String())
		if err != nil {
			return false, nil, err
		}

		success := project.Name == "Production Environment"
		return success, map[string]interface{}{
			"project_name": project.Name,
		}, nil
	}

	// Real server - get the created project
	if len(s.createdProjectIDs) == 0 {
		return false, nil, fmt.Errorf("no project created to retrieve")
	}

	project, err := s.client.Projects().Get(s.createdProjectIDs[0])
	if err != nil {
		return false, nil, err
	}

	success := project.ID == s.createdProjectIDs[0]
	return success, map[string]interface{}{
		"project_id":   project.ID,
		"project_name": project.Name,
	}, nil
}

func (s *GoSDKTestSuite) testProjectUpdate() (bool, map[string]interface{}, error) {
	var projectID string
	if len(s.createdProjectIDs) > 0 {
		projectID = s.createdProjectIDs[0]
	} else {
		// Create a project first
		project, err := s.client.Projects().Create(s.organizationID, "update-test")
		if err != nil {
			return false, nil, err
		}
		projectID = project.ID
		s.createdProjectIDs = append(s.createdProjectIDs, projectID)
	}

	newName := fmt.Sprintf("updated-project-%s", uuid.Must(uuid.NewV4()).String()[:8])

	// Use correct ID based on test mode
	updateID := projectID
	if s.testMode == "fake-server" {
		updateID = uuid.Must(uuid.NewV4()).String()
	}

	updated, err := s.client.Projects().Update(updateID, s.organizationID, newName)
	if err != nil {
		return false, nil, err
	}

	success := strings.Contains(updated.Name, newName)
	return success, map[string]interface{}{
		"project_id": projectID,
		"new_name":   updated.Name,
	}, nil
}

func (s *GoSDKTestSuite) testProjectDelete() (bool, map[string]interface{}, error) {
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

	// Clean up our tracking
	for _, id := range idsToDelete {
		for i, projectID := range s.createdProjectIDs {
			if projectID == id {
				s.createdProjectIDs = append(s.createdProjectIDs[:i], s.createdProjectIDs[i+1:]...)
				break
			}
		}
	}

	return true, map[string]interface{}{
		"deleted_count": len(idsToDelete),
	}, nil
}

// testGeneratorDefault tests password generation with default parameters
func (s *GoSDKTestSuite) testGeneratorDefault() (bool, map[string]interface{}, error) {
	// Generate password with default settings
	// Go SDK requires explicit values (no defaults like Python)
	password, err := s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Length:         16,
		Lowercase:      true,
		Uppercase:      true,
		Numbers:        true,
		Special:        false,
		AvoidAmbiguous: true,
	})
	if err != nil {
		return false, nil, err
	}

	// Check password characteristics
	checks := map[string]bool{
		"length_16": len(*password) == 16,
		"has_lowercase": containsAny(*password, "abcdefghijklmnopqrstuvwxyz"),
		"has_uppercase": containsAny(*password, "ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
		"has_numbers": containsAny(*password, "0123456789"),
		"no_special": !containsAny(*password, "!@#$%^&*()-_=+[]{}|;:'\",.<>/?`~"),
		"no_ambiguous": !containsAny(*password, "0O1lI"),
	}

	success := checks["length_16"] && checks["has_lowercase"] && checks["has_uppercase"] &&
		checks["has_numbers"] && checks["no_special"] && checks["no_ambiguous"]

	sample := *password
	if len(sample) > 10 {
		sample = sample[:10] + "..."
	}

	return success, map[string]interface{}{
		"checks": checks,
		"sample": sample,
	}, nil
}

// testGeneratorCustomParams tests password generation with custom parameters
func (s *GoSDKTestSuite) testGeneratorCustomParams() (bool, map[string]interface{}, error) {
	// Generate password with custom settings
	minLowercase := int64(2)
	minUppercase := int64(2)
	minNumber := int64(4)
	minSpecial := int64(4)

	password, err := s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Length:         128,
		AvoidAmbiguous: false,
		Lowercase:      true,
		Uppercase:      true,
		Numbers:        true,
		Special:        true,
		MinLowercase:   &minLowercase,
		MinUppercase:   &minUppercase,
		MinNumber:      &minNumber,
		MinSpecial:     &minSpecial,
	})
	if err != nil {
		return false, nil, err
	}

	// Check password characteristics
	checks := map[string]bool{
		"length_128": len(*password) == 128,
		"has_ambiguous": containsAny(*password, "0O1lI"),
		"min_lowercase_2": countChars(*password, "abcdefghijklmnopqrstuvwxyz") >= 2,
		"min_uppercase_2": countChars(*password, "ABCDEFGHIJKLMNOPQRSTUVWXYZ") >= 2,
		"min_numbers_4": countChars(*password, "0123456789") >= 4,
		"min_special_4": countChars(*password, "!@#$%^&*()-_=+[]{}|;:'\",.<>/?`~") >= 4,
	}

	success := true
	for _, check := range checks {
		if !check {
			success = false
			break
		}
	}

	return success, map[string]interface{}{
		"checks": checks,
	}, nil
}

// testGeneratorValidationErrors tests generator input validation
func (s *GoSDKTestSuite) testGeneratorValidationErrors() (bool, map[string]interface{}, error) {
	errorCases := []map[string]interface{}{}

	// Test 1: All character sets disabled
	_, err := s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Lowercase: false,
		Uppercase: false,
		Numbers:   false,
		Special:   false,
	})

	if err != nil {
		errorCases = append(errorCases, map[string]interface{}{
			"test": "all_disabled",
			"success": true,
			"message": "Correctly raised error",
		})
	} else {
		errorCases = append(errorCases, map[string]interface{}{
			"test": "all_disabled",
			"success": false,
			"message": "Should have raised error",
		})
	}

	// Test 2: Invalid length (0)
	_, err = s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Length: 0,
	})

	if err != nil {
		errorCases = append(errorCases, map[string]interface{}{
			"test": "zero_length",
			"success": true,
			"message": "Correctly raised error",
		})
	} else {
		errorCases = append(errorCases, map[string]interface{}{
			"test": "zero_length",
			"success": false,
			"message": "Should have raised error",
		})
	}

	// Test 3: Min values exceed length
	minLowercase := int64(20)
	_, err = s.client.Generators().GeneratePassword(sdk.PasswordGeneratorRequest{
		Length:       10,
		MinLowercase: &minLowercase,
	})

	if err != nil {
		errorCases = append(errorCases, map[string]interface{}{
			"test": "min_exceeds_length",
			"success": true,
			"message": "Correctly raised error",
		})
	} else {
		errorCases = append(errorCases, map[string]interface{}{
			"test": "min_exceeds_length",
			"success": false,
			"message": "Should have raised error",
		})
	}

	// Check if all tests passed
	allPassed := true
	for _, tc := range errorCases {
		if success, ok := tc["success"].(bool); ok && !success {
			allPassed = false
			break
		}
	}

	return allPassed, map[string]interface{}{
		"error_cases": errorCases,
	}, nil
}

// Helper function to check if string contains any character from chars
func containsAny(s string, chars string) bool {
	for _, c := range s {
		if strings.ContainsRune(chars, c) {
			return true
		}
	}
	return false
}

// Helper function to count characters from a set
func countChars(s string, chars string) int {
	count := 0
	for _, c := range s {
		if strings.ContainsRune(chars, c) {
			count++
		}
	}
	return count
}

// RunAllTests runs all test operations
func (s *GoSDKTestSuite) RunAllTests() {
	s.startTime = time.Now()

	// Setup client
	if err := s.SetupClient(); err != nil {
		s.logStderr(fmt.Sprintf("Failed to setup client: %v", err))
		return
	}

	// Define all tests with categories
	allTests := []struct {
		testFunc    func() (bool, map[string]interface{}, error)
		name        string
		displayName string
		category    string
	}{
		// Authentication
		{s.testAuth, "test_auth", "Authentication with Access Token", "auth"},

		// Secret operations
		{s.testSecretCreate, "test_secret_create", "Create Secret", "secrets"},
		{s.testSecretList, "test_secret_list", "List Secrets", "secrets"},
		{s.testSecretGet, "test_secret_get", "Get Secret", "secrets"},
		{s.testSecretUpdate, "test_secret_update", "Update Secret", "secrets"},
		{s.testSecretGetByIDs, "test_secret_get_by_ids", "Get Secrets by IDs", "secrets"},
		{s.testSecretSync, "test_secret_sync", "Sync Secrets", "secrets"},
		{s.testSecretDelete, "test_secret_delete", "Delete Secrets", "secrets"},

		// Project operations
		{s.testProjectCreate, "test_project_create", "Create Project", "projects"},
		{s.testProjectList, "test_project_list", "List Projects", "projects"},
		{s.testProjectGet, "test_project_get", "Get Project", "projects"},
		{s.testProjectUpdate, "test_project_update", "Update Project", "projects"},
		{s.testProjectDelete, "test_project_delete", "Delete Projects", "projects"},

		// Generator operations
		{s.testGeneratorDefault, "test_generator_default", "Generate with Default Params", "generators"},
		{s.testGeneratorCustomParams, "test_generator_custom_params", "Generate with Custom Params", "generators"},
		{s.testGeneratorValidationErrors, "test_generator_validation_errors", "Generator Input Validation", "generators"},
	}

	// Print header for text output
	if s.outputFormat == "text" {
		s.logStdout("==========================================")
		s.logStdout("Go SDK Test Suite")
		s.logStdout(fmt.Sprintf("Test Mode: %s", s.testMode))
		s.logStdout(fmt.Sprintf("SDK Source: %s", s.sdkSource))
		s.logStdout("==========================================")
		s.logStdout("")
	}

	// Run tests by category
	currentCategory := ""
	for _, test := range allTests {
		if s.outputFormat == "text" && test.category != currentCategory {
			if currentCategory != "" {
				s.logStdout("")
			}
			s.logStdout(fmt.Sprintf("Testing %s...", test.category))
			currentCategory = test.category
		}

		s.runTest(test.testFunc, test.name, test.displayName, test.category)
	}

	// Clean up client
	if s.client != nil {
		s.client.Close()
	}

	s.endTime = time.Now()

	// Print summary for text output
	if s.outputFormat == "text" {
		s.printSummary()
	}
}

func (s *GoSDKTestSuite) printSummary() {
	passed := 0
	failed := 0
	errored := 0
	skipped := 0

	for _, tc := range s.testCases {
		switch tc.Status {
		case StatusPassed:
			passed++
		case StatusFailed:
			failed++
		case StatusError:
			errored++
		case StatusSkipped:
			skipped++
		}
	}

	total := len(s.testCases)
	duration := s.endTime.Sub(s.startTime).Milliseconds()

	s.logStdout("")
	s.logStdout("==========================================")
	s.logStdout("Test Summary")
	s.logStdout(fmt.Sprintf("Total: %d | Passed: %d | Failed: %d | Errors: %d | Skipped: %d",
		total, passed, failed, errored, skipped))
	s.logStdout(fmt.Sprintf("Duration: %dms", duration))

	if failed > 0 || errored > 0 {
		s.logStdout("")
		s.logStdout("Failed/Errored Tests:")
		for _, tc := range s.testCases {
			if tc.Status == StatusFailed || tc.Status == StatusError {
				errorMsg := "Unknown error"
				if tc.Error != nil {
					errorMsg = tc.Error.Message
				}
				s.logStdout(fmt.Sprintf("  ❌ %s: %s", tc.DisplayName, errorMsg))
			}
		}
	}

	s.logStdout("==========================================")

	if failed > 0 || errored > 0 {
		s.logStdout("❌ Test suite failed")
	} else {
		s.logStdout("✅ All tests passed")
	}
}

func (s *GoSDKTestSuite) GenerateJSONReport() ComprehensiveTestResult {
	// Categorize test cases
	var passed, failed, errored, skipped []TestCase
	var totalDuration int64

	for _, tc := range s.testCases {
		switch tc.Status {
		case StatusPassed:
			passed = append(passed, tc)
		case StatusFailed:
			failed = append(failed, tc)
		case StatusError:
			errored = append(errored, tc)
		case StatusSkipped:
			skipped = append(skipped, tc)
		}
	}

	if !s.endTime.IsZero() && !s.startTime.IsZero() {
		totalDuration = s.endTime.Sub(s.startTime).Milliseconds()
	}

	overallStatus := StatusPassed
	if len(failed) > 0 || len(errored) > 0 {
		overallStatus = StatusFailed
	}

	return ComprehensiveTestResult{
		Language:   "go",
		SDKVersion: s.getSDKVersion(),
		TestResults: TestResults{
			Summary: TestSummary{
				Total:      len(s.testCases),
				Passed:     len(passed),
				Failed:     len(failed),
				Errored:    len(errored),
				Skipped:    len(skipped),
				DurationMs: totalDuration,
				Status:     overallStatus,
			},
			PassedTests:  passed,
			FailedTests:  failed,
			ErrorTests:   errored,
			SkippedTests: skipped,
		},
		TestExecution: TestExecution{
			StartTime:   s.startTime.UTC().Format(time.RFC3339),
			EndTime:     s.endTime.UTC().Format(time.RFC3339),
			DurationMs:  totalDuration,
			TestOrder:   "sequential",
			Parallelism: 1,
			RetryPolicy: RetryPolicy{
				Enabled:    false,
				MaxRetries: 0,
			},
		},
		Environment: s.getEnvironmentInfo(),
		BuildInfo:   s.getBuildInfo(),
		Logs:        s.logs,
		Timestamp:   time.Now().UTC().Format(time.RFC3339),
	}
}

func (s *GoSDKTestSuite) getSDKVersion() string {
	// TODO: Get actual SDK version from the SDK package
	return "1.0.0"
}

func (s *GoSDKTestSuite) getEnvironmentInfo() Environment {
	cpuInfo := CPUInfo{
		Cores: runtime.NumCPU(),
	}

	return Environment{
		OS: OSInfo{
			Platform: runtime.GOOS,
			Version:  "", // Would need platform-specific code to get
			Kernel:   "", // Would need platform-specific code to get
		},
		Arch: runtime.GOARCH,
		CPU:  cpuInfo,
		Memory: MemoryInfo{
			// Would need platform-specific code to get memory info
		},
		Runtime: RuntimeInfo{
			Name:    "Go",
			Version: runtime.Version(),
			Path:    os.Args[0],
		},
		SDK: SDKInfo{
			Source: s.sdkSource,
			Path:   os.Getenv("GOPATH"),
		},
		Server: ServerInfo{
			Type:    s.testMode,
			URL:     getEnvOrDefault("API_URL", "http://localhost:4000"),
			Healthy: true, // Would check with actual health endpoint
		},
		EnvVariables: map[string]string{
			"ORGANIZATION_ID": s.organizationID,
			"API_URL":        getEnvOrDefault("API_URL", "http://localhost:4000"),
			"IDENTITY_URL":   getEnvOrDefault("IDENTITY_URL", "http://localhost:33656"),
			"STATE_FILE":     s.stateFile,
			"TEST_MODE":      s.testMode,
			"SDK_SOURCE":     s.sdkSource,
		},
	}
}

func (s *GoSDKTestSuite) getBuildInfo() BuildInfo {
	buildInfo := BuildInfo{
		BuiltAt:   time.Now().UTC().Format(time.RFC3339),
		GoVersion: runtime.Version(),
	}

	// Try to get git information
	if output, err := exec.Command("git", "rev-parse", "HEAD").Output(); err == nil {
		buildInfo.Commit = strings.TrimSpace(string(output))
	}

	if output, err := exec.Command("git", "rev-parse", "--abbrev-ref", "HEAD").Output(); err == nil {
		buildInfo.Branch = strings.TrimSpace(string(output))
	}

	if output, err := exec.Command("git", "status", "--porcelain").Output(); err == nil {
		buildInfo.Dirty = len(strings.TrimSpace(string(output))) > 0
	}

	if output, err := exec.Command("git", "describe", "--tags", "--abbrev=0").Output(); err == nil {
		buildInfo.Tag = strings.TrimSpace(string(output))
	}

	return buildInfo
}

func getEnvOrDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

// GenerateSimpleJSONReport generates a simple JSON report for CI/CD integration
func (s *GoSDKTestSuite) GenerateSimpleJSONReport() SimpleTestResult {
	var operations []SimpleTestOperation
	totalDuration := int64(0)

	// Convert test cases to simple operations
	for _, tc := range s.testCases {
		op := SimpleTestOperation{
			Operation:  tc.Name,
			Success:    tc.Status == StatusPassed,
			DurationMs: tc.DurationMs,
			Details:    tc.Details,
		}

		if tc.Status != StatusPassed && tc.Error != nil {
			errorMsg := tc.Error.Message
			op.Error = &errorMsg
		}

		operations = append(operations, op)
		totalDuration += tc.DurationMs
	}

	// Get SDK version (would need to implement proper version detection)
	sdkVersion := "unknown"
	if info, ok := debug.ReadBuildInfo(); ok {
		for _, dep := range info.Deps {
			if dep.Path == "github.com/bitwarden/sdk-go/v2" {
				sdkVersion = dep.Version
				break
			}
		}
	}

	return SimpleTestResult{
		Language:        "go",
		SDKVersion:      sdkVersion,
		Operations:      operations,
		TotalDurationMs: totalDuration,
		OS:              runtime.GOOS,
		Architecture:    runtime.GOARCH,
		Timestamp:       time.Now().UTC().Format(time.RFC3339),
	}
}

func main() {
	// Parse command line arguments
	var (
		jsonOutput  = flag.Bool("json", false, "Output results in simple JSON format for CI/CD integration")
		outputFile  = flag.String("output-file", "", "Save results to file (JSON format)")
		verbose     = flag.Bool("verbose", false, "Enable verbose output")
		testMode    = flag.String("test-mode", "", "Override TEST_MODE environment variable")
		sdkSource   = flag.String("sdk-source", "", "Override SDK_SOURCE environment variable")
	)

	flag.Parse()

	// Override environment variables if specified
	if *testMode != "" {
		os.Setenv("TEST_MODE", *testMode)
	}
	if *sdkSource != "" {
		os.Setenv("SDK_SOURCE", *sdkSource)
	}

	// Determine output format
	outputFormat := "text"
	simpleJSON := false
	if *jsonOutput {
		outputFormat = "json"
		simpleJSON = true  // --json flag means simple format for CI/CD
	} else if *outputFile != "" {
		outputFormat = "json"
		// outputFile without --json flag means comprehensive format
	}

	// Run tests
	suite := NewGoSDKTestSuite(outputFormat, *verbose)
	suite.RunAllTests()

	// Generate report
	if outputFormat == "json" || *outputFile != "" {
		var report interface{}
		if simpleJSON {
			report = suite.GenerateSimpleJSONReport()
		} else {
			report = suite.GenerateJSONReport()
		}

		jsonData, err := json.MarshalIndent(report, "", "  ")
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to marshal results: %v\n", err)
			os.Exit(1)
		}

		// Output to file if specified
		if *outputFile != "" {
			err := os.WriteFile(*outputFile, jsonData, 0644)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Failed to write output file: %v\n", err)
				os.Exit(1)
			}
			if outputFormat == "text" {
				fmt.Printf("Results saved to %s\n", *outputFile)
			}
		}

		// Output to stdout if JSON format requested
		if outputFormat == "json" {
			fmt.Println(string(jsonData))
		}
	}

	// Exit with appropriate code
	allPassed := true
	for _, tc := range suite.testCases {
		if tc.Status != StatusPassed && tc.Status != StatusSkipped {
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