# Coding Standards for SDK Test Framework

## Core Philosophy
Write code that is **maintainable**, **readable**, **testable**, and **comprehensible**. Every piece of code should be simple enough that a new developer can understand it quickly.

---

## Object Calisthenics Rules

### 1. One Level of Indentation per Method
Keep methods simple with minimal nesting. Extract complex logic into separate methods.

**Bad:**
```csharp
public void ProcessTestResults(List<TestResult> results)
{
    foreach (var result in results)
    {
        if (result.Success)
        {
            if (result.Duration > 1000)
            {
                // Deep nesting makes code hard to read
                LogSlowTest(result);
            }
        }
    }
}
```

**Good:**
```csharp
public void ProcessTestResults(List<TestResult> results)
{
    foreach (var result in results)
    {
        ProcessSingleResult(result);
    }
}

private void ProcessSingleResult(TestResult result)
{
    if (!result.Success) return;
    if (IsSlowTest(result)) LogSlowTest(result);
}

private bool IsSlowTest(TestResult result) => result.Duration > SLOW_TEST_THRESHOLD_MS;
```

### 2. Don't Use the `else` Keyword
Use early returns and guard clauses instead of else blocks.

**Bad:**
```csharp
public string GetTestStatus(TestResult result)
{
    if (result.Success)
    {
        return "PASSED";
    }
    else
    {
        return "FAILED";
    }
}
```

**Good:**
```csharp
public string GetTestStatus(TestResult result)
{
    if (result.Success) return "PASSED";
    return "FAILED";
}
```

### 3. Wrap All Primitives and Strings
Create meaningful types instead of using primitives directly. This provides type safety and clarity.

**Bad:**
```csharp
public class TestRunner
{
    private string accessToken;
    private int timeout;
    private string organizationId;
}
```

**Good:**
```csharp
public class TestRunner
{
    private AccessToken accessToken;
    private Timeout timeout;
    private OrganizationId organizationId;
}

public record AccessToken(string Value);
public record Timeout(int Milliseconds);
public record OrganizationId(string Value);
```

### 4. First Class Collections
Any class that contains a collection should contain no other member variables.

**Bad:**
```csharp
public class TestReport
{
    private List<TestResult> results;
    private string reportName;
    private DateTime createdAt;
}
```

**Good:**
```csharp
public class TestResults
{
    private readonly List<TestResult> results;

    public void Add(TestResult result) => results.Add(result);
    public int Count => results.Count;
    // Other collection-specific operations
}

public class TestReport
{
    private TestResults results;
    private ReportMetadata metadata;
}
```

### 5. One Dot per Line
Follow the Law of Demeter - don't chain method calls.

**Bad:**
```csharp
var result = testRunner.GetResults().GetLatest().GetDetails().Success;
```

**Good:**
```csharp
var results = testRunner.GetResults();
var latestResult = results.GetLatest();
var success = latestResult.IsSuccessful();
```

### 6. Don't Abbreviate
Use full, meaningful names. Code is read more often than written.

**Bad:**
```csharp
public class TstRnnr
{
    private TstCfg cfg;
    public void RunTst(string tstNm) { }
}
```

**Good:**
```csharp
public class TestRunner
{
    private TestConfiguration configuration;
    public void RunTest(string testName) { }
}
```

### 7. Keep All Entities Small
- Classes: Maximum 50 lines
- Methods: Maximum 5 lines (aim for, not strict)
- Packages/Namespaces: Maximum 10 files

### 8. No Classes with More Than Two Instance Variables
Keep classes focused and cohesive.

**Example:**
```csharp
public class TestOperation
{
    private readonly OperationName name;
    private readonly OperationResult result;
}
```

### 9. No Getters/Setters/Properties
Tell, don't ask. Use behavior instead of exposing state.

**Bad:**
```csharp
public class TestResult
{
    public bool Success { get; set; }
    public int Duration { get; set; }
}

// Usage
if (result.Success && result.Duration < 1000) { }
```

**Good:**
```csharp
public class TestResult
{
    private bool success;
    private int duration;

    public bool IsSuccessfulAndFast() => success && duration < 1000;
}
```

---

## SOLID Principles

### Single Responsibility Principle (SRP)
Each class should have one, and only one, reason to change.

```csharp
// Good: Each class has a single responsibility
public class TestExecutor { /* Executes tests */ }
public class TestReporter { /* Reports results */ }
public class TestValidator { /* Validates test inputs */ }
```

### Open/Closed Principle (OCP)
Open for extension, closed for modification.

```csharp
// Good: Use abstractions to allow extension
public abstract class BaseTestRunner
{
    public abstract Task<TestResult> ExecuteTest();
}

public class PythonTestRunner : BaseTestRunner { }
public class GoTestRunner : BaseTestRunner { }
```

### Liskov Substitution Principle (LSP)
Derived classes must be substitutable for their base classes.

### Interface Segregation Principle (ISP)
Many client-specific interfaces are better than one general-purpose interface.

```csharp
// Good: Segregated interfaces
public interface ITestExecutor { Task Execute(); }
public interface ITestReporter { void Report(); }

// Not: One large interface
public interface ITestFramework { /* All methods */ }
```

### Dependency Inversion Principle (DIP)
Depend on abstractions, not concretions.

```csharp
// Good: Depend on interface
public class TestOrchestrator
{
    private readonly ITestRunner runner;

    public TestOrchestrator(ITestRunner runner)
    {
        this.runner = runner;
    }
}
```

---

## Additional Standards

### No Magic Numbers
Always use named constants.

**Bad:**
```csharp
if (result.Duration > 5000) { } // What is 5000?
```

**Good:**
```csharp
private const int SLOW_TEST_THRESHOLD_MS = 5000;
if (result.Duration > SLOW_TEST_THRESHOLD_MS) { }
```

### DRY (Don't Repeat Yourself)
Extract common code into reusable methods or classes.

### Descriptive Naming
- Classes: Nouns (TestRunner, ResultAggregator)
- Methods: Verbs (ExecuteTest, GenerateReport)
- Booleans: Questions (IsValid, HasCompleted, CanExecute)
- Constants: UPPER_SNAKE_CASE

### Method Organization
```csharp
public class ExampleClass
{
    // Constants
    private const int MAX_RETRIES = 3;

    // Fields
    private readonly ILogger logger;

    // Constructor
    public ExampleClass(ILogger logger) { }

    // Public methods
    public void PublicMethod() { }

    // Protected methods
    protected void ProtectedMethod() { }

    // Private methods
    private void PrivateMethod() { }
}
```

### Error Handling
- Use exceptions for exceptional cases
- Validate inputs early (fail fast)
- Provide meaningful error messages

### Testing Guidelines
- Each class should have a corresponding test class
- Test method names should describe what they test
- Follow AAA pattern: Arrange, Act, Assert

---

## Example: Applying All Principles

```csharp
// Single responsibility, small class, meaningful names
public class TestExecutionResult
{
    private readonly TestName name;
    private readonly ExecutionStatus status;

    public TestExecutionResult(TestName name, ExecutionStatus status)
    {
        this.name = name;
        this.status = status;
    }

    public bool WasSuccessful() => status.IsSuccess();
    public bool NeedsRetry() => status.IsRetryable();
}

// Value objects instead of primitives
public record TestName(string Value)
{
    public TestName(string value) : this(value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Test name cannot be empty");
        Value = value;
    }
}

// Small, focused interface
public interface ITestExecutor
{
    Task<TestExecutionResult> Execute(TestName testName);
}

// Clear separation of concerns
public class PythonTestExecutor : ITestExecutor
{
    private readonly IProcessRunner processRunner;
    private const int DEFAULT_TIMEOUT_MS = 30000;

    public PythonTestExecutor(IProcessRunner processRunner)
    {
        this.processRunner = processRunner;
    }

    public async Task<TestExecutionResult> Execute(TestName testName)
    {
        var result = await RunPythonTest(testName);
        return CreateExecutionResult(testName, result);
    }

    private async Task<ProcessResult> RunPythonTest(TestName testName)
    {
        return await processRunner.Run("python", testName.Value, DEFAULT_TIMEOUT_MS);
    }

    private TestExecutionResult CreateExecutionResult(TestName name, ProcessResult result)
    {
        var status = DetermineStatus(result);
        return new TestExecutionResult(name, status);
    }

    private ExecutionStatus DetermineStatus(ProcessResult result)
    {
        if (result.ExitCode == 0) return ExecutionStatus.Success();
        if (result.TimedOut) return ExecutionStatus.Timeout();
        return ExecutionStatus.Failed(result.Error);
    }
}
```

---

## Pragmatic Application

While these rules provide excellent guidelines, apply them pragmatically:

1. **Start with the most impactful**: Focus on SOLID principles and avoiding magic numbers first
2. **Gradual adoption**: Don't try to apply all rules at once
3. **Team agreement**: Discuss and agree on which rules to enforce strictly
4. **Tool assistance**: Use linters and code analyzers to enforce standards
5. **Code reviews**: Use these standards as a checklist during reviews

Remember: The goal is to write **maintainable, testable, and understandable code**. These rules are means to that end, not the end itself.