use clap::{Parser, Subcommand};
use sqlx::{Connection, SqliteConnection};

mod access;
mod domain;
mod dump;
mod heartbeat;
mod mock;
mod sandbox;
mod token;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Dump database
    Dump {
        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,

        #[clap(subcommand)]
        command: DumpCommand,
    },
    /// Mock access
    Mock {
        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,

        #[clap(subcommand)]
        command: MockCommand,
    },
    /// Post heartbeat to access cloud
    Heartbeat {
        /// Access cloud host
        #[clap(short = 'a', long, env)]
        access_api_url: String,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },
    /// API token
    Token {
        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,

        /// Set
        #[clap(short, long, default_value_t = String::from(""))]
        set: String,
    },
    /// Access with code for point at position. Position is 1-based. Returns "GRANT" | "DENY"
    Access {
        /// Code
        #[clap(short, long)]
        code: String,

        /// Point position (1-based)
        #[clap(short, long, parse(try_from_str))]
        position: i64,

        /// Location of the DB, by default will be read from the DATABASE_URL env var
        #[clap(long, short = 'D', env)]
        database_url: String,
    },
}

#[derive(Subcommand, Debug)]
enum DumpCommand {
    /// Dump hub
    Hub {},
    /// Dump points
    Points {
        /// Number of users to take
        #[clap(short, long, parse(try_from_str), default_value_t = 50)]
        take: i32,

        /// Number of users to skip
        #[clap(short, long, parse(try_from_str), default_value_t = 0)]
        skip: i32,
    },
    /// Dump users
    Users {
        /// Number of users to take
        #[clap(short, long, parse(try_from_str), default_value_t = 50)]
        take: i32,

        /// Number of users to skip
        #[clap(short, long, parse(try_from_str), default_value_t = 0)]
        skip: i32,
    },
    /// Dump events
    Events {
        /// Number of events to take
        #[clap(short, long, parse(try_from_str), default_value_t = 10)]
        take: i32,

        /// Number of events to skip
        #[clap(short, long, parse(try_from_str), default_value_t = 0)]
        skip: i32,
    },
    /// Dump active codes
    Codes {},
    /// Dump sqlite version
    SqliteVersion {},
}

#[derive(Subcommand, Debug)]
enum MockCommand {
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
    /// Swap codes of first two access users. Way to test recycled codes.
    Swap {},
}

// #[async_std::main]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let args = Cli::parse();
    match args.command {
        Command::Dump {
            database_url,
            command,
        } => {
            let mut conn = SqliteConnection::connect(&database_url).await?;
            match command {
                DumpCommand::Hub {} => {
                    dump::dump_hub(&mut conn).await?;
                }
                DumpCommand::Points { take, skip } => {
                    dump::dump_points(take, skip, &mut conn).await?;
                }
                DumpCommand::Users { take, skip } => {
                    dump::dump_users(take, skip, &mut conn).await?;
                }
                DumpCommand::Events { take, skip } => {
                    dump::dump_events(take, skip, &mut conn).await?;
                }
                DumpCommand::Codes {} => {
                    dump::dump_codes(&mut conn).await?;
                }
                DumpCommand::SqliteVersion {} => {
                    dump::dump_sqlite_version(&mut conn).await?;
                }
            }
        }
        Command::Mock {
            database_url,
            command,
        } => {
            let mut conn = SqliteConnection::connect(&database_url).await?;
            match command {
                MockCommand::Grant { point, user } => {
                    mock::grant(user, point, &mut conn).await?;
                }
                MockCommand::Deny { point, code } => {
                    mock::deny(point, code, &mut conn).await?;
                }
                MockCommand::Swap {} => {
                    mock::swap(&mut conn).await?;
                }
            }
        }
        Command::Token { database_url, set } => token::token(&set, &database_url).await?,
        Command::Heartbeat {
            access_api_url,
            database_url,
        } => heartbeat::heartbeat(&access_api_url, &database_url).await?,
        Command::Access {
            code,
            position,
            database_url,
        } => access::access(&code, position, &database_url).await?,
    }
    Ok(())
}

#[test]
fn verify_app() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
