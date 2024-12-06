use mlperf_log_parser::log_summary::grammar::Document;

#[test]
fn test_grammar() {
    let input = r#"
================================================
MLPerf Results Summary
================================================
SUT name : BERT SUT
Scenario : Offline
Mode : PerformanceOnly
  Min duration satisfied : NO
  Min queries satisfied : Yes
    Further details : NOT REQUIRED
    This is a message
  Early stopping satisfied: Yes
Recommendations:
 * Increase expected QPS so the loadgen pre-generates a larger (coalesced) query.

================================================
Additional Stats
================================================
Min latency (ns)                : 8740448339
Max latency (ns)                : 1443876288808
Mean latency (ns)               : 729202473042
50.00 percentile latency (ns)   : 729112772963
90.00 percentile latency (ns)   : 1302950749196
95.00 percentile latency (ns)   : 1374489698221
97.00 percentile latency (ns)   : 1402380862634
99.00 percentile latency (ns)   : 1431532216788
99.90 percentile latency (ns)   : 1440960003323

================================================
Test Parameters Used
================================================
samples_per_query : 13368
target_qps : 1
target_latency (ns): 0
max_async_queries : 1
min_duration (ms): 600000
max_duration (ms): 0
min_query_count : 1
max_query_count : 0
qsl_rng_seed : 3066443479025735752
sample_index_rng_seed : 10688027786191513374
schedule_rng_seed : 14962580496156340209
accuracy_log_rng_seed : 0
accuracy_log_probability : 0
accuracy_log_sampling_target : 0
print_timestamps : 0
performance_issue_unique : 0
performance_issue_same : 0
performance_issue_same_index : 0
performance_sample_count : 13368

No warnings encountered during test.

No errors encountered during test.
"#;
    let result = Document::parse(input);
    println!("{:?}", result);
}
