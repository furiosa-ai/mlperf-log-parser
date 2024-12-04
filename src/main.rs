use mlperf_log_parser::save_summary_as_json;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];

    if let Err(e) = save_summary_as_json(input_file, output_file) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    println!(
        "Successfully parsed MLPerf results from {} to {}",
        input_file, output_file
    );
}
