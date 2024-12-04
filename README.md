# MLPerf Log Parser

MLPerf loadgen 로그 파일을 파싱하여 JSON 또는 YAML 형식으로 변환하는 도구입니다.

## 기능

- MLPerf loadgen summary 로그 파일 파싱 (`*log_summary.txt`)
- MLPerf loadgen detail 로그 파일 파싱 (`*detail.txt`) 
- JSON 또는 YAML 형식으로 출력 지원

## 설치

```bash
cargo install https://github.com/furiosa-ai/mlperf-log-parser.git
```

## 사용법

### 명령행 인터페이스 (CLI)

```bash
# Summary 로그 파일을 JSON으로 변환
mlperf-log-parser summary -f json mlperf_log_summary.txt mlperf_log_summary.json

# Detail 로그 파일을 YAML로 변환
mlperf-log-parser detail -f yaml mlperf_log_detail.txt mlperf_log_detail.yaml
```

## 출력 형식

### JSON 출력 예시

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

### YAML 출력 예시

```yaml
test_parameters_used:
  batch_size: 32
  data_type: INT8
  dataset: ImageNet
  model: ResNet50 v1.5
  quality_target: 99%
  target_latency_ns: 10000000
```

## 라이선스

MIT License

## 기여하기

버그 리포트, 기능 제안, 풀 리퀘스트는 언제나 환영합니다.
이슈를 생성하거나 풀 리퀘스트를 보내주시기 바랍니다.

