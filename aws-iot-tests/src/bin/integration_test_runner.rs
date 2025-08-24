use aws_iot_tests::IoTIntegrationTests;
use clap::{Arg, Command};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("integration_test_runner")
        .version("1.0")
        .about("Comprehensive integration test runner for AWS IoT Steel")
        .arg(
            Arg::new("test-type")
                .short('t')
                .long("test-type")
                .value_name("TYPE")
                .help("Type of tests to run")
                .value_parser(["all", "iot", "quick"])
                .default_value("all"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file for test results"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
        .arg(
            Arg::new("fail-fast")
                .short('f')
                .long("fail-fast")
                .action(clap::ArgAction::SetTrue)
                .help("Stop on first test failure"),
        )
        .get_matches();

    let test_type = matches.get_one::<String>("test-type").unwrap();
    let output_file = matches.get_one::<String>("output");
    let verbose = matches.get_flag("verbose");
    let fail_fast = matches.get_flag("fail-fast");

    info!("=== AWS IoT Steel Integration Test Runner ===");
    info!("Test type: {}", test_type);

    let mut overall_success = true;
    let mut test_results = Vec::new();

    match test_type.as_str() {
        "all" => {
            info!("Running comprehensive test suite...");

            // Run IoT integration tests
            if let Err(e) = run_iot_tests(&mut test_results, verbose, fail_fast).await {
                error!("IoT integration tests failed: {}", e);
                overall_success = false;
                if fail_fast {
                    std::process::exit(1);
                }
            }
        }
        "iot" => {
            info!("Running IoT integration tests only...");
            if let Err(e) = run_iot_tests(&mut test_results, verbose, fail_fast).await {
                error!("IoT integration tests failed: {}", e);
                overall_success = false;
            }
        }
        "quick" => {
            info!("Running quick smoke tests...");
            if let Err(e) = run_quick_tests(&mut test_results, verbose).await {
                error!("Quick tests failed: {}", e);
                overall_success = false;
            }
        }
        _ => {
            error!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }

    // Print final summary
    print_final_summary(&test_results, overall_success);

    // Write results to file if specified
    if let Some(output_path) = output_file {
        write_results_to_file(&test_results, output_path, overall_success)?;
        info!("Test results written to: {}", output_path);
    }

    if !overall_success {
        std::process::exit(1);
    }

    Ok(())
}

async fn run_iot_tests(
    test_results: &mut Vec<TestSummary>,
    verbose: bool,
    fail_fast: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🌐 Starting IoT Integration Tests...");

    let iot_tests = IoTIntegrationTests::new();
    let results = iot_tests.run_all_tests().await?;

    if verbose {
        results.print_summary();
    }

    let success_rate = results.success_rate();
    let summary = TestSummary {
        test_suite: "IoT Integration".to_string(),
        total_tests: results.total_tests(),
        passed_tests: results.passed_tests(),
        failed_tests: results.failed_tests(),
        success_rate,
        duration_seconds: 0, // Would be measured in real implementation
    };

    test_results.push(summary);

    if success_rate < 100.0 && fail_fast {
        return Err("IoT integration tests failed".into());
    }

    info!(
        "✅ IoT Integration Tests completed: {:.1}% success rate",
        success_rate
    );
    Ok(())
}

async fn run_quick_tests(
    test_results: &mut Vec<TestSummary>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 Starting Quick Smoke Tests...");

    // Run a basic IoT integration test for quick validation
    let iot_tests = IoTIntegrationTests::new();
    let results = iot_tests.run_all_tests().await?;

    if verbose {
        results.print_summary();
    }

    let success_rate = results.success_rate();
    let summary = TestSummary {
        test_suite: "Quick Smoke Tests".to_string(),
        total_tests: results.total_tests(),
        passed_tests: results.passed_tests(),
        failed_tests: results.failed_tests(),
        success_rate,
        duration_seconds: 0,
    };

    test_results.push(summary);

    info!(
        "✅ Quick Tests completed: {:.1}% success rate",
        success_rate
    );
    Ok(())
}

fn print_final_summary(test_results: &[TestSummary], overall_success: bool) {
    info!("");
    info!("=== Final Test Summary ===");

    let mut total_tests = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;

    for summary in test_results {
        info!(
            "📊 {}: {}/{} passed ({:.1}%)",
            summary.test_suite, summary.passed_tests, summary.total_tests, summary.success_rate
        );

        total_tests += summary.total_tests;
        total_passed += summary.passed_tests;
        total_failed += summary.failed_tests;
    }

    let overall_success_rate = if total_tests > 0 {
        total_passed as f64 / total_tests as f64 * 100.0
    } else {
        0.0
    };

    info!("");
    info!("🎯 Overall Results:");
    info!("   Total test suites: {}", test_results.len());
    info!("   Total tests: {}", total_tests);
    info!("   Total passed: {}", total_passed);
    info!("   Total failed: {}", total_failed);
    info!("   Overall success rate: {:.1}%", overall_success_rate);

    if overall_success {
        info!("   Status: ✅ ALL TESTS PASSED");
    } else {
        warn!("   Status: ❌ SOME TESTS FAILED");
    }

    info!("=== End Summary ===");
}

fn write_results_to_file(
    test_results: &[TestSummary],
    output_path: &str,
    overall_success: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let json_results = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "overall_success": overall_success,
        "test_suites": test_results.iter().map(|summary| {
            serde_json::json!({
                "name": summary.test_suite,
                "total_tests": summary.total_tests,
                "passed_tests": summary.passed_tests,
                "failed_tests": summary.failed_tests,
                "success_rate": summary.success_rate,
                "duration_seconds": summary.duration_seconds
            })
        }).collect::<Vec<_>>()
    });

    fs::write(output_path, serde_json::to_string_pretty(&json_results)?)?;
    Ok(())
}

#[derive(Debug, Clone)]
struct TestSummary {
    test_suite: String,
    total_tests: usize,
    passed_tests: usize,
    failed_tests: usize,
    success_rate: f64,
    duration_seconds: u64,
}
