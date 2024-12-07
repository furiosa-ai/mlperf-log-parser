use mlperf_log_parser::log_summary::grammar::LogSummaryParser;
use mlperf_log_parser::log_summary::lexer::Lexer;
use serde_json;
use serde_yaml;
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

"###,
    );

    println!("{}", input);
    if !input.ends_with("\n\n") {
        input.push_str("\n\n");
    }

    let tokens = Lexer::new(&input);
    let parser = LogSummaryParser::new();
    let result = parser.parse(tokens);
    println!("{:?}", result);
    let dict = result.unwrap().to_dict();
    println!("{}", serde_json::to_string_pretty(&dict).unwrap());
    println!("{}", serde_yaml::to_string(&dict).unwrap());
}
