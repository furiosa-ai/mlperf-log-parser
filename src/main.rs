use mlperf_log_parser::{save_log_detail, save_summary};
use std::fs;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

fn validate_summary_input_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    // 파일 존재 여부 확인
    if !path.exists() {
        return Err(format!("파일이 존재하지 않습니다: {}", path.display()));
    }

    // .txt 확장자 확인
    if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
        return Err(format!(
            "입력 파일은 .txt 확장자여야 합니다: {}",
            path.display()
        ));
    }

    // loadgen 검증
    let content =
        fs::read_to_string(path).map_err(|e| format!("파일을 읽을 수 없습니다: {}", e))?;

    if !content.contains("MLPerf Results Summary") {
        return Err("MLPerf loadgen summary 로그 형식이 아닙니다.".to_string());
    }

    Ok(())
}

fn validate_detail_input_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    // 파일 존재 여부 확인
    if !path.exists() {
        return Err(format!("파일이 존재하지 않습니다: {}", path.display()));
    }

    // .txt 확장자 확인
    if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
        return Err(format!(
            "입력 파일은 .txt 확장자여야 합니다: {}",
            path.display()
        ));
    }

    // loadgen 검증
    let content =
        fs::read_to_string(path).map_err(|e| format!("파일을 읽을 수 없습니다: {}", e))?;

    if !content.contains(":::MLLOG") {
        return Err("MLPerf loadgen detail 로그 형식이 아닙니다.".to_string());
    }

    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "mlperf-log-parser", author, about)]
pub enum Cli {
    /// Performance 관련 로그 파싱 (MLPerf loadgen 으로 생성된 *log_summary.txt 파일 필요)
    Performance {
        /// Input file path (ex, *log_summary.txt MLPerf loadgen 로그 파일)
        #[structopt(parse(from_os_str), validator = validate_summary_input_file)]
        input_file: PathBuf,

        /// Output file path (.json | .yaml 파일)
        #[structopt(parse(from_os_str))]
        output_file: PathBuf,

        /// Output format (.json | .yaml)
        #[structopt(short, long, default_value = "json")]
        format: String,
    },

    /// 상세 로그 파싱
    LogDetail {
        /// Input file path
        #[structopt(parse(from_os_str), validator = validate_detail_input_file)]
        input_file: std::path::PathBuf,

        /// Output file path
        #[structopt(parse(from_os_str))]
        output_file: std::path::PathBuf,

        /// Output format (.json | .yaml)
        #[structopt(short, long, default_value = "json")]
        format: String,
    },
}

impl std::fmt::Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cli::Performance { .. } => write!(f, "performance"),
            Cli::LogDetail { .. } => write!(f, "log_detail"),
        }
    }
}

fn main() {
    let cli = Cli::from_args();

    match &cli {
        Cli::Performance {
            input_file,
            output_file,
            format,
        } => {
            if let Err(e) = save_summary(
                input_file.to_str().unwrap(),
                output_file.to_str().unwrap(),
                format,
            ) {
                eprintln!("에러: {}", e);
                process::exit(1);
            }
            println!(
                "{} 명령어로 {} 파일을 파싱하여 {}에 저장했습니다",
                cli.to_string(),
                input_file.display(),
                output_file.display()
            );
        }
        Cli::LogDetail {
            input_file,
            output_file,
            format,
        } => {
            if let Err(e) = save_log_detail(
                input_file.to_str().unwrap(),
                output_file.to_str().unwrap(),
                format,
            ) {
                eprintln!("에러: {}", e);
                process::exit(1);
            }
        }
    }
}
