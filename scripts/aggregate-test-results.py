#!/usr/bin/env python3
"""
Test Result Aggregation Script
Aggregates JSON test results from multiple language/platform test runs
"""

import json
import os
import sys
import glob
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any


class TestResultAggregator:
    """Aggregates test results from multiple test runs"""

    def __init__(self, results_dir: str):
        self.results_dir = Path(results_dir)
        self.results = []
        self.aggregate = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "total_runs": 0,
            "successful_runs": 0,
            "failed_runs": 0,
            "success_rate": 0.0,
            "languages": [],
            "platforms": [],
            "test_mode": None,
            "sdk_source": None,
            "by_language": {},
            "by_platform": {},
            "all_passed": True,
            "failed_tests_summary": [],
            "performance_metrics": {},
            "detailed_results": []
        }

    def load_results(self):
        """Load all JSON result files from the results directory"""
        # Find all JSON files in subdirectories
        json_files = list(self.results_dir.glob("**/test-results-*.json"))

        if not json_files:
            print(f"Warning: No test result files found in {self.results_dir}", file=sys.stderr)
            return

        for json_file in json_files:
            try:
                with open(json_file, 'r') as f:
                    result = json.load(f)
                    # Add metadata
                    result['_file_path'] = str(json_file)
                    result['_file_name'] = json_file.name
                    self.results.append(result)
            except Exception as e:
                print(f"Error loading {json_file}: {e}", file=sys.stderr)

    def aggregate_results(self):
        """Aggregate all loaded results"""
        if not self.results:
            return

        self.aggregate["total_runs"] = len(self.results)

        # Collect unique languages and platforms
        languages = set()
        platforms = set()

        for result in self.results:
            # Extract language
            language = result.get("language", "unknown")
            languages.add(language)

            # Extract platform
            env = result.get("environment", {})
            platform = env.get("os", {}).get("platform", "unknown")
            platforms.add(platform)

            # Get test mode and SDK source (should be same across all runs)
            if not self.aggregate["test_mode"]:
                self.aggregate["test_mode"] = env.get("server", {}).get("type", "unknown")
            if not self.aggregate["sdk_source"]:
                self.aggregate["sdk_source"] = env.get("sdk", {}).get("source", "unknown")

            # Process test results
            test_results = result.get("test_results", {})
            summary = test_results.get("summary", {})

            # Check if this run passed
            run_passed = summary.get("status") == "passed"
            if run_passed:
                self.aggregate["successful_runs"] += 1
            else:
                self.aggregate["failed_runs"] += 1
                self.aggregate["all_passed"] = False

            # Aggregate by language
            if language not in self.aggregate["by_language"]:
                self.aggregate["by_language"][language] = {
                    "platforms": [],
                    "total_tests": 0,
                    "passed": 0,
                    "failed": 0,
                    "errored": 0,
                    "skipped": 0,
                    "all_passed": True,
                    "duration_ms": 0,
                    "test_runs": []
                }

            lang_stats = self.aggregate["by_language"][language]
            lang_stats["platforms"].append(platform)
            lang_stats["total_tests"] += summary.get("total", 0)
            lang_stats["passed"] += summary.get("passed", 0)
            lang_stats["failed"] += summary.get("failed", 0)
            lang_stats["errored"] += summary.get("errored", 0)
            lang_stats["skipped"] += summary.get("skipped", 0)
            lang_stats["duration_ms"] += summary.get("duration_ms", 0)
            lang_stats["test_runs"].append({
                "platform": platform,
                "status": summary.get("status", "unknown"),
                "total": summary.get("total", 0),
                "passed": summary.get("passed", 0),
                "failed": summary.get("failed", 0)
            })

            if not run_passed:
                lang_stats["all_passed"] = False

            # Aggregate by platform
            if platform not in self.aggregate["by_platform"]:
                self.aggregate["by_platform"][platform] = {
                    "languages": [],
                    "total_tests": 0,
                    "passed": 0,
                    "failed": 0,
                    "errored": 0,
                    "skipped": 0,
                    "all_passed": True,
                    "duration_ms": 0
                }

            plat_stats = self.aggregate["by_platform"][platform]
            plat_stats["languages"].append(language)
            plat_stats["total_tests"] += summary.get("total", 0)
            plat_stats["passed"] += summary.get("passed", 0)
            plat_stats["failed"] += summary.get("failed", 0)
            plat_stats["errored"] += summary.get("errored", 0)
            plat_stats["skipped"] += summary.get("skipped", 0)
            plat_stats["duration_ms"] += summary.get("duration_ms", 0)

            if not run_passed:
                plat_stats["all_passed"] = False

            # Collect failed tests
            failed_tests = test_results.get("failed_tests", [])
            error_tests = test_results.get("error_tests", [])

            for test in failed_tests + error_tests:
                self.aggregate["failed_tests_summary"].append({
                    "language": language,
                    "platform": platform,
                    "test_name": test.get("display_name", test.get("name", "unknown")),
                    "category": test.get("category", "unknown"),
                    "error": test.get("error", {}).get("message", "Unknown error"),
                    "error_type": test.get("error", {}).get("type", "unknown")
                })

            # Add to detailed results
            self.aggregate["detailed_results"].append({
                "language": language,
                "platform": platform,
                "status": summary.get("status", "unknown"),
                "summary": summary,
                "file": result.get("_file_name", "unknown")
            })

        # Set final values
        self.aggregate["languages"] = sorted(list(languages))
        self.aggregate["platforms"] = sorted(list(platforms))

        # Calculate success rate
        if self.aggregate["total_runs"] > 0:
            self.aggregate["success_rate"] = (
                self.aggregate["successful_runs"] / self.aggregate["total_runs"]
            ) * 100

        # Clean up platform lists in language stats
        for lang_stats in self.aggregate["by_language"].values():
            lang_stats["platforms"] = sorted(list(set(lang_stats["platforms"])))

        # Clean up language lists in platform stats
        for plat_stats in self.aggregate["by_platform"].values():
            plat_stats["languages"] = sorted(list(set(plat_stats["languages"])))

        # Calculate performance metrics
        self._calculate_performance_metrics()

    def _calculate_performance_metrics(self):
        """Calculate performance metrics across all tests"""
        total_duration = 0
        test_durations = []
        tests_by_category = {}

        for result in self.results:
            test_results = result.get("test_results", {})
            summary = test_results.get("summary", {})
            duration = summary.get("duration_ms", 0)

            if duration > 0:
                total_duration += duration
                test_durations.append(duration)

            # Collect test times by category
            for test_type in ["passed_tests", "failed_tests", "error_tests"]:
                for test in test_results.get(test_type, []):
                    category = test.get("category", "unknown")
                    if category not in tests_by_category:
                        tests_by_category[category] = []
                    tests_by_category[category].append(test.get("duration_ms", 0))

        # Calculate metrics
        if test_durations:
            self.aggregate["performance_metrics"] = {
                "total_duration_ms": total_duration,
                "average_duration_ms": total_duration / len(test_durations),
                "min_duration_ms": min(test_durations),
                "max_duration_ms": max(test_durations),
                "median_duration_ms": sorted(test_durations)[len(test_durations) // 2]
            }

            # Category metrics
            category_metrics = {}
            for category, durations in tests_by_category.items():
                if durations:
                    category_metrics[category] = {
                        "count": len(durations),
                        "total_ms": sum(durations),
                        "average_ms": sum(durations) / len(durations),
                        "min_ms": min(durations),
                        "max_ms": max(durations)
                    }
            self.aggregate["performance_metrics"]["by_category"] = category_metrics

    def generate_markdown_summary(self) -> str:
        """Generate a markdown summary of the results"""
        lines = []
        lines.append("# SDK Test Results Summary\n")
        lines.append(f"Generated: {self.aggregate['timestamp']}\n")

        # Overall statistics
        lines.append("## Overall Statistics\n")
        lines.append(f"- **Total Test Runs**: {self.aggregate['total_runs']}")
        lines.append(f"- **Successful Runs**: {self.aggregate['successful_runs']} ✅")
        lines.append(f"- **Failed Runs**: {self.aggregate['failed_runs']} ❌")
        lines.append(f"- **Success Rate**: {self.aggregate['success_rate']:.1f}%")
        lines.append(f"- **Overall Status**: {'✅ PASSED' if self.aggregate['all_passed'] else '❌ FAILED'}\n")

        # Configuration
        lines.append("## Configuration\n")
        lines.append(f"- **Test Mode**: {self.aggregate['test_mode']}")
        lines.append(f"- **SDK Source**: {self.aggregate['sdk_source']}")
        lines.append(f"- **Languages**: {', '.join(self.aggregate['languages'])}")
        lines.append(f"- **Platforms**: {', '.join(self.aggregate['platforms'])}\n")

        # Language results
        lines.append("## Results by Language\n")
        for lang, stats in self.aggregate["by_language"].items():
            status = "✅" if stats["all_passed"] else "❌"
            lines.append(f"### {lang.capitalize()} {status}\n")
            lines.append(f"- **Platforms**: {', '.join(stats['platforms'])}")
            lines.append(f"- **Total Tests**: {stats['total_tests']}")
            lines.append(f"- **Passed**: {stats['passed']} ({stats['passed']/stats['total_tests']*100:.1f}%)")
            lines.append(f"- **Failed**: {stats['failed']}")
            lines.append(f"- **Errored**: {stats['errored']}")
            lines.append(f"- **Skipped**: {stats['skipped']}")
            lines.append(f"- **Duration**: {stats['duration_ms']}ms\n")

        # Platform results
        lines.append("## Results by Platform\n")
        for platform, stats in self.aggregate["by_platform"].items():
            status = "✅" if stats["all_passed"] else "❌"
            lines.append(f"### {platform.capitalize()} {status}\n")
            lines.append(f"- **Languages**: {', '.join(stats['languages'])}")
            lines.append(f"- **Total Tests**: {stats['total_tests']}")
            lines.append(f"- **Passed**: {stats['passed']}")
            lines.append(f"- **Failed**: {stats['failed']}")
            lines.append(f"- **Duration**: {stats['duration_ms']}ms\n")

        # Failed tests summary
        if self.aggregate["failed_tests_summary"]:
            lines.append("## Failed Tests\n")
            for test in self.aggregate["failed_tests_summary"]:
                lines.append(f"- **{test['test_name']}** ({test['language']}/{test['platform']})")
                lines.append(f"  - Category: {test['category']}")
                lines.append(f"  - Error: {test['error']}")
                lines.append(f"  - Type: {test['error_type']}\n")

        # Performance metrics
        if self.aggregate["performance_metrics"]:
            metrics = self.aggregate["performance_metrics"]
            lines.append("## Performance Metrics\n")
            lines.append(f"- **Total Duration**: {metrics['total_duration_ms']}ms")
            lines.append(f"- **Average Duration**: {metrics['average_duration_ms']:.0f}ms")
            lines.append(f"- **Min Duration**: {metrics['min_duration_ms']}ms")
            lines.append(f"- **Max Duration**: {metrics['max_duration_ms']}ms")
            lines.append(f"- **Median Duration**: {metrics['median_duration_ms']}ms\n")

            if "by_category" in metrics:
                lines.append("### By Category\n")
                for category, cat_metrics in metrics["by_category"].items():
                    lines.append(f"- **{category}**: {cat_metrics['count']} tests, "
                               f"avg {cat_metrics['average_ms']:.0f}ms")

        return "\n".join(lines)

    def save_results(self, output_file: str = None):
        """Save aggregated results to a file"""
        if output_file:
            with open(output_file, 'w') as f:
                json.dump(self.aggregate, f, indent=2)
        else:
            # Print to stdout
            print(json.dumps(self.aggregate, indent=2))

    def save_markdown(self, output_file: str):
        """Save markdown summary to a file"""
        markdown = self.generate_markdown_summary()
        with open(output_file, 'w') as f:
            f.write(markdown)


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description="Aggregate SDK test results")
    parser.add_argument(
        "results_dir",
        help="Directory containing test result JSON files"
    )
    parser.add_argument(
        "-o", "--output",
        help="Output file for aggregated JSON results (default: stdout)"
    )
    parser.add_argument(
        "-m", "--markdown",
        help="Output file for markdown summary"
    )
    parser.add_argument(
        "-v", "--verbose",
        action="store_true",
        help="Enable verbose output"
    )

    args = parser.parse_args()

    # Check if results directory exists
    if not os.path.exists(args.results_dir):
        print(f"Error: Results directory '{args.results_dir}' does not exist", file=sys.stderr)
        sys.exit(1)

    # Create aggregator
    aggregator = TestResultAggregator(args.results_dir)

    # Load and aggregate results
    if args.verbose:
        print(f"Loading results from {args.results_dir}...", file=sys.stderr)

    aggregator.load_results()

    if args.verbose:
        print(f"Loaded {len(aggregator.results)} result files", file=sys.stderr)

    aggregator.aggregate_results()

    # Save results
    if args.output:
        aggregator.save_results(args.output)
        if args.verbose:
            print(f"Saved aggregated results to {args.output}", file=sys.stderr)
    else:
        aggregator.save_results()

    # Save markdown if requested
    if args.markdown:
        aggregator.save_markdown(args.markdown)
        if args.verbose:
            print(f"Saved markdown summary to {args.markdown}", file=sys.stderr)

    # Exit with appropriate code
    sys.exit(0 if aggregator.aggregate["all_passed"] else 1)


if __name__ == "__main__":
    main()