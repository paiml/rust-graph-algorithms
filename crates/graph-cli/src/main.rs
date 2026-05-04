//! `graph` CLI — thin shell over [`graph_cli::run`].

use clap::Parser;
use graph_cli::{run, Cli};
use std::io::{stdin, stdout};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli, stdin().lock(), stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("graph: {e}");
            ExitCode::FAILURE
        }
    }
}
