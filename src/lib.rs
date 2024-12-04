use serde_json::{json, Value};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

pub fn parse_mlperf_results(text: &str) -> Value {
    let mut sections = json!({
        "mlperf_result_summary": {},
        "additional_stats": {},
        "test_parameters_used": {},
        "message": []
    });

    let mut current_section = "";
    let mut extra_message: Vec<String> = Vec::new();

    for line in text.lines() {
        let line = line.trim();

        // Determine section
        if line.starts_with("MLPerf Results Summary") {
            current_section = "mlperf_result_summary";
            continue;
        } else if line.starts_with("Additional Stats") {
            current_section = "additional_stats";
            continue;
        } else if line.starts_with("Test Parameters Used") {
            current_section = "test_parameters_used";
            continue;
        } else if line.starts_with("===") {
            continue;
        } else if line.is_empty() {
            // The NEW of SECTION
            current_section = "";
            continue;
        }

        if current_section.is_empty() {
            extra_message.push(line.to_string());
        }
        // Parse key-value pairs
        else if let Some((key, value)) = line.split_once(':') {
            let mut key = key
                .trim()
                .to_lowercase()
                .replace("(", "")
                .replace(")", "")
                .replace(" ", "_");
            let value = value.trim();

            // Convert value to appropriate type
            let processed_value = match value.to_lowercase().as_str() {
                "yes" => json!(true),
                "no" => json!(false),
                _ => {
                    if let Ok(num) = value.parse::<i64>() {
                        json!(num)
                    } else if let Ok(num) = value.parse::<f64>() {
                        json!(num)
                    } else {
                        json!(value)
                    }
                }
            };

            // Handle special result fields
            if [
                "result_is",
                "min_duration_satisfied",
                "min_queries_satisfied",
                "early_stopping_satisfied",
            ]
            .contains(&key.as_str())
            {
                if key == "result_is" {
                    sections["mlperf_result_summary"]["result"] = json!({
                        "state": value,
                        "min_duration_satisfied": false,
                        "min_queries_satisfied": false,
                        "early_stopping_satisfied": false
                    });
                } else {
                    if let Some(obj) = sections["mlperf_result_summary"]["result"].as_object_mut() {
                        obj.insert(key.clone(), processed_value);
                    }
                }
            } else if current_section == "additional_stats" {
                // Handle percentile mappings
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

                sections[current_section][key] = processed_value;
            } else if !current_section.is_empty() {
                sections[current_section][key] = processed_value;
            }
        }
    }

    sections["message"] = json!(extra_message);
    sections
}

pub fn save_summary_as_json(summary_file: &str, output_file: &str) -> io::Result<()> {
    // Read the summary file
    let file = File::open(summary_file)?;
    let reader = BufReader::new(file);
    let text: String = reader.lines().collect::<io::Result<Vec<_>>>()?.join("\n");

    // Parse the content
    let summary = parse_mlperf_results(&text);

    // Write to output file
    let mut output = File::create(output_file)?;
    output.write_all(serde_json::to_string_pretty(&summary)?.as_bytes())?;

    Ok(())
}
