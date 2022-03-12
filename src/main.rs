use clap::{Parser, Subcommand};
use sqlx::sqlite::SqlitePool;
use std::env;

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
}

#[derive(Subcommand, Debug)]
enum MockCommands {
    /// Mock grant
    Grant {
        /// Point id
        #[clap(short, long, parse(try_from_str))]
        point: usize,

        /// User id
        #[clap(short, long, parse(try_from_str))]
        user: usize,
    },
    /// Mock deny
    Deny {
        /// Point id
        #[clap(short, long, parse(try_from_str))]
        point: usize,

        /// Code
        #[clap(short, long)]
        code: String,
    },
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;

    match &args.command {
        Commands::Dump { command } => {
            println!("Dumping {:?}", command);
            match command {
                DumpCommands::Events { take, skip } => {
                    println!("Events take: {} skip: {}", take, skip);

                    let recs = sqlx::query!(
                        "
                SELECT id, at, access, code, accessUserId, accessPointId
                FROM AccessEvent
                ORDER BY at DESC LIMIT ? OFFSET ?",
                        take,
                        skip
                    )
                    .fetch_all(&pool)
                    .await?;

                    for rec in recs {
                        println!("{:?}", rec);
                    }
                }
                DumpCommands::Users { take, skip, swap } => {
                    println!("Users take: {} skip: {} swap: {}", take, skip, swap);
                    let recs = sqlx::query!(
                        "
select id, name, code, activateCodeAt, expireCodeAt 
from AccessUser order by id asc limit ? offset ?",
                        take,
                        skip
                    )
                    .fetch_all(&pool)
                    .await?;

                    for rec in recs {
                        println!("{:?}", rec);
                    }
                }
            }
        }
        Commands::Mock { command } => {
            println!("Mocking {:?}", command);
        }
        Commands::Heartbeat { host } => {
            println!("Heartbeat {:?}", host)
        }
    }
    Ok(())
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
