use clap::{Parser, Subcommand};
use sqlx::sqlite::SqlitePool;
use std::env;

mod domain;
mod dump;
mod mock;
mod heartbeat;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Dump database
    Dump {
        #[clap(subcommand)]
        command: DumpCommands,
    },
    /// Mock access
    Mock {
        #[clap(subcommand)]
        command: MockCommands,
    },
    /// Post heartbeat to access cloud
    Heartbeat {
        #[clap(short = 'o', long, default_value_t = String::from("http://localhost:3000"))]
        host: String,
    },
}

#[derive(Subcommand, Debug)]
enum DumpCommands {
    /// Dump events
    Events {
        /// Number of events to take
        #[clap(short, long, parse(try_from_str), default_value_t = 10)]
        take: i32,

        /// Number of events to skip
        #[clap(short, long, parse(try_from_str), default_value_t = 0)]
        skip: i32,
    },
    Users {
        /// Number of users to take
        #[clap(short, long, parse(try_from_str), default_value_t = 50)]
        take: i32,

        /// Number of users to skip
        #[clap(short, long, parse(try_from_str), default_value_t = 0)]
        skip: i32,

        /// Swap codes of first two access users
        #[clap(short = 'w', long)]
        swap: bool,
    },
    SqliteVersion {
    }
}

#[derive(Subcommand, Debug)]
enum MockCommands {
    /// Mock grant
    Grant {
        /// Point id
        #[clap(short, long, parse(try_from_str))]
        point: i64,

        /// User id
        #[clap(short, long, parse(try_from_str))]
        user: i64,
    },
    /// Mock deny
    Deny {
        /// Point id
        #[clap(short, long, parse(try_from_str))]
        point: i64,

        /// Code
        #[clap(short, long)]
        code: String,
    },
}

// #[async_std::main]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    match args.command {
        Commands::Dump { command } => match command {
            DumpCommands::Events { take, skip } => {
                dump::dump_events(take, skip, &pool).await?;
            }
            DumpCommands::Users { take, skip, swap } => {
                dump::dump_users(take, skip, swap, &pool).await?;
            }
            DumpCommands::SqliteVersion {} => {
                dump::dump_sqlite_version(&pool).await?;
            }
        },
        Commands::Mock { command } => match command {
            MockCommands::Grant { point, user } => {
                mock::grant(user, point, &pool).await?;
            }
            MockCommands::Deny { point, code } => {
                mock::deny(point, code, &pool).await?;
            }
        },
        Commands::Heartbeat { host } => {
            heartbeat::heartbeat(host).await?
        }
    }
    Ok(())
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
