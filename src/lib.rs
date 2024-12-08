pub mod log_detail;
pub mod log_summary;

pub use log_detail::{
    parse_mlperf_log_detail, parse_mlperf_log_detail_file, save_log_detail, MLLogEntry,
};
pub use log_summary::{parse_log_summary, parse_mlperf_results_file, save_summary};
