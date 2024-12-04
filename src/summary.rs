use serde_value::Value;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

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

pub fn parse_mlperf_results_file(file_path: &str) -> io::Result<Value> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let text = reader.lines().collect::<io::Result<Vec<_>>>()?.join("\n");
    Ok(parse_mlperf_results(&text))
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
