use mlperf_log_parser::parse_mlperf_results;
use std::fs;

#[test]
fn test_parse_mlperf_results() {
    let test_data = fs::read_to_string("tests/data/mlperf_log_summary.txt")
        .expect("Failed to read test data file");

    let result = parse_mlperf_results(&test_data);

    // Test MLPerf Results Summary section
    assert_eq!(result["mlperf_result_summary"]["sut_name"], "Sample System");
    assert_eq!(result["mlperf_result_summary"]["scenario"], "Offline");
    assert_eq!(result["mlperf_result_summary"]["mode"], "Performance");
    assert_eq!(
        result["mlperf_result_summary"]["samples_per_second"],
        1234.56
    );

    // Test result status
    assert_eq!(result["mlperf_result_summary"]["result"]["state"], "VALID");
    assert_eq!(
        result["mlperf_result_summary"]["result"]["min_duration_satisfied"],
        true
    );
    assert_eq!(
        result["mlperf_result_summary"]["result"]["min_queries_satisfied"],
        true
    );
    assert_eq!(
        result["mlperf_result_summary"]["result"]["early_stopping_satisfied"],
        true
    );

    // Test Additional Stats section
    assert_eq!(result["additional_stats"]["min_latency_ns"], 123456);
    assert_eq!(result["additional_stats"]["max_latency_ns"], 789012);
    assert_eq!(result["additional_stats"]["mean_latency_ns"], 345678);
    assert_eq!(result["additional_stats"]["p50_latency_ns"], 234567);
    assert_eq!(result["additional_stats"]["p90_latency_ns"], 456789);
    assert_eq!(result["additional_stats"]["p95_latency_ns"], 567890);
    assert_eq!(result["additional_stats"]["p97_latency_ns"], 678901);
    assert_eq!(result["additional_stats"]["p99_latency_ns"], 789012);
    assert_eq!(result["additional_stats"]["p999_latency_ns"], 890123);

    // Test Parameters section
    assert_eq!(result["test_parameters_used"]["batch_size"], 32);
    assert_eq!(result["test_parameters_used"]["data_type"], "INT8");
    assert_eq!(result["test_parameters_used"]["dataset"], "ImageNet");
    assert_eq!(result["test_parameters_used"]["model"], "ResNet50 v1.5");
    assert_eq!(result["test_parameters_used"]["quality_target"], "99%");
    assert_eq!(
        result["test_parameters_used"]["target_latency_ns"],
        10000000
    );

    // Test message section
    assert_eq!(result["message"].as_array().unwrap().len(), 1);
    assert_eq!(
        result["message"].as_array().unwrap()[0],
        "Notes: This is a sample result file for testing purposes."
    );
}
