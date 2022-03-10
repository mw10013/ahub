use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    // #[clap(subcommand)]
// command: Commands,
}

fn main() {
    let _args = Cli::parse();
}
