use serde_value::Value;
use std::collections::BTreeMap;
use std::collections::HashMap; // 상단에 BTreeMap import 추가

use serde_json::Value as JsonValue;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn parse_value(value: &str) -> Value {
    match value.to_lowercase().as_str() {
        "yes" | "true" => Value::Bool(true),
        "no" | "false" => Value::Bool(false),
        _ => {
            if let Ok(num) = value.parse::<i64>() {
                Value::I64(num)
            } else if let Ok(num) = value.parse::<f64>() {
                Value::F64(num)
            } else {
                Value::String(value.to_string())
            }
        }
    }
}

pub fn parse_mlperf_results(text: &str) -> Value {
    let mut sections: BTreeMap<Value, Value> = BTreeMap::new();
    sections.insert(
        Value::String("mlperf_result_summary".to_string()),
        Value::Map(BTreeMap::new()),
    );
    sections.insert(
        Value::String("additional_stats".to_string()),
        Value::Map(BTreeMap::new()),
    );
    sections.insert(
        Value::String("test_parameters_used".to_string()),
        Value::Map(BTreeMap::new()),
    );
    sections.insert(Value::String("message".to_string()), Value::Seq(Vec::new()));

    let mut current_section = "";

    for line in text.lines() {
        let line = line.trim();

        match line {
            s if s.starts_with("MLPerf Results Summary") => {
                current_section = "mlperf_result_summary";
                continue;
            }
            s if s.starts_with("Additional Stats") => {
                current_section = "additional_stats";
                continue;
            }
            s if s.starts_with("Test Parameters Used") => {
                current_section = "test_parameters_used";
                continue;
            }
            s if s.starts_with("===") => continue,
            s if s.is_empty() => {
                current_section = "";
                continue;
            }
            _ => {}
        }

        if current_section.is_empty() {
            if let Value::Seq(messages) = sections
                .get_mut(&Value::String("message".to_string()))
                .unwrap()
            {
                messages.push(Value::String(line.to_string()));
            }
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let mut key = key
                .trim()
                .to_lowercase()
                .replace(['(', ')'], "")
                .replace(' ', "_");
            let value = value.trim();
            let processed_value = parse_value(value);

            if let Value::Map(section_map) = sections
                .get_mut(&Value::String(current_section.to_string()))
                .unwrap()
            {
                match current_section {
                    "additional_stats" => {
                        let percentile_mappings = [
                            ("50.00", "p50"),
                            ("90.00", "p90"),
                            ("95.00", "p95"),
                            ("97.00", "p97"),
                            ("99.00", "p99"),
                            ("99.90", "p999"),
                        ];

                        for (old, new) in percentile_mappings.iter() {
                            key = key.replace(old, new);
                        }
                        key = key.replace("_percentile", "");
                        section_map.insert(Value::String(key), processed_value);
                    }
                    "mlperf_result_summary" => {
                        if key == "result_is" {
                            let mut result_map = BTreeMap::new();
                            result_map.insert(
                                Value::String("state".to_string()),
                                Value::String(value.to_string()),
                            );
                            result_map.insert(
                                Value::String("min_duration_satisfied".to_string()),
                                Value::Bool(false),
                            );
                            result_map.insert(
                                Value::String("min_queries_satisfied".to_string()),
                                Value::Bool(false),
                            );
                            result_map.insert(
                                Value::String("early_stopping_satisfied".to_string()),
                                Value::Bool(false),
                            );
                            section_map.insert(
                                Value::String("result".to_string()),
                                Value::Map(result_map),
                            );
                        } else if key.ends_with("_satisfied") {
                            if let Some(Value::Map(result_map)) =
                                section_map.get_mut(&Value::String("result".to_string()))
                            {
                                result_map.insert(Value::String(key), processed_value);
                            }
                        } else {
                            section_map.insert(Value::String(key), processed_value);
                        }
                    }
                    _ => {
                        section_map.insert(Value::String(key), processed_value);
                    }
                }
            }
        }
    }

    Value::Map(sections)
}

pub fn save_summary_as_json(summary_file: &str, output_file: &str) -> io::Result<()> {
    let file = File::open(summary_file)?;
    let reader = BufReader::new(file);
    let text: String = reader.lines().collect::<io::Result<Vec<_>>>()?.join("\n");

    let summary = parse_mlperf_results(&text);

    let json_value =
        serde_json::to_value(&summary).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut output = File::create(output_file)?;
    serde_json::to_writer_pretty(&mut output, &json_value)?;
    Ok(())
}

pub fn save_summary_as_yaml(summary_file: &str, output_file: &str) -> io::Result<()> {
    let file = File::open(summary_file)?;
    let reader = BufReader::new(file);
    let text: String = reader.lines().collect::<io::Result<Vec<_>>>()?.join("\n");

    let summary = parse_mlperf_results(&text);

    let mut output = File::create(output_file)?;
    serde_yaml::to_writer(&mut output, &summary)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}

pub fn save_summary(summary_file: &str, output_file: &str, format: &str) -> io::Result<()> {
    if format == "json" {
        save_summary_as_json(summary_file, output_file)
    } else if format == "yaml" {
        save_summary_as_yaml(summary_file, output_file)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid format. Use 'json' or 'yaml'.",
        ))
    }
}

#[derive(Debug, serde::Serialize)]
pub struct MLLogEntry {
    pub key: String,
    pub value: JsonValue,
    pub time_ms: f64,
    pub namespace: String,
    pub event_type: String,
    pub metadata: BTreeMap<String, JsonValue>,
}

pub fn parse_mlperf_log_detail(text: &str) -> io::Result<Vec<MLLogEntry>> {
    let mut entries = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        if !line.starts_with(":::MLLOG") {
            continue;
        }

        // ":::MLLOG" 프리픽스 제거 후 JSON 파싱
        if let Some(json_str) = line.strip_prefix(":::MLLOG ") {
            match serde_json::from_str::<JsonValue>(json_str) {
                Ok(json) => {
                    if let JsonValue::Object(map) = json {
                        let entry = MLLogEntry {
                            key: map
                                .get("key")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            value: map.get("value").cloned().unwrap_or(JsonValue::Null),
                            time_ms: map
                                .get("time_ms")
                                .and_then(|v| v.as_f64())
                                .unwrap_or_default(),
                            namespace: map
                                .get("namespace")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            event_type: map
                                .get("event_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                            metadata: map
                                .get("metadata")
                                .and_then(|v| v.as_object())
                                .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                                .unwrap_or_default(),
                        };
                        entries.push(entry);
                    }
                }
                Err(e) => {
                    eprintln!("JSON 파싱 에러 (라인 {}): {}", line_no + 1, e);
                    continue;
                }
            }
        }
    }

    Ok(entries)
}

pub fn parse_mlperf_log_detail_file(file_path: &str) -> io::Result<Vec<MLLogEntry>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let text = reader.lines().collect::<io::Result<Vec<_>>>()?.join("\n");

    parse_mlperf_log_detail(&text)
}

pub fn save_log_detail_as_json(input_file: &str, output_file: &str) -> io::Result<()> {
    let entries = parse_mlperf_log_detail_file(input_file)?;

    // MLLogEntry를 JsonValue로 변환
    let json_entries: Vec<JsonValue> = entries
        .iter()
        .map(|entry| {
            let mut map = serde_json::Map::new();
            map.insert("key".to_string(), JsonValue::String(entry.key.clone()));
            map.insert("value".to_string(), entry.value.clone());
            map.insert(
                "time_ms".to_string(),
                JsonValue::Number(serde_json::Number::from_f64(entry.time_ms).unwrap()),
            );
            map.insert(
                "namespace".to_string(),
                JsonValue::String(entry.namespace.clone()),
            );
            map.insert(
                "event_type".to_string(),
                JsonValue::String(entry.event_type.clone()),
            );

            let metadata = JsonValue::Object(
                entry
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            );
            map.insert("metadata".to_string(), metadata);

            JsonValue::Object(map)
        })
        .collect();

    let mut output = File::create(output_file)?;
    serde_json::to_writer_pretty(&mut output, &json_entries)?;
    Ok(())
}

pub fn save_log_detail_as_yaml(input_file: &str, output_file: &str) -> io::Result<()> {
    let entries = parse_mlperf_log_detail_file(input_file)?;
    let mut output = File::create(output_file)?;
    serde_yaml::to_writer(&mut output, &entries)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}

pub fn save_log_detail(input_file: &str, output_file: &str, format: &str) -> io::Result<()> {
    match format {
        "json" => save_log_detail_as_json(input_file, output_file),
        "yaml" => save_log_detail_as_yaml(input_file, output_file),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid format. Use 'json' or 'yaml'.",
        )),
    }
}
