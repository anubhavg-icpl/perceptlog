use clap::Parser;
use perceptlog::cli::{cmd::cmd, Opts};

fn main() {
    std::process::exit(cmd(&Opts::parse(), perceptlog::stdlib::all()));
}
