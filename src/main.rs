use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dump database
    // #[clap(arg_required_else_help = true)]
    Dump {
        #[clap(subcommand)]
        command: DumpCommands,
    },
    /// Mock access
    #[clap(arg_required_else_help = true)]
    Mock {
        /// The remote to target
        remote: String,
    },
}

#[derive(Subcommand, Debug)]
enum DumpCommands {
    /// Dump events
    // #[clap(arg_required_else_help = true)]
    Events {
        /// Number of events to take
        #[clap(short, long, parse(try_from_str), default_value_t = 10)]
        take: usize,
    },
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Dump { command } => {
            println!("Dumping {:?}", command);
        }
        Commands::Mock { remote } => {
            println!("Pushing to {}", remote);
        }
    }
}
