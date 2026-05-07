# Adding Integration Categories

This guide explains how to implement new integration test categories (Categories 2-4) in the SDK Test Framework.

## Overview

The framework supports 4 test categories:
1. **SDK Language Wrappers** (implemented)
2. **SDK + K8s Operator** (placeholder)
3. **SDK + Terraform Provider** (placeholder)
4. **SDK + Ansible Collection** (placeholder)

## Architecture

Each integration category tests the SDK's interaction with external tools/platforms:

```
TestOrchestrator.cs
    ├── Category 1: SDK Wrappers (Python, Go, etc.)
    ├── Category 2: K8s Integration
    ├── Category 3: Terraform Integration
    └── Category 4: Ansible Integration
```

## Implementing a New Category

### Step 1: Update TestOrchestrator.cs

Replace the placeholder method with actual implementation:

```csharp
public async Task<CategoryResult> RunCategory2_K8sIntegration()
{
    Console.WriteLine("═══════════════════════════════════════════════════════════════════");
    Console.WriteLine(" Category 2: SDK + K8s Operator Integration");
    Console.WriteLine("═══════════════════════════════════════════════════════════════════");
    Console.WriteLine();

    var category = new CategoryResult
    {
        CategoryName = "SDK + K8s Operator",
        CategoryNumber = 2,
        StartTime = DateTime.UtcNow,
        LanguageResults = new List<TestResult>()
    };

    try
    {
        // 1. Check prerequisites
        if (!await CheckK8sPrerequisites())
        {
            category.Error = "K8s prerequisites not met";
            return category;
        }

        // 2. Setup test environment
        await SetupK8sEnvironment();

        // 3. Run integration tests
        var k8sTests = new K8sIntegrationTests(config);
        var results = await k8sTests.RunTests();

        // 4. Convert to TestResult format
        category.LanguageResults.Add(ConvertToTestResult(results));

        // 5. Cleanup
        await CleanupK8sEnvironment();

        // Calculate totals
        category.TotalOperations = results.Operations.Count;
        category.PassedOperations = results.Operations.Count(op => op.Success);
        category.FailedOperations = category.TotalOperations - category.PassedOperations;
    }
    catch (Exception ex)
    {
        category.Error = $"Category 2 error: {ex.Message}";
    }
    finally
    {
        category.EndTime = DateTime.UtcNow;
        category.DurationMs = (long)(category.EndTime - category.StartTime).TotalMilliseconds;
    }

    return category;
}
```

### Step 2: Create Integration Test Class

Create a new class in `tests/SdkTestFramework/Integrations/`:

```csharp
namespace SdkTestFramework.Integrations;

public class K8sIntegrationTests
{
    private readonly TestConfig config;
    private readonly ProcessRunner processRunner;
    private readonly string k8sNamespace = "sm-sdk-test";

    public K8sIntegrationTests(TestConfig config)
    {
        this.config = config;
        this.processRunner = new ProcessRunner();
    }

    public async Task<IntegrationTestResult> RunTests()
    {
        var operations = new List<TestOperation>();

        // Test 1: Deploy K8s Operator
        operations.Add(await TestDeployOperator());

        // Test 2: Create Secret via CRD
        operations.Add(await TestCreateSecretCRD());

        // Test 3: Verify Secret Sync
        operations.Add(await TestSecretSync());

        // Test 4: Update Secret
        operations.Add(await TestUpdateSecret());

        // Test 5: Delete Secret
        operations.Add(await TestDeleteSecret());

        // Test 6: Cleanup
        operations.Add(await TestCleanup());

        return new IntegrationTestResult
        {
            IntegrationType = "K8s",
            Operations = operations,
            Platform = OsDetector.GetOsName()
        };
    }

    private async Task<TestOperation> TestDeployOperator()
    {
        var op = new TestOperation
        {
            Operation = "deploy_operator",
            Success = false
        };

        try
        {
            // Apply K8s manifests
            var result = await processRunner.RunAsync(
                "kubectl",
                new[] { "apply", "-f", "k8s/operator.yaml", "-n", k8sNamespace },
                new Dictionary<string, string>(),
                30000
            );

            op.Success = result.Success;
            op.Error = result.Success ? null : result.Error;
        }
        catch (Exception ex)
        {
            op.Error = ex.Message;
        }

        return op;
    }

    // ... implement other test operations
}
```

### Step 3: Define Integration Models

Create models specific to integration testing:

```csharp
public record IntegrationTestResult
{
    public required string IntegrationType { get; init; }
    public required List<TestOperation> Operations { get; init; }
    public required string Platform { get; init; }
    public Dictionary<string, object>? Metadata { get; init; }

    public TestResult ToTestResult()
    {
        return new TestResult
        {
            Language = IntegrationType,
            SdkVersion = "integration",
            Operations = Operations,
            TotalDurationMs = Operations.Sum(op => op.DurationMs),
            Os = Platform,
            Architecture = OsDetector.GetArchitecture(),
            Timestamp = DateTime.UtcNow.ToString("O")
        };
    }
}
```

## Category-Specific Implementation Guides

### Category 2: K8s Operator Integration

**Prerequisites:**
- Kubernetes cluster (kind, minikube, or real cluster)
- kubectl installed and configured
- K8s operator manifests

**Test Operations:**
1. Deploy operator to cluster
2. Create SecretProviderClass CRD
3. Mount secrets in pod
4. Verify secret values
5. Update secret and verify propagation
6. Test rotation
7. Cleanup resources

**Directory Structure:**
```
tests/
├── Integrations/
│   └── K8s/
│       ├── manifests/
│       │   ├── operator.yaml
│       │   ├── secret-provider-class.yaml
│       │   └── test-pod.yaml
│       └── K8sIntegrationTests.cs
```

### Category 3: Terraform Provider Integration

**Prerequisites:**
- Terraform CLI installed
- Provider binary built or downloaded
- Test infrastructure configuration

**Test Operations:**
1. Initialize Terraform
2. Create secret resource
3. Create project resource
4. Reference secret in another resource
5. Update secret value
6. Import existing secret
7. Destroy resources

**Directory Structure:**
```
tests/
├── Integrations/
│   └── Terraform/
│       ├── configs/
│       │   ├── main.tf
│       │   ├── variables.tf
│       │   └── outputs.tf
│       └── TerraformIntegrationTests.cs
```

**Example Test:**
```csharp
private async Task<TestOperation> TestCreateResource()
{
    var tfConfig = @"
        resource ""bitwarden_secret"" ""test"" {
            name  = ""integration-test-secret""
            value = ""test-value""
            organization_id = var.organization_id
        }
    ";

    await File.WriteAllTextAsync("test.tf", tfConfig);

    var result = await processRunner.RunAsync(
        "terraform",
        new[] { "apply", "-auto-approve" },
        GetTerraformEnv(),
        60000
    );

    return new TestOperation
    {
        Operation = "terraform_create_secret",
        Success = result.Success,
        DurationMs = stopwatch.ElapsedMilliseconds,
        Error = result.Error
    };
}
```

### Category 4: Ansible Collection Integration

**Prerequisites:**
- Ansible installed
- Collection installed (`ansible-galaxy collection install bitwarden.secrets`)
- Inventory configured

**Test Operations:**
1. Lookup secret value
2. Create secret via module
3. Update secret properties
4. Use in playbook variable
5. Template with secret values
6. Sync secrets
7. Delete secret

**Directory Structure:**
```
tests/
├── Integrations/
│   └── Ansible/
│       ├── playbooks/
│       │   ├── test-lookup.yml
│       │   ├── test-module.yml
│       │   └── test-sync.yml
│       └── AnsibleIntegrationTests.cs
```

**Example Playbook:**
```yaml
---
- name: Test Bitwarden Secrets Lookup
  hosts: localhost
  vars:
    secret_value: "{{ lookup('bitwarden.secrets.secret', 'secret-id') }}"
  tasks:
    - name: Display secret value
      debug:
        msg: "Secret value is: {{ secret_value }}"

    - name: Create new secret
      bitwarden.secrets.secret:
        name: "ansible-test-secret"
        value: "test-value"
        organization_id: "{{ organization_id }}"
        state: present
```

## Platform Considerations

### OS-Specific Logic

```csharp
private bool ShouldRunCategory2()
{
    // K8s tests only run on macOS and Linux
    if (OsDetector.IsWindows())
    {
        Console.WriteLine("Skipping K8s tests on Windows");
        return false;
    }

    // Check if kubectl is available
    return CommandExists("kubectl");
}
```

### Environment Detection

```csharp
private async Task<bool> CheckK8sPrerequisites()
{
    // Check kubectl
    if (!await CheckCommand("kubectl", "version"))
        return false;

    // Check cluster access
    var result = await processRunner.RunAsync(
        "kubectl",
        new[] { "cluster-info" },
        new Dictionary<string, string>(),
        5000
    );

    if (!result.Success)
    {
        Console.WriteLine("No Kubernetes cluster available");
        return false;
    }

    return true;
}
```

## Configuration

### Add Category Settings

Update `test-config.json`:

```json
{
  "integrations": {
    "k8s": {
      "enabled": true,
      "namespace": "sm-sdk-test",
      "cleanup": true
    },
    "terraform": {
      "enabled": true,
      "version": "1.5.0",
      "provider_version": "0.1.0"
    },
    "ansible": {
      "enabled": true,
      "collection_version": "1.0.0",
      "inventory": "localhost,"
    }
  }
}
```

### GitHub Actions Updates

Add integration-specific setup:

```yaml
# For K8s testing
- name: Setup Kind cluster
  if: matrix.os != 'windows-latest'
  run: |
    curl -Lo ./kind https://kind.sigs.k8s.io/dl/v0.20.0/kind-$(uname)-amd64
    chmod +x ./kind
    ./kind create cluster --name sm-sdk-test

# For Terraform testing
- name: Setup Terraform
  uses: hashicorp/setup-terraform@v3
  with:
    terraform_version: '1.5.0'

# For Ansible testing
- name: Setup Ansible
  if: matrix.os != 'windows-latest'
  run: |
    pip install ansible
    ansible-galaxy collection install bitwarden.secrets
```

## Testing Strategy

### Local Development

1. **Mock Mode**: Test with mock responses first
2. **Docker Mode**: Use containerized dependencies
3. **Integration Mode**: Test with real services

### CI/CD Pipeline

1. **Conditional Execution**: Only run if dependencies available
2. **Isolated Namespaces**: Use unique namespaces/prefixes
3. **Cleanup**: Always cleanup resources
4. **Timeout Protection**: Set reasonable timeouts

## Error Handling

### Graceful Degradation

```csharp
public async Task<CategoryResult> RunCategoryWithFallback()
{
    try
    {
        if (!await CheckPrerequisites())
        {
            return CreateSkippedResult("Prerequisites not met");
        }

        return await RunActualTests();
    }
    catch (Exception ex)
    {
        return CreateErrorResult(ex);
    }
}
```

### Cleanup on Failure

```csharp
private async Task EnsureCleanup()
{
    try
    {
        await CleanupResources();
    }
    catch (Exception ex)
    {
        Console.WriteLine($"Cleanup failed: {ex.Message}");
        // Log but don't fail the test
    }
}
```

## Checklist for New Integration

- [ ] Prerequisites documented
- [ ] Test operations defined (6-7 operations)
- [ ] Integration test class created
- [ ] Models/DTOs defined
- [ ] TestOrchestrator method implemented
- [ ] Platform compatibility checked
- [ ] Configuration added to test-config.json
- [ ] GitHub Actions workflow updated
- [ ] Local testing successful
- [ ] Cleanup verified
- [ ] Documentation updated

## Resources

- [Kubernetes CRD Documentation](https://kubernetes.io/docs/concepts/extend-kubernetes/api-extension/custom-resources/)
- [Terraform Provider Development](https://developer.hashicorp.com/terraform/plugin)
- [Ansible Collection Development](https://docs.ansible.com/ansible/latest/dev_guide/developing_collections.html)