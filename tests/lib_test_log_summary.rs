use log::debug;
use mlperf_log_parser::log_summary::grammar::LogSummaryParser;
use mlperf_log_parser::log_summary::lexer::Lexer;
use serde_json;
use serde_yaml;
use test_log::test;

#[test]
fn test_grammar() {
    let mut input = String::from(
        r###"
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
      MyKey : MyValue
    This is a message ==== 2
  Early stopping satisfied: Yes
Recommendations:
 * Increase expected QPS so the loadgen pre-generates a larger (coalesced) query.
TOP Message

================================================
Additional Stats
================================================
Min latency (ns)                : 123456
Max latency (ns)                : 789012
Mean latency (ns)               : 345678
50.00 percentile latency (ns)   : 234567
90.00 percentile latency (ns)   : 456789
95.00 percentile latency (ns)   : 567890
97.00 percentile latency (ns)   : 678901
99.00 percentile latency (ns)   : 789012
99.90 percentile latency (ns)   : 890123

================================================
Test Parameters Used
================================================
Batch size                      : 32
Data type                       : INT8
Dataset                         : ImageNet
Model                          : ResNet50 v1.5
Quality target                 : 99%
Target latency (ns)            : 10000000

Notes: This is a sample result file for testing purposes. 


"###,
    );

    debug!("{}", input);
    if !input.ends_with("\n\n") {
        input.push_str("\n\n");
    }

    let tokens = Lexer::new(&input);
    let parser = LogSummaryParser::new();
    let result = parser.parse(tokens);
    debug!("{:?}", result);
    let dict = result.unwrap().to_dict();
    debug!("{}", serde_json::to_string_pretty(&dict).unwrap());
    debug!("{}", serde_yaml::to_string(&dict).unwrap());
}
