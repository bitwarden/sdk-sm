# SDK Test Framework - Complete Code Analysis

## Table of Contents
1. [Section 1: SDK Test Framework (C#)](#section-1-sdk-test-framework-c)
2. [Section 2: Test Scripts and Language Implementations](#section-2-test-scripts-and-language-implementations)
3. [Section 3: Integration - How Everything Works Together](#section-3-integration---how-everything-works-together)

---

# Section 1: SDK Test Framework (C#)

## Overview
The SDK Test Framework is a NUnit-based test orchestration system designed to run language-specific SDK tests across multiple platforms. It follows a modular architecture with clear separation of concerns between configuration, runners, models, and test execution.

## Directory Structure
```
SdkTestFramework/
├── Common/
│   └── ConsoleFormatting.cs
├── Config/
│   └── TestConfig.cs
├── Models/
│   ├── OsContext.cs
│   ├── TestOperation.cs
│   └── TestResult.cs
└── Runners/
    ├── BaseRunner.cs
    ├── FakeServerManager.cs
    ├── GoRunner.cs
    ├── OsDetector.cs
    ├── ProcessRunner.cs
    └── PythonRunner.cs

SdkTestFramework.Tests/
├── ConfigurationService.cs
├── Global.cs
├── TestBase.cs
└── SdkWrappers/
    ├── GoTests.cs
    ├── PythonTests.cs
    ├── SdkWrappersDataSetup.cs
    └── SdkWrappersTestBase.cs
```

---

## SdkTestFramework Components

### Common/ConsoleFormatting.cs
**Purpose**: Provides centralized console formatting utilities for consistent output presentation.

**Why Needed**: Ensures consistent visual presentation across the framework, making output more readable.

**Classes**:
- `ConsoleFormatting` (static class)

**Properties**:
- `BoxTop` - Top border for boxed headers
- `BoxBottom` - Bottom border for boxed headers
- `BoxSide` - Side border character (private)
- `BoxWidth` - Total width including borders (private)
- `LineSeparator` - Full-width separator line
- `DashedLine` - Dashed separator line

**Methods**:
- `CreateBoxedHeader(string text)` - Creates centered text within box borders
  - Calculates padding to center text
  - Returns formatted string with borders
- `PrintBoxedHeader(string title)` - Prints complete boxed header to console
  - Uses BoxTop, CreateBoxedHeader, and BoxBottom
  - Adds extra newline for spacing

**Improvements Needed**: None - All properties and methods are actively used.

---

### Config/TestConfig.cs
**Purpose**: Strongly-typed configuration management for the test framework.

**Why Needed**: Provides type-safe access to configuration values from JSON and environment variables.

**Classes**:
- `TestConfig` (record) - Main configuration container
- `TestFrameworkInfo` (record) - Framework metadata
- `ConfigurationSettings` (record) - Test execution settings
- `TimeoutSettings` (record) - Timeout configurations

**TestConfig Methods**:
- `LoadFromConfiguration(IConfiguration)` - Loads config from IConfiguration
  - Maps configuration sections to strongly-typed objects
  - Sets defaults for missing values
  - **Used by**: Global.cs for initial setup

**ConfigurationSettings Methods**:
- `IsFakeServerMode()` - Checks if test mode is "fake-server"
  - **Used by**: Global.cs and FakeServerManager.cs

**Properties Analysis**:
- ✅ `TestFramework.Name` - Used for logging
- ✅ `TestFramework.Version` - Used for logging
- ✅ `Configuration.TestMode` - Used via IsFakeServerMode()
- ✅ `Configuration.SdkSource` - Passed to test scripts via environment
- ✅ `Configuration.BuildSdk` - Used in BaseRunner for timeout selection
- ✅ `Configuration.AutoStartFakeServer` - Used by FakeServerManager
- ✅ `Configuration.FakeServerPort` - Used by FakeServerManager
- ✅ `Configuration.PythonVersion` - Passed to Python test scripts
- ✅ `Configuration.EnabledLanguages` - Used to enable/disable language tests
- ✅ `Timeouts.DefaultTimeoutMs` - Used by BaseRunner for test execution
- ✅ `Timeouts.BuildTimeoutMs` - Used when BUILD_SDK is true
- ✅ `TestOperations` - List of operations to test

**Improvements Needed**: None - All unused code has been removed

---

### Models/OsContext.cs
**Purpose**: Represents operating system context information for platform-specific behavior.

**Why Needed**: Enables platform-specific logic and reporting of test environment details.

**Classes**:
- `OsContext` (record) - OS information container
- `OperatingSystemType` (enum) - OS type enumeration

**Properties**:
- ✅ `OsType` - Used for determining platform-specific behavior
- ✅ `OsDisplayName` - Used in test result reporting
- ✅ `Architecture` - Used in test result reporting
- ✅ `Version` - NOW USED in test result reporting (added os_version field)
- ✅ `IsWindows` - Used for Windows-specific logic
- ✅ `IsMacOs` - Set for future platform-specific logic
- ✅ `IsLinux` - Set for future platform-specific logic
- ✅ `IsArm` - Used by IsWindowsArm()

**Methods**:
- `IsWindowsArm()` - Checks if running on Windows ARM
  - **Used by**: OsDetector to show unsupported platform warning
  - Returns true if both IsWindows and IsArm are true

**Improvements Needed**: None - All properties now have clear purposes

---

### Models/TestOperation.cs
**Purpose**: Represents a single test operation result with timing and error information.

**Why Needed**: Provides structured data for individual test operations within a test suite run.

**Classes**:
- `TestOperation` (record)

**Properties** (all actively used):
- ✅ `Operation` - Name of the test operation
- ✅ `Success` - Whether the operation succeeded
- ✅ `DurationMs` - Execution time in milliseconds
- ✅ `Error` - Error message if failed (nullable)
- ✅ `Details` - Additional operation details (Dictionary)

**Methods**:
- `IsSuccessful()` - Returns the Success property
  - **Used by**: TestResult methods for counting passed/failed operations

**Improvements Needed**: None - All properties and methods are actively used.

---

### Models/TestResult.cs
**Purpose**: Aggregates test results from a complete language SDK test run.

**Why Needed**: Provides structured output from test runners that can be consumed by the test framework.

**Classes**:
- `TestResult` (record)

**Properties** (all actively used):
- ✅ `Language` - SDK language being tested
- ✅ `SdkVersion` - Version of the SDK being tested
- ✅ `Operations` - List of test operations performed
- ✅ `TotalDurationMs` - Total execution time
- ✅ `OperatingSystem` - OS where tests ran
- ✅ `Architecture` - CPU architecture
- ✅ `OperatingSystemVersion` - OS version (newly added)
- ✅ `Timestamp` - When tests were executed

**Methods** (all actively used):
- ✅ `PassedCount()` - Counts successful operations
- ✅ `FailedCount()` - Counts failed operations
- ✅ `AllPassed()` - Checks if all operations passed
- ✅ `SuccessRate()` - Calculates percentage of passed tests

**Improvements Needed**: None - All properties and methods are actively used.

---

### Runners/BaseRunner.cs
**Purpose**: Abstract base class providing common functionality for all language-specific test runners.

**Why Needed**: Implements shared logic for test execution, environment setup, and result parsing.

**Abstract Methods** (must be implemented by derived classes):
- `Language` - Language name for reporting
- `IsSupportedOnCurrentPlatform()` - Platform compatibility check
- `GetTestScriptPath()` - Path to language test script
- `GetExecuteCommand()` - Command to run tests

**Virtual Methods**:
- `GetExecuteArguments()` - Arguments for test command (default: script path + "--json")
- `VerifyPrerequisites()` - Checks bash and test script existence
- `GetWorkingDirectory()` - Working directory for test execution (default: null)

**Protected Methods**:
- `CheckBashInstalled()` - Verifies bash is available
  - Runs `bash --version` to check installation
- `GetSdkBasePath()` - Finds SDK root directory
  - Looks for Cargo.toml and languages directory

**Private Methods**:
- `ExecuteTests()` - Main test execution logic
  - Sets timeout based on BuildSdk flag
  - Runs process and captures output
  - Parses JSON results
- `ParseTestOutput()` - Deserializes JSON test results
- `LoadEnvironmentVariables()` - Prepares environment for child processes
- `AddEnvironmentVariable()` - Helper to add env vars to dictionary
- `CreateBaseResult()` - Creates TestResult with OS info
- `CreateUnsupportedResult()` - Result for unsupported platforms
- `CreatePrerequisiteFailureResult()` - Result when prerequisites fail
- `CreateExecutionFailureResult()` - Result for execution failures
- `CreateExceptionResult()` - Result for exceptions
- `CreateParseFailureResult()` - Result for JSON parsing errors

**Key Features**:
- Uses ProcessRunner for process execution (composition)
- Supports BUILD_SDK flag for longer timeouts during builds
- Handles environment variable propagation
- Provides comprehensive error reporting

**Improvements Needed**: None - All methods serve specific purposes.

---

### Runners/ProcessRunner.cs
**Purpose**: Handles external process execution with timeout, output capture, and real-time streaming.

**Why Needed**: Provides reliable process execution with proper error handling and output management.

**Classes**:
- `ProcessRunner` - Main process execution class
- `ProcessResult` (record) - Process execution result

**ProcessRunner Methods**:
- `RunAsync()` - Main execution method
  - Creates process with configuration
  - Streams output in real-time
  - Handles timeout with cancellation
  - Returns structured result
- `CreateProcess()` (static, private) - Configures ProcessStartInfo
- `AddEnvironmentVariables()` (static, private) - Manages environment variables
  - **Important**: Copies current environment before adding test variables
- `WaitForExitAsync()` (static, private) - Waits with timeout
- `KillProcess()` (static, private) - Terminates process tree

**ProcessResult Properties**:
- ✅ `ExitCode` - Process exit code
- ✅ `Output` - Captured stdout
- ✅ `Error` - Captured stderr
- ✅ `DurationMs` - Execution duration
- ✅ `Success` - Whether exit code was 0

**ProcessResult Methods**:
- `Timeout()` (static) - Creates timeout result

**Improvements Needed**: None - All unused code has been removed

---

### Runners/PythonRunner.cs
**Purpose**: Implements Python-specific test runner logic.

**Why Needed**: Handles Python SDK test execution with proper script paths and arguments.

**Overridden Methods**:
- `Language` - Returns "python"
- `IsSupportedOnCurrentPlatform()` - Returns true (Python works everywhere)
- `GetTestScriptPath()` - Returns path to languages/python/test.sh
- `GetExecuteCommand()` - Returns "bash"
- `GetExecuteArguments()` - Adds --no-build flag if needed
- `GetWorkingDirectory()` - Returns SDK base path

**Improvements Needed**: None - Clean implementation with no duplicates

---

### Runners/GoRunner.cs
**Purpose**: Implements Go-specific test runner logic.

**Why Needed**: Handles Go SDK test execution with proper script paths and arguments.

**Overridden Methods**:
- `Language` - Returns "go"
- `IsSupportedOnCurrentPlatform()` - Returns true (Go works everywhere)
- `GetTestScriptPath()` - Returns path to languages/go/test.sh
- `GetExecuteCommand()` - Returns "bash"
- `GetExecuteArguments()` - Adds --no-build flag if needed

**Improvements Needed**: None - Clean implementation with no duplicates.

---

### Runners/OsDetector.cs
**Purpose**: Detects current operating system and architecture information.

**Why Needed**: Provides platform detection for conditional logic and reporting.

**Classes**:
- `OsDetector` (static class)

**Methods**:
- `GetCurrentOsContext()` - Main detection method
  - Uses RuntimeInformation for platform detection
  - Maps architecture strings to display names
  - Sets all boolean flags for convenience
- `GetOsType()` (private) - Maps RuntimeInformation to enum
- `GetOsDisplayName()` (private) - Creates human-readable OS name
- `GetArchitectureDisplayName()` (private) - Maps architecture to string
- `IsArmArchitecture()` (private) - Detects ARM processors
- `PrintOsContext()` - Outputs OS info to console
  - Shows warning for Windows ARM (unsupported)

**Improvements Needed**: None - All methods serve specific purposes.

---

### Runners/FakeServerManager.cs
**Purpose**: Manages the lifecycle of the fake-server process for local testing.

**Why Needed**: Automates fake-server startup/shutdown for test isolation without external dependencies.

**Classes**:
- `FakeServerManager` (sealed, implements IDisposable)

**Public Methods**:
- `StartIfNeeded()` - Conditionally starts fake-server
  - Checks if already running
  - Only starts in fake-server mode with auto-start enabled
- `Dispose()` - Cleanup method

**Private Methods**:
- `StartFakeServer()` - Starts the fake-server process
  - Attempts to build if executable missing
  - Provides detailed error messages
- `IsServerRunning()` - Checks if server responds on port
- `BuildFakeServerAsync()` - Builds fake-server with cargo
  - Provides helpful messages for missing dependencies
- `GetSdkRootPath()` - Finds SDK root directory
- `GetFakeServerExecutablePath()` - Constructs path to executable
- `Stop()` - Terminates fake-server process

**Key Features**:
- Auto-builds fake-server if missing
- Detailed error messages with solutions
- Checks for existing server before starting
- Platform-aware executable naming (.exe on Windows)

**Improvements Needed**: None - Comprehensive error handling and user guidance.

---

## SdkTestFramework.Tests Components

### ConfigurationService.cs
**Purpose**: Manages configuration loading from JSON and environment variables.

**Why Needed**: Provides centralized configuration access following Bitwarden patterns.

**Methods**:
- `Initialize()` - Loads configuration from JSON and .env
  - Loads from Configuration/.env if exists
  - Adds test-config.json
  - Adds environment variables
- `GetValue()` - Gets single configuration value
- `GetSection<T>()` - Gets typed configuration section

**Properties**:
- `Configuration` - Lazy-loaded IConfigurationRoot

**Improvements Needed**: None - Simple and effective configuration management.

---

### Global.cs
**Purpose**: NUnit SetUpFixture providing one-time global setup/teardown for all tests.

**Why Needed**: Initializes framework-wide resources like configuration and fake-server.

**Methods**:
- `Global_SetUp()` - One-time initialization
  - Loads configuration
  - Starts fake-server if needed
  - Validates environment variables
  - Auto-generates STATE_FILE for test isolation
- `Global_TearDown()` - One-time cleanup
  - Disposes fake-server
- `GetTestConfig()` - Provides access to loaded configuration
- `AddCommandLineParametersAsync()` - Adds NUnit parameters to environment
- `ValidateRequiredVariablesAsync()` - Ensures required env vars exist

**Key Features**:
- Automatic STATE_FILE generation for test isolation
- Command-line parameter support
- Comprehensive setup validation

**Improvements Needed**: None - Robust initialization logic.

---

### TestBase.cs
**Purpose**: Base class for all test fixtures providing common setup/teardown logic.

**Why Needed**: Ensures consistent test initialization and reporting across all test classes.

**Properties**:
- `TestConfig` - Access to test configuration
- `ProcessRunner` - Shared ProcessRunner instance

**Virtual Methods** (can be overridden):
- `TestBase_OneTimeSetUp()` - Class-level setup
- `TestBase_SetUp()` - Per-test setup
- `TestBase_TearDown()` - Per-test cleanup with result logging
- `TestBase_OneTimeTearDown()` - Class-level cleanup

**Protected Methods**:
- `IsLanguageEnabled()` - Checks if language is in EnabledLanguages

**Improvements Needed**: None - Clean base class implementation.

---

### SdkWrappers/SdkWrappersTestBase.cs
**Purpose**: Base class for SDK-specific test fixtures.

**Why Needed**: Provides common functionality for language SDK tests.

**Methods**:
- `TestBase_OneTimeSetUp()` (override) - Validates environment
- `ValidateEnvironmentVariables()` - Ensures SDK env vars are set
- `CreateAndInitializePythonRunner()` - Factory for Python runner
  - Checks if Python enabled
  - Verifies prerequisites
  - Returns configured runner
- `CreateAndInitializeGoRunner()` - Factory for Go runner
  - Similar to Python runner creation

**Improvements Needed**: None - Good abstraction for SDK tests.

---

### SdkWrappers/SdkWrappersDataSetup.cs
**Purpose**: NUnit SetUpFixture for SDK wrapper test data initialization.

**Why Needed**: Placeholder for future SDK-specific data setup/cleanup.

**Methods**:
- `SetUp()` - One-time data initialization (currently empty)
- `TearDown()` - One-time data cleanup (currently empty)

**Status**: Currently provides no functionality but structure is in place for future needs.

**Improvements Needed**: None - Ready for future data setup requirements.

---

### SdkWrappers/PythonTests.cs
**Purpose**: NUnit test fixture for Python SDK tests.

**Why Needed**: Orchestrates Python SDK test execution.

**Methods**:
- `SetUp()` - Initializes Python runner
- `Python_SDK_Tests()` - Main test method
  - Runs Python tests via runner
  - Reports results
  - Asserts all operations passed

**Improvements Needed**: None - Clean test implementation.

---

### SdkWrappers/GoTests.cs
**Purpose**: NUnit test fixture for Go SDK tests.

**Why Needed**: Orchestrates Go SDK test execution.

**Methods**:
- `SetUp()` - Initializes Go runner
- `Go_SDK_Tests()` - Main test method
  - Runs Go tests via runner
  - Reports results
  - Asserts all operations passed

**Improvements Needed**: None - Clean test implementation.

---

---

# Section 2: Test Scripts and Language Implementations

## Scripts Directory Structure
```
scripts/
├── test-utils.sh           # Shared utilities for all language test scripts
├── build-from-main.sh      # Builds SDK from main branch
└── aggregate-test-results.py # Aggregates test results (not analyzed here)

languages/python/
├── test.sh                 # Python test runner shell script
└── test/
    └── test_suite.py       # Python test implementation

languages/go/
├── test.sh                 # Go test runner shell script
└── test/
    └── test_suite.go       # Go test implementation
```

## Shared Scripts

### scripts/test-utils.sh
**Purpose**: Provides common utilities for all language-specific test scripts.

**Why Needed**: Centralizes shared functionality to avoid duplication across language test scripts.

**Key Functions**:

#### Logging Functions
- `log_info()` - Logs informational messages (suppressed in JSON mode)
- `log_success()` - Logs success messages with green checkmark
- `log_error()` - Logs error messages to stderr with red X
- `log_warning()` - Logs warning messages with yellow exclamation
- `disable_colors_if_json()` - Disables color codes when JSON output is enabled

#### Environment Management
- `load_test_environment($repo_root)` - Main environment loader
  - Loads .env file from test framework configuration directory
  - Sources environment variables (ACCESS_TOKEN, ORGANIZATION_ID, etc.)
  - Exports variables for child processes
  - Reads test-config.json for TEST_MODE and SDK_SOURCE
  - Uses jq if available, falls back to grep/sed

#### Authentication
- `authenticate_real_server()` - Handles real Bitwarden server authentication
  - Uses BWS_CLIENT_ID and BWS_CLIENT_SECRET
  - Creates access token in format: `0.{client_id}.{client_secret}`
  - Optionally validates with bws CLI if available

- `handle_authentication()` - Routes authentication based on TEST_MODE
  - For "real-server": calls authenticate_real_server()
  - For "fake-server": verifies ACCESS_TOKEN from .env exists

#### Fake Server Management
- `start_fake_server_if_needed($repo_root)` - Manages fake server lifecycle
  - Checks AUTO_START_FAKE_SERVER configuration
  - Verifies if server already running on port
  - Builds fake-server with cargo if needed
  - Starts server and waits for health check
  - Returns PID for cleanup

#### SDK Building
- `build_sdk_from_main($language, $repo_root, $verbose)` - Builds SDK from main branch
  - Calls centralized build-from-main.sh script
  - Supports verbose output option
  - Language-specific build logic

**Environment Variables Used**:
- Input: SDK_TEST_ENV, SDK_TEST_CONFIG, BWS_CLIENT_ID, BWS_CLIENT_SECRET
- Output: ACCESS_TOKEN, ORGANIZATION_ID, API_URL, IDENTITY_URL, STATE_FILE, TEST_MODE, SDK_SOURCE

---

## Python Implementation

### languages/python/test.sh
**Purpose**: Shell script orchestrator for Python SDK tests.

**Command Line Arguments**:
- `--json` - Output JSON format for CI/CD
- `--no-build` - Skip SDK build step
- `--sdk-source TYPE` - SDK source: local|main
- `--output-file FILE` - Save results to file
- `--verbose` - Enable verbose output
- `--python VERSION` - Specify Python version
- `--help` - Show usage

**Key Functions**:

#### Setup and Requirements
- `check_requirements()` - Validates environment
  - Checks python3 is installed
  - Checks uv is installed (only if building)
  - Verifies schemas.py exists

- `setup_python_environment($python_version)` - Creates virtual environment
  - Uses uv to create venv in temp directory
  - Activates virtual environment
  - Upgrades pip to latest

- `source_venv($python_version)` - Cross-platform venv activation
  - Handles Unix and Windows activation scripts

#### Building
- `build_package($python_version)` - Builds Python SDK
  - Calls build_sdk_from_main if SDK_SOURCE=main
  - Installs maturin and dependencies via uv
  - Platform-specific handling (Linux requires patchelf)
  - Builds Rust extension with maturin develop
  - Redirects output to stderr in JSON mode

#### Test Execution
- `run_tests($python_version)` - Executes Python tests
  - Activates venv if BUILD_SDK=true
  - Starts fake server if TEST_MODE=fake-server
  - Handles authentication
  - Runs test_suite.py with appropriate flags
  - Returns test exit code

#### Main Flow
- `main()` - Orchestrates test execution
  - Sets up cleanup trap
  - Checks requirements
  - Loops through PYTHON_VERSIONS
  - For each version: setup → build → test
  - Tracks overall success/failure
  - Exits with appropriate code

**Key Features**:
- Multi-Python version support
- Temporary directory cleanup
- JSON output for CI/CD integration
- Build caching (reuses venv if exists)

---

### languages/python/test/test_suite.py
**Purpose**: Python implementation of SDK tests matching C# framework's JSON contract.

**Classes**:
- `PythonSDKTestSuite` - Main test suite class
  - Manages test execution and result collection
  - Handles both text and JSON output formats
  - Tracks created resources for cleanup

**Key Methods**:

#### Setup and Configuration
- `__init__()` - Initializes test suite
  - Loads environment variables
  - Sets test mode and SDK source
  - Initializes bitwarden_sdk client

- `setup_client()` - Configures Bitwarden client
  - Uses API_URL and IDENTITY_URL from environment
  - Handles state file if provided

#### Test Operations (all return success, details, error)
- `test_auth()` - Access token authentication
- `test_secret_create()` - Creates test secret with random name
- `test_secret_list()` - Lists secrets in organization
- `test_secret_get()` - Gets secret (fake-server returns "btw")
- `test_secret_update()` - Updates secret key and value
- `test_secret_get_by_ids()` - Batch secret retrieval
- `test_secret_sync()` - Tests sync with/without date
- `test_secret_delete()` - Deletes secrets
- `test_project_create()` - Creates test project
- `test_project_list()` - Lists projects
- `test_project_get()` - Gets project
- `test_project_update()` - Updates project name
- `test_project_delete()` - Deletes projects
- `test_generator_default()` - Password generation with defaults
- `test_generator_custom_params()` - Custom password parameters
- `test_generator_validation_errors()` - Tests error handling

#### Test Execution
- `run_test()` - Executes individual test with timing and error handling
  - Captures start/end time
  - Handles exceptions
  - Formats output based on mode

- `run_all_tests()` - Orchestrates all tests
  - Runs tests in defined order
  - Collects results
  - Handles cleanup

#### Output Generation
- `generate_json_output()` - Creates JSON matching C# contract
  ```json
  {
    "language": "python",
    "sdk_version": "version",
    "operations": [...],
    "total_duration_ms": 100,
    "os": "darwin",
    "architecture": "arm64",
    "timestamp": "ISO8601"
  }
  ```

- `print_summary()` - Text output for human consumption

**Exit Behavior**:
- JSON mode: Always exits 0, success/failure in JSON
- Text mode: Exit 0 if all pass, 1 if any fail

---

## Go Implementation

### languages/go/test.sh
**Purpose**: Shell script orchestrator for Go SDK tests.

**Command Line Arguments**: (same as Python)
- `--json`, `--no-build`, `--sdk-source`, `--output-file`, `--verbose`, `--help`

**Key Functions**:

#### Setup and Requirements
- `check_requirements()` - Validates environment
  - Checks go is installed
  - Checks cargo if BUILD_SDK=true

- `setup_library_paths()` - Configures library paths
  - Detects platform (darwin/linux/windows)
  - Detects architecture (x64/arm64)
  - Creates lib directory structure
  - Runs setup.sh to link libraries
  - Sets DYLD_LIBRARY_PATH (macOS), LD_LIBRARY_PATH (Linux), or PATH (Windows)

#### Building
- `build_sdk()` - Builds Go SDK
  - Calls build_sdk_from_main if SDK_SOURCE=main
  - Otherwise builds bitwarden-c library locally
  - Runs cargo build -p bitwarden-c --release
  - Links libraries via setup.sh
  - Updates Go dependencies with go mod tidy

#### Test Execution
- `run_tests()` - Executes Go tests
  - Starts fake server if needed
  - Handles authentication
  - Changes to test directory
  - Runs test_suite.go with flags
  - Returns test exit code

**Key Differences from Python**:
- Library path management for C bindings
- Platform-specific library handling
- No virtual environment (Go uses modules)

---

### languages/go/test/test_suite.go
**Purpose**: Go implementation of SDK tests with comprehensive and simple JSON output.

**Structures**:

#### Simple Format (for C# framework)
- `SimpleTestOperation` - Individual test result
- `SimpleTestResult` - Complete test result matching C# contract

#### Comprehensive Format (detailed reporting)
- `TestCase` - Detailed test case with stack traces
- `TestSummary` - Summary statistics
- `TestResults` - Categorized results
- `TestExecution` - Execution metadata
- `Environment` - System information
- `BuildInfo` - Build details
- `ComprehensiveTestResult` - Full test report

**Main Class**:
- `GoSDKTestSuite` - Test suite implementation
  - Fields: client, organizationID, testMode, logs, testCases
  - Tracks created resources for cleanup

**Key Methods**:

#### Logging
- `logStdout()`, `logStderr()`, `logDebug()` - Formatted logging
  - Timestamps all log entries
  - Suppresses output in JSON mode
  - Debug only in verbose mode

#### Test Runner
- `runTest()` - Generic test executor
  - Captures panics with recover()
  - Times execution
  - Creates TestCase with status
  - Handles stack traces if verbose

#### Test Operations (same as Python)
- All test methods follow same pattern as Python
- Return (success bool, details map, error)
- Handle fake-server vs real-server modes

#### Report Generation
- `GenerateSimpleJSONReport()` - C# framework format
  - Converts TestCases to SimpleTestOperations
  - Matches exact JSON structure expected

- `GenerateJSONReport()` - Comprehensive format
  - Includes environment info
  - Git commit/branch detection
  - System resource information

#### Helper Functions
- `containsAny()` - Check string contains chars
- `countChars()` - Count character occurrences
- `getEnvOrDefault()` - Environment with fallback

**Main Function**:
- Parses command line flags
- Determines output format (text/json/simple)
- Runs test suite
- Outputs to file and/or stdout
- Exits with appropriate code

---

# Section 3: Integration - How Everything Works Together

## Test Execution Flow

### 1. Entry Point: NUnit Test Runner
When a developer runs tests (via IDE or `dotnet test`), NUnit discovers test classes in `SdkTestFramework.Tests`:

```
dotnet test --filter "Name=Python_SDK_Tests"
```

### 2. Framework Initialization Sequence

#### Phase 1: Global Setup (Global.cs)
```csharp
[OneTimeSetUp] Global_SetUp()
├── Load configuration from test-config.json
├── Initialize ConfigurationService
├── Load .env file
├── Start FakeServerManager (if configured)
└── Validate environment variables
```

#### Phase 2: Test Class Setup (TestBase.cs → SdkWrappersTestBase.cs)
```csharp
[OneTimeSetUp] TestBase_OneTimeSetUp()
├── Get TestConfig from Global
├── Create ProcessRunner instance
└── Validate SDK-specific environment variables
```

#### Phase 3: Language Runner Creation (PythonTests.cs/GoTests.cs)
```csharp
[OneTimeSetUp] SetUp()
├── CreateAndInitializePythonRunner() or CreateAndInitializeGoRunner()
├── Verify prerequisites (bash, test scripts)
└── Return configured runner or skip tests
```

### 3. Test Execution Pipeline

#### Step 1: C# Test Method Invocation
```csharp
[Test] Python_SDK_Tests() / Go_SDK_Tests()
└── await runner.RunTests()
```

#### Step 2: BaseRunner.RunTests() Orchestration
```csharp
RunTests()
├── IsSupportedOnCurrentPlatform() - Check OS compatibility
├── VerifyPrerequisites() - Check bash and scripts exist
└── ExecuteTests() - Main execution logic
```

#### Step 3: Process Execution via ProcessRunner
```csharp
ExecuteTests()
├── GetExecuteCommand() - Returns "bash"
├── GetExecuteArguments() - Returns ["test.sh", "--json", "--no-build"]
├── LoadEnvironmentVariables() - Prepares env for child process
├── ProcessRunner.RunAsync()
│   ├── Creates Process with environment
│   ├── Streams output in real-time
│   ├── Handles timeout (300s default, 500s if building)
│   └── Returns ProcessResult
└── ParseTestOutput() - Deserialize JSON to TestResult
```

### 4. Shell Script Execution

#### Step 1: test.sh Entry
```bash
languages/{python|go}/test.sh --json [--no-build]
├── source test-utils.sh
├── load_test_environment()
├── check_requirements()
├── setup environment (venv for Python, lib paths for Go)
├── build_package() or build_sdk() (unless --no-build)
└── run_tests()
```

#### Step 2: Environment Setup (test-utils.sh)
```bash
load_test_environment()
├── Read .env file → ACCESS_TOKEN, ORGANIZATION_ID, etc.
├── Read test-config.json → TEST_MODE, SDK_SOURCE
├── Export variables for child processes
└── Return to language test.sh
```

#### Step 3: Test Script Execution
```bash
run_tests()
├── start_fake_server_if_needed() (if TEST_MODE=fake-server)
├── handle_authentication()
├── Execute language test file:
│   Python: python3 test_suite.py --json
│   Go: go run test_suite.go --json
└── Return exit code
```

### 5. Language Test Implementation

#### Test Suite Execution (test_suite.py/test_suite.go)
```
main()
├── Parse command line arguments
├── Create test suite instance
├── SetupClient() - Initialize Bitwarden SDK client
├── RunAllTests()
│   ├── For each test operation:
│   │   ├── Call test method (test_auth, test_secret_create, etc.)
│   │   ├── Capture timing and results
│   │   └── Add to operations list
│   └── Generate JSON output
└── Exit with appropriate code
```

### 6. Result Processing

#### JSON Contract (returned to C#)
```json
{
  "language": "python",
  "sdk_version": "2.0.0",
  "operations": [
    {
      "operation": "test_auth",
      "success": true,
      "duration_ms": 3,
      "error": null,
      "details": {
        "method": "access_token",
        "state_file": true
      }
    }
  ],
  "total_duration_ms": 100,
  "os": "darwin",
  "architecture": "arm64",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### C# Result Processing
```csharp
ParseTestOutput(output)
├── Deserialize JSON to TestResult
├── Check all operations passed
├── Log failed operations if any
└── Assert.That(result.AllPassed())
```

## Key Integration Points

### 1. Environment Variable Flow
```
.env file → C# ConfigurationService → Environment.SetEnvironmentVariable()
→ ProcessRunner environment dictionary → Shell process environment
→ test-utils.sh sources and exports → Language test scripts
```

### 2. Configuration Hierarchy
```
test-config.json (static config)
├── TEST_MODE, SDK_SOURCE, BUILD_SDK, timeouts
├── Loaded by C# and shell scripts
└── Overrides via command line flags

.env file (secrets)
├── ACCESS_TOKEN, ORGANIZATION_ID, API URLs
├── Loaded by C# and test-utils.sh
└── Exported to all child processes
```

### 3. Process Communication
```
C# Framework ←→ Shell Scripts ←→ Language Tests
     ↓               ↓                ↓
ProcessRunner   test-utils.sh    SDK Client
     ↓               ↓                ↓
  stdout/err    Environment      API Calls
     ↓               ↓                ↓
  JSON Result   Fake Server     Test Results
```

### 4. Error Handling Chain
```
Language Test Error
└── Caught in test_suite.py/go
    └── Included in JSON output
        └── Shell script returns exit code
            └── ProcessRunner captures stderr
                └── BaseRunner creates TestResult
                    └── NUnit assertion fails
                        └── Test marked as failed
```

## Platform-Specific Considerations

### Windows
- Requires Git Bash, WSL, or similar for bash scripts
- ProcessRunner handles .exe extension for fake-server
- Python venv uses Scripts/ instead of bin/

### macOS
- Uses DYLD_LIBRARY_PATH for Go SDK
- Native bash available
- ARM64 detection for M1/M2 chips

### Linux
- Uses LD_LIBRARY_PATH for Go SDK
- Requires patchelf for Python builds
- Native bash available

## Test Modes

### Fake Server Mode (default)
1. C# starts FakeServerManager if AUTO_START_FAKE_SERVER=true
2. Or test-utils.sh starts fake server if not running
3. Tests use hardcoded responses from fake server
4. No external dependencies required

### Real Server Mode
1. Requires BWS_CLIENT_ID and BWS_CLIENT_SECRET
2. test-utils.sh creates access token dynamically
3. Tests run against actual Bitwarden server
4. Results depend on real server state

## Build Modes

### Local Build (default)
- Uses current repository code
- Builds Rust extensions with maturin (Python) or cargo (Go)
- Faster for development

### Main Branch Build
- SDK_SOURCE=main triggers build-from-main.sh
- Fetches latest main branch
- Builds from main for regression testing

## Timeout Management
```
C# BaseRunner determines timeout:
├── BUILD_SDK=true → BuildTimeoutMs (500s)
└── BUILD_SDK=false → DefaultTimeoutMs (300s)
    └── Passed to ProcessRunner.RunAsync()
        └── Process killed if exceeds timeout
```

## JSON Output Modes

### Simple Format (--json flag)
- Used by C# test framework
- Minimal structure for pass/fail determination
- Always exits 0, status in JSON

### Comprehensive Format (--output-file without --json)
- Detailed test information
- Stack traces, environment details
- Used for debugging and reporting

---

## Summary of Improvements Needed

All high-priority improvements have been completed! The codebase is now clean with no duplicate or unused code.

### Already Completed
✅ **TestConfig.cs** - Removed unused code:
   - `LoadAsync()` method
   - `GetJsonOptions()` method
   - `RetrySettings` record and all related properties

✅ **ProcessRunner.cs** - Removed unused code:
   - `AppendLine()` method - Was redundant helper for StringBuilder

✅ **PythonRunner.cs** - Removed duplicate methods:
   - `CreateExecutionFailureResult()` - Duplicate of BaseRunner method
   - `CreateExceptionResult()` - Duplicate of BaseRunner method
   - `CreateBaseResult()` - Duplicate of BaseRunner method
   - `ParseTestOutput()` - Duplicate of BaseRunner method
   - Removed unnecessary using statements

### Recently Completed Improvements
✅ **OsContext.cs** - All properties now actively used:
   - `Version` - Added to test reports as `os_version` field
   - `IsMacOs` and `IsLinux` - Ready for platform-specific logic

✅ **PythonRunner.cs** - Cleaned up redundant code:
   - Removed unnecessary ternary operator returning same value

✅ **Platform Documentation** - Created PLATFORM_CONSIDERATIONS.md:
   - Comprehensive guide for platform differences
   - Troubleshooting guide for each OS
   - CI/CD recommendations

### Low Priority (Keep for Future Use)
1. **SdkWrappersDataSetup.cs** - Empty but provides structure for future data setup

## Architecture Strengths
1. **Clear Separation of Concerns** - Models, Runners, Config, and Tests are properly separated
2. **Composition over Inheritance** - ProcessRunner is injected, not inherited
3. **Platform Awareness** - Proper OS detection and platform-specific handling
4. **Error Handling** - Comprehensive error messages with actionable solutions
5. **Test Isolation** - Automatic STATE_FILE generation prevents test interference
6. **Extensibility** - Easy to add new language runners by extending BaseRunner

## Code Quality Assessment
- **Overall**: Well-structured, maintainable code
- **Naming**: Clear, descriptive names following C# conventions
- **Documentation**: Good XML documentation on public members
- **Error Handling**: Comprehensive with helpful messages
- **SOLID Principles**: Generally well-followed
- **Main Issue**: Some code duplication in PythonRunner that should be removed