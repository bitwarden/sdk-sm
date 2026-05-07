# Test Framework Structure

## Directory Layout

```
tests/
├── env.example                    # Environment template
├── test-config.json               # Test configuration
├── .env                          # Actual environment vars (gitignored)
├── SdkTestFramework/
│   ├── SdkTestFramework.csproj
│   ├── Models/
│   │   ├── TestOperation.cs
│   │   ├── SmokeTestResult.cs
│   │   ├── AggregatedTestReport.cs
│   │   └── OsContext.cs
│   ├── Orchestration/
│   │   ├── TestOrchestrator.cs
│   │   └── TestReporter.cs
│   ├── Runners/
│   │   ├── BaseSmokeRunner.cs
│   │   ├── PythonSmokeRunner.cs
│   │   ├── GoSmokeRunner.cs
│   │   ├── ProcessRunner.cs
│   │   └── OsDetector.cs
│   └── Config/
│       ├── TestConfig.cs
│       └── EnvironmentValidator.cs
├── SdkTests/
│   ├── SdkTests.csproj
│   └── Integration/
│       └── UnifiedTestSuite.cs
└── IntegrationTests/             # Placeholder for future K8s, Terraform, Ansible
```

## Language Test Files

```
languages/
├── python/
│   └── test/
│       └── smoke_tests.py       # 6 operations: auth, create, list, get, delete, sync
└── go/
    └── test/
        └── smoke_tests.go        # 6 operations: auth, create, list, get, delete, sync
```

## Test Operations

Each language implements these operations:
1. **auth** - Login with access token
2. **create_secret** - Create a new secret
3. **list_secrets** - List all secrets
4. **get_secret** - Get a specific secret (tests fake-server's "btw" value)
5. **delete_secret** - Delete a secret
6. **sync** - Test sync functionality (with and without date)