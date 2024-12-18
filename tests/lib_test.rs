use log::info;
use mlperf_log_parser::{parse_mlperf_log_detail_file, parse_mlperf_results_file};
use serde_value::Value;
use test_log::test;

#[test]
fn test_parse_mlperf_results() {
    let test_data_file = "tests/data/mlperf_log_summary.txt";

    let result = parse_mlperf_results_file(&test_data_file).unwrap();

    info!("{:?}", result);

    // Test MLPerf Results Summary section
    if let Value::Map(summary) = &result {
        if let Value::Map(mlperf_summary) =
            &summary[&Value::String("mlperf_results_summary".to_string())]
        {
            assert_eq!(
                mlperf_summary[&Value::String("sut_name".to_string())],
                Value::String("Sample System".to_string())
            );
            assert_eq!(
                mlperf_summary[&Value::String("scenario".to_string())],
                Value::String("Offline".to_string())
            );
            assert_eq!(
                mlperf_summary[&Value::String("mode".to_string())],
                Value::String("Performance".to_string())
            );
            assert_eq!(
                mlperf_summary[&Value::String("samples_per_second".to_string())],
                Value::F64(1234.56)
            );

            // Test result status
            if let Value::Map(result_map) = &mlperf_summary[&Value::String("result_is".to_string())]
            {
                assert_eq!(
                    result_map[&Value::String("value".to_string())],
                    Value::String("VALID".to_string())
                );
                if let Value::Map(details) = &result_map[&Value::String("details".to_string())] {
                    assert_eq!(
                        details[&Value::String("min_duration_satisfied".to_string())],
                        Value::Bool(true)
                    );
                    assert_eq!(
                        details[&Value::String("min_queries_satisfied".to_string())],
                        Value::Bool(true)
                    );
                    assert_eq!(
                        details[&Value::String("early_stopping_satisfied".to_string())],
                        Value::Bool(true)
                    );
                }
            }
        }

        // Test Additional Stats section
        if let Value::Map(additional_stats) =
            &summary[&Value::String("additional_stats".to_string())]
        {
            assert_eq!(
                additional_stats[&Value::String("min_latency_ns".to_string())],
                Value::I64(123456)
            );
            assert_eq!(
                additional_stats[&Value::String("max_latency_ns".to_string())],
                Value::I64(789012)
            );
            assert_eq!(
                additional_stats[&Value::String("mean_latency_ns".to_string())],
                Value::I64(345678)
            );
            assert_eq!(
                additional_stats[&Value::String("50_00_percentile_latency_ns".to_string())],
                Value::I64(234567)
            );
            assert_eq!(
                additional_stats[&Value::String("90_00_percentile_latency_ns".to_string())],
                Value::I64(456789)
            );
            assert_eq!(
                additional_stats[&Value::String("95_00_percentile_latency_ns".to_string())],
                Value::I64(567890)
            );
            assert_eq!(
                additional_stats[&Value::String("97_00_percentile_latency_ns".to_string())],
                Value::I64(678901)
            );
            assert_eq!(
                additional_stats[&Value::String("99_00_percentile_latency_ns".to_string())],
                Value::I64(789012)
            );
            assert_eq!(
                additional_stats[&Value::String("99_90_percentile_latency_ns".to_string())],
                Value::I64(890123)
            );
        }

        // Test Parameters section
        if let Value::Map(test_params) =
            &summary[&Value::String("test_parameters_used".to_string())]
        {
            assert_eq!(
                test_params[&Value::String("batch_size".to_string())],
                Value::I64(32)
            );
            assert_eq!(
                test_params[&Value::String("data_type".to_string())],
                Value::String("INT8".to_string())
            );
            assert_eq!(
                test_params[&Value::String("dataset".to_string())],
                Value::String("ImageNet".to_string())
            );
            assert_eq!(
                test_params[&Value::String("model".to_string())],
                Value::String("ResNet50 v1.5".to_string())
            );
            assert_eq!(
                test_params[&Value::String("quality_target".to_string())],
                Value::String("99%".to_string())
            );
            assert_eq!(
                test_params[&Value::String("target_latency_ns".to_string())],
                Value::I64(10000000)
            );
        }

        // Test note section
        if let Value::Seq(notes) = &summary[&Value::String("note".to_string())] {
            assert_eq!(notes.len(), 1);
            assert_eq!(
                notes[0],
                Value::String(
                    "Notes: This is a sample result file for testing purposes.".to_string()
                )
            );
        }
    }
}

#[test]
fn test_mlperf_log_parser() {
    let log_path = "tests/data/mlperf_log_detail.txt";
    let summary = parse_mlperf_log_detail_file(log_path);
    info!("{:?}", summary);
    assert_eq!(summary.is_ok(), true);
}
