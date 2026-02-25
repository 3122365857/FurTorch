//! TorchRust CLI - reads JSON from stdin, processes log, writes JSON to stdout.
//! Used by Node.js server via subprocess.

use torchrust_core::process::{process_log_chunk, ProcessInput};
use std::io::{self, Read};

fn main() {
    let mut stdin = io::stdin();
    let mut input = String::new();
    stdin.read_to_string(&mut input).expect("Failed to read stdin");

    let process_input: ProcessInput =
        serde_json::from_str(&input).expect("Failed to parse input JSON");

    let output = process_log_chunk(process_input);
    let json = serde_json::to_string(&output).expect("Failed to serialize output");
    println!("{}", json);
}
