```README.md
// Start of Selection
# MLPerf Log Parser

A tool to parse MLPerf loadgen log files and convert them to JSON or YAML format.

## Features

- Parse MLPerf loadgen summary log files (`*log_summary.txt`)
- Parse MLPerf loadgen detail log files (`*detail.txt`)
- Support output in JSON or YAML format

## Installation

```bash
cargo install --git https://github.com/furiosa-ai/mlperf-log-parser.git --branch v4.1
```

## Usage

### Command Line Interface (CLI)

```bash
# Convert summary log file to JSON
mlperf-log-parser log-summary -f json mlperf_log_summary.txt mlperf_log_summary.json

# Convert detail log file to YAML
mlperf-log-parser log-detail -f yaml mlperf_log_detail.txt mlperf_log_detail.yaml
```

## Output Format

### Example JSON Output

```json
{
  "test_parameters_used": {
    "batch_size": 32,
    "data_type": "INT8",
    "dataset": "ImageNet",
    "model": "ResNet50 v1.5",
    "quality_target": "99%",
    "target_latency_ns": 10000000
  }
}
```

### Example YAML Output

```yaml
test_parameters_used:
  batch_size: 32
  data_type: INT8
  dataset: ImageNet
  model: ResNet50 v1.5
  quality_target: 99%
  target_latency_ns: 10000000
```

## License

MIT License

## Contributing

Bug reports, feature suggestions, and pull requests are always welcome.
Please create an issue or submit a pull request.

