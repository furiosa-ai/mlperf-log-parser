mod log_detail;
mod summary;

pub use log_detail::{
    parse_mlperf_log_detail, parse_mlperf_log_detail_file, save_log_detail, MLLogEntry,
};
pub use summary::{parse_mlperf_results, save_summary};
