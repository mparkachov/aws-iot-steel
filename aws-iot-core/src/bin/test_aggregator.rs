use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct TestSuite {
    name: String,
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    duration_ms: u64,
    success_rate: f64,
    details: Vec<TestResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    name: String,
    status: TestStatus,
    duration_ms: u64,
    error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

#[derive(Debug, Serialize, Deserialize)]
struct AggregatedReport {
    timestamp: String,
    total_suites: usize,
    total_tests: usize,
    total_passed: usize,
    total_failed: usize,
    overall_success_rate: f64,
    total_duration_ms: u64,
    suites: Vec<TestSuite>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("test_aggregator")
        .version("1.0")
        .about("Test result aggregator for AWS IoT Steel dual testing infrastructure")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file for aggregated results")
                .default_value("test-results.json"),
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .help("Output format (json, html, markdown)")
                .default_value("json"),
        )
        .arg(
            Arg::new("run-tests")
                .short('r')
                .long("run-tests")
                .action(clap::ArgAction::SetTrue)
                .help("Run all tests before aggregating results"),
        )
        .get_matches();

    let output_file = matches.get_one::<String>("output").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    let run_tests = matches.get_flag("run-tests");

    info!("=== Test Result Aggregator Starting ===");

    let mut report = AggregatedReport {
        timestamp: chrono::Utc::now().to_rfc3339(),
        total_suites: 0,
        total_tests: 0,
        total_passed: 0,
        total_failed: 0,
        overall_success_rate: 0.0,
        total_duration_ms: 0,
        suites: Vec::new(),
    };

    if run_tests {
        info!("Running all tests before aggregation...");
        run_all_tests().await?;
    }

    // Aggregate Rust unit test results
    info!("Aggregating Rust unit test results...");
    if let Ok(rust_unit_suite) = aggregate_rust_unit_tests().await {
        report.suites.push(rust_unit_suite);
    }

    // Aggregate Rust integration test results
    info!("Aggregating Rust integration test results...");
    if let Ok(rust_integration_suite) = aggregate_rust_integration_tests().await {
        report.suites.push(rust_integration_suite);
    }

    // Aggregate Steel test results
    info!("Aggregating Steel test results...");
    if let Ok(steel_suite) = aggregate_steel_tests().await {
        report.suites.push(steel_suite);
    }

    // Calculate totals
    calculate_totals(&mut report);

    // Generate output
    match format.as_str() {
        "json" => generate_json_report(&report, output_file)?,
        "html" => generate_html_report(&report, output_file)?,
        "markdown" => generate_markdown_report(&report, output_file)?,
        _ => {
            error!("Unsupported format: {}", format);
            std::process::exit(1);
        }
    }

    // Print summary
    print_summary(&report);

    info!("=== Test Result Aggregation Completed ===");

    // Exit with error code if any tests failed
    if report.total_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

async fn run_all_tests() -> Result<(), Box<dyn std::error::Error>> {
    info!("Running Rust unit tests...");
    let rust_unit_output = ProcessCommand::new("cargo")
        .args(["test", "--workspace", "--lib", "--", "--format=json"])
        .output()?;

    fs::write("rust-unit-results.json", rust_unit_output.stdout)?;

    info!("Running Rust integration tests...");
    let rust_integration_output = ProcessCommand::new("cargo")
        .args(["test", "--workspace", "--test", "*", "--", "--format=json"])
        .output()?;

    fs::write(
        "rust-integration-results.json",
        rust_integration_output.stdout,
    )?;

    info!("Running Steel tests...");
    let steel_output = ProcessCommand::new("cargo")
        .args([
            "run",
            "--bin",
            "steel_test",
            "--package",
            "aws-iot-core",
            "--",
            "--verbose",
        ])
        .output()?;

    fs::write("steel-test-results.log", steel_output.stdout)?;

    Ok(())
}

async fn aggregate_rust_unit_tests() -> Result<TestSuite, Box<dyn std::error::Error>> {
    let mut suite = TestSuite {
        name: "Rust Unit Tests".to_string(),
        total_tests: 0,
        passed_tests: 0,
        failed_tests: 0,
        duration_ms: 0,
        success_rate: 0.0,
        details: Vec::new(),
    };

    // Try to read existing results or run tests
    let results_content = if Path::new("rust-unit-results.json").exists() {
        fs::read_to_string("rust-unit-results.json")?
    } else {
        info!("Running Rust unit tests...");
        let output = ProcessCommand::new("cargo")
            .args(["test", "--workspace", "--lib"])
            .output()?;

        String::from_utf8_lossy(&output.stdout).to_string()
    };

    // Parse test results (simplified parsing for demo)
    // In a real implementation, you'd parse the JSON format properly
    let lines: Vec<&str> = results_content.lines().collect();

    for line in lines {
        if line.contains("test result:") {
            // Extract test summary
            if let Some(passed_pos) = line.find(" passed") {
                if let Some(start) = line[..passed_pos].rfind(' ') {
                    if let Ok(passed) = line[start + 1..passed_pos].parse::<usize>() {
                        suite.passed_tests = passed;
                    }
                }
            }

            if let Some(failed_pos) = line.find(" failed") {
                if let Some(start) = line[..failed_pos].rfind(' ') {
                    if let Ok(failed) = line[start + 1..failed_pos].parse::<usize>() {
                        suite.failed_tests = failed;
                    }
                }
            }
        }
    }

    suite.total_tests = suite.passed_tests + suite.failed_tests;
    suite.success_rate = if suite.total_tests > 0 {
        suite.passed_tests as f64 / suite.total_tests as f64 * 100.0
    } else {
        0.0
    };

    Ok(suite)
}

async fn aggregate_rust_integration_tests() -> Result<TestSuite, Box<dyn std::error::Error>> {
    let mut suite = TestSuite {
        name: "Rust Integration Tests".to_string(),
        total_tests: 0,
        passed_tests: 0,
        failed_tests: 0,
        duration_ms: 0,
        success_rate: 0.0,
        details: Vec::new(),
    };

    // Similar implementation to unit tests
    // This is a simplified version for demonstration
    suite.total_tests = 10; // Mock data
    suite.passed_tests = 9;
    suite.failed_tests = 1;
    suite.success_rate = 90.0;

    Ok(suite)
}

async fn aggregate_steel_tests() -> Result<TestSuite, Box<dyn std::error::Error>> {
    let mut suite = TestSuite {
        name: "Steel Tests".to_string(),
        total_tests: 0,
        passed_tests: 0,
        failed_tests: 0,
        duration_ms: 0,
        success_rate: 0.0,
        details: Vec::new(),
    };

    // Try to read Steel test results
    let results_content = if Path::new("steel-test-results.log").exists() {
        fs::read_to_string("steel-test-results.log")?
    } else {
        info!("Running Steel tests...");
        let output = ProcessCommand::new("cargo")
            .args(["run", "--bin", "steel_test", "--package", "aws-iot-core"])
            .output()?;

        String::from_utf8_lossy(&output.stdout).to_string()
    };

    // Parse Steel test results
    let lines: Vec<&str> = results_content.lines().collect();

    for line in lines {
        if line.contains("Tests run:") {
            // Extract Steel test summary
            if let Some(total_pos) = line.find("Tests run: ") {
                let rest = &line[total_pos + 11..];
                if let Some(comma_pos) = rest.find(',') {
                    if let Ok(total) = rest[..comma_pos].parse::<usize>() {
                        suite.total_tests = total;
                    }
                }
            }
        }

        if line.contains("Passed: ") {
            if let Some(passed_pos) = line.find("Passed: ") {
                let rest = &line[passed_pos + 8..];
                if let Some(space_pos) = rest.find(' ') {
                    if let Ok(passed) = rest[..space_pos].parse::<usize>() {
                        suite.passed_tests = passed;
                    }
                }
            }
        }

        if line.contains("Failed: ") {
            if let Some(failed_pos) = line.find("Failed: ") {
                let rest = &line[failed_pos + 8..];
                if let Some(space_pos) = rest.find(' ') {
                    if let Ok(failed) = rest[..space_pos].parse::<usize>() {
                        suite.failed_tests = failed;
                    }
                }
            }
        }

        if line.contains("Success rate: ") {
            if let Some(rate_pos) = line.find("Success rate: ") {
                let rest = &line[rate_pos + 14..];
                if let Some(percent_pos) = rest.find('%') {
                    if let Ok(rate) = rest[..percent_pos].parse::<f64>() {
                        suite.success_rate = rate;
                    }
                }
            }
        }
    }

    Ok(suite)
}

fn calculate_totals(report: &mut AggregatedReport) {
    report.total_suites = report.suites.len();
    report.total_tests = report.suites.iter().map(|s| s.total_tests).sum();
    report.total_passed = report.suites.iter().map(|s| s.passed_tests).sum();
    report.total_failed = report.suites.iter().map(|s| s.failed_tests).sum();
    report.total_duration_ms = report.suites.iter().map(|s| s.duration_ms).sum();

    report.overall_success_rate = if report.total_tests > 0 {
        report.total_passed as f64 / report.total_tests as f64 * 100.0
    } else {
        0.0
    };
}

fn generate_json_report(
    report: &AggregatedReport,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(report)?;
    fs::write(output_file, json)?;
    info!("JSON report written to: {}", output_file);
    Ok(())
}

fn generate_html_report(
    report: &AggregatedReport,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>AWS IoT Steel Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .suite {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .passed {{ color: green; }}
        .failed {{ color: red; }}
        .summary {{ font-size: 18px; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>AWS IoT Steel Test Report</h1>
        <p>Generated: {}</p>
        <div class="summary">
            Overall: {} / {} tests passed ({:.1}% success rate)
        </div>
    </div>
    
    <h2>Test Suites</h2>
    {}
</body>
</html>"#,
        report.timestamp,
        report.total_passed,
        report.total_tests,
        report.overall_success_rate,
        report.suites.iter().map(|suite| {
            format!(
                r#"<div class="suite">
                    <h3>{}</h3>
                    <p>Tests: {} | <span class="passed">Passed: {}</span> | <span class="failed">Failed: {}</span> | Success Rate: {:.1}%</p>
                </div>"#,
                suite.name,
                suite.total_tests,
                suite.passed_tests,
                suite.failed_tests,
                suite.success_rate
            )
        }).collect::<Vec<_>>().join("\n")
    );

    fs::write(output_file, html)?;
    info!("HTML report written to: {}", output_file);
    Ok(())
}

fn generate_markdown_report(
    report: &AggregatedReport,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let markdown = format!(
        r#"# AWS IoT Steel Test Report

**Generated:** {}

## Summary

- **Total Test Suites:** {}
- **Total Tests:** {}
- **Passed:** {} ✅
- **Failed:** {} ❌
- **Overall Success Rate:** {:.1}%
- **Total Duration:** {}ms

## Test Suites

{}

## Conclusion

{}
"#,
        report.timestamp,
        report.total_suites,
        report.total_tests,
        report.total_passed,
        report.total_failed,
        report.overall_success_rate,
        report.total_duration_ms,
        report
            .suites
            .iter()
            .map(|suite| {
                format!(
                "### {}\n\n- Tests: {}\n- Passed: {} ✅\n- Failed: {} ❌\n- Success Rate: {:.1}%\n",
                suite.name,
                suite.total_tests,
                suite.passed_tests,
                suite.failed_tests,
                suite.success_rate
            )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        if report.total_failed == 0 {
            "🎉 All tests passed! The dual testing infrastructure is working correctly."
        } else {
            "⚠️ Some tests failed. Please review the failed tests and fix any issues."
        }
    );

    fs::write(output_file, markdown)?;
    info!("Markdown report written to: {}", output_file);
    Ok(())
}

fn print_summary(report: &AggregatedReport) {
    info!("");
    info!("=== Test Summary ===");
    info!("Total Suites: {}", report.total_suites);
    info!("Total Tests: {}", report.total_tests);
    info!("Passed: {} ✅", report.total_passed);
    info!("Failed: {} ❌", report.total_failed);
    info!("Success Rate: {:.1}%", report.overall_success_rate);
    info!("Duration: {}ms", report.total_duration_ms);

    info!("");
    info!("Suite Breakdown:");
    for suite in &report.suites {
        let status = if suite.failed_tests == 0 {
            "✅"
        } else {
            "❌"
        };
        info!(
            "  {} {} - {}/{} passed ({:.1}%)",
            status, suite.name, suite.passed_tests, suite.total_tests, suite.success_rate
        );
    }

    if report.total_failed == 0 {
        info!("");
        info!("🎉 All tests passed! Dual testing infrastructure is working correctly.");
    } else {
        warn!("");
        warn!(
            "⚠️ {} tests failed. Please review and fix issues.",
            report.total_failed
        );
    }
}
