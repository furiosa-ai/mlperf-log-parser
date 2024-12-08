pub mod grammar;
pub mod lexer;
pub mod model;

use grammar::LogSummaryParser;
use lexer::Lexer;
use model::Document;
use serde_value::Value;
use std::fs;
use std::fs::File;
use std::io;

pub fn parse_log_summary(log_summary: &str) -> Result<Document, String> {
    let lexer = Lexer::new(log_summary);
    let parser = LogSummaryParser::new();
    match parser.parse(lexer) {
        Ok(doc) => Ok(doc),
        Err(e) => {
            // 에러가 발생한 위치를 찾기 위해 각 라인의 길이를 계산
            let mut current_pos = 0;
            let mut line_number = 1;

            // 에러 메시지에서 위치 정보 추출
            if let Some(location) = format!("{:?}", e).find("location: ") {
                if let Ok(error_pos) = format!("{:?}", e)[location + 10..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<usize>()
                {
                    // 에러 위치까지 라인 수와 단어 위치 계산
                    let mut line_content = "";
                    for line in log_summary.lines() {
                        if current_pos + line.len() + 1 > error_pos {
                            line_content = line;
                            break;
                        }
                        current_pos += line.len() + 1; // +1 for newline
                        line_number += 1;
                    }

                    // 라인에서의 단어 위치 계산
                    let word_pos = error_pos - current_pos;
                    let word = line_content
                        .split_whitespace()
                        .nth(word_pos / 2)
                        .unwrap_or("<unknown>");

                    Err(format!(
                        "Parse error at line {}, word '{}' (position {}): {:?}",
                        line_number, word, word_pos, e
                    ))
                } else {
                    Err(format!("Parse error: {:?}", e))
                }
            } else {
                Err(format!("Parse error: {:?}", e))
            }
        }
    }
}

pub fn parse_mlperf_results_file(input_file: &str) -> io::Result<Value> {
    let mut content = fs::read_to_string(input_file)?;
    if !content.ends_with("\n\n") {
        content.push_str("\n\n");
    }
    match parse_log_summary(&content) {
        Ok(doc) => Ok(serde_value::to_value(doc.to_dict()).unwrap()),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}

pub fn save_summary_as_json(input_file: &str, output_file: &str) -> io::Result<()> {
    let summary = parse_mlperf_results_file(input_file)?;
    let mut output = File::create(output_file)?;
    serde_json::to_writer_pretty(&mut output, &summary)?;
    Ok(())
}

pub fn save_summary_as_yaml(input_file: &str, output_file: &str) -> io::Result<()> {
    let summary = parse_mlperf_results_file(input_file)?;
    let mut output = File::create(output_file)?;
    serde_yaml::to_writer(&mut output, &summary)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}

pub fn save_summary(input_file: &str, output_file: &str, format: &str) -> io::Result<()> {
    match format {
        "json" => save_summary_as_json(input_file, output_file),
        "yaml" => save_summary_as_yaml(input_file, output_file),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid format. Use 'json' or 'yaml'.",
        )),
    }
}
