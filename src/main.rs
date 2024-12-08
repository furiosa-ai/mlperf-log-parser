use env_logger;
use mlperf_log_parser::{log_detail::save_log_detail, log_summary::save_summary};
use std::fs;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;

fn validate_summary_input_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    // Check if file exists
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    // Check .txt extension
    if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
        return Err(format!(
            "Input file must have .txt extension: {}",
            path.display()
        ));
    }

    // Validate loadgen format
    let content = fs::read_to_string(path).map_err(|e| format!("Could not read file: {}", e))?;

    if !content.contains("MLPerf Results Summary") {
        return Err("Not a valid MLPerf loadgen summary log format".to_string());
    }

    Ok(())
}

fn validate_detail_input_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    // Check if file exists
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    // Check .txt extension
    if path.extension().and_then(|ext| ext.to_str()) != Some("txt") {
        return Err(format!(
            "Input file must have .txt extension: {}",
            path.display()
        ));
    }

    // Validate loadgen format
    let content = fs::read_to_string(path).map_err(|e| format!("Could not read file: {}", e))?;

    if !content.contains(":::MLLOG") {
        return Err("Not a valid MLPerf loadgen detail log format".to_string());
    }

    Ok(())
}

#[derive(StructOpt)]
#[structopt(name = "mlperf-log-parser", author, about)]
pub enum Cli {
    /// Parse performance related logs (Requires *log_summary.txt file generated by MLPerf loadgen)
    LogSummary {
        /// Input file path (ex, *log_summary.txt MLPerf loadgen log file)
        #[structopt(parse(from_os_str), validator = validate_summary_input_file)]
        input_file: PathBuf,

        /// Output file path (.json | .yaml file)
        #[structopt(parse(from_os_str))]
        output_file: PathBuf,

        /// Output format (.json | .yaml)
        #[structopt(short, long, default_value = "json")]
        format: String,
    },

    /// Parse detailed logs
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
            Cli::LogSummary { .. } => write!(f, "log_summary"),
            Cli::LogDetail { .. } => write!(f, "log_detail"),
        }
    }
}

fn main() {
    env_logger::init();
    let cli = Cli::from_args();

    match &cli {
        Cli::LogSummary {
            input_file,
            output_file,
            format,
        } => {
            if let Err(e) = save_summary(
                input_file.to_str().unwrap(),
                output_file.to_str().unwrap(),
                format,
            ) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            println!(
                "Command {} parsed {} file and saved to {}",
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
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            println!(
                "Command {} parsed {} file and saved to {}",
                cli.to_string(),
                input_file.display(),
                output_file.display()
            );
        }
    }
}
