use log::warn;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

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
            warn!("Invalid line[{}]: {}", line_no, line);
            continue;
        }

        // Remove ":::MLLOG" prefix and parse JSON
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
                    eprintln!("JSON parsing error (line {}): {}", line_no + 1, e);
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

pub fn save_log_detail_as_json<W: io::Write>(input_file: &str, output: &mut W) -> io::Result<()> {
    let entries = parse_mlperf_log_detail_file(input_file)?;
    serde_json::to_writer_pretty(output, &entries)?;
    Ok(())
}

pub fn save_log_detail_as_yaml<W: io::Write>(input_file: &str, output: &mut W) -> io::Result<()> {
    let entries = parse_mlperf_log_detail_file(input_file)?;
    serde_yaml::to_writer(output, &entries).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}

pub fn save_log_detail<W: io::Write>(
    input_file: &str,
    output: &mut W,
    format: &str,
) -> io::Result<()> {
    match format {
        "json" => save_log_detail_as_json(input_file, output),
        "yaml" => save_log_detail_as_yaml(input_file, output),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid format. Use 'json' or 'yaml'.",
        )),
    }
}
