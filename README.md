# ahub

Access hub command-line utility.

## Setup

### Create .env in root and put DATABASE_URL in it for rust-analyzer.
```bash
DATABASE_URL="sqlite://db/dev.db"
```

###  Declare DATABASE_URL for cargo (relative path)
```bash
export DATABASE_URL="sqlite://db/dev.db"
```

###  Launch config for debugging
```json
"env": {
    "DATABASE_URL": "sqlite://db/dev.db",
}
```

### WSL
Use "$(hostname).local" or nameserver ip to connect to Windows localhost. Must open port on Windows.
```bash
echo "$(hostname).local"
cat /etc/resolv.conf
```    

## Cargo in root
```bash
cargo run -- --help
cargo run dump sqlite-version   
cargo run dump events
cargo run dump users -t2
cargo run mock grant -u1 -p1
cargo run mock deny -p1 -c666
cargo run heartbeat --host "http://$(hostname).local:3000"
```

## Sqlx CLI cheatsheet
```bash
cargo sqlx --help
cargo sqlx database setup
cargo sqlx migrate add schema
cargo sqlx migrate run
cargo sqlx database reset -y
```
Did you know you can embed your migrations in your application binary?
On startup, after creating your database connection or pool, add:

sqlx::migrate!().run(<&your_pool OR &mut your_connection>).await?;

Note that the compiler won't pick up new migrations if no Rust source files have changed.
You can create a Cargo build script to work around this with `sqlx migrate build-script`.

See: https://docs.rs/sqlx/0.5/sqlx/macro.migrate.html

## Sqlx CLI run using cargo
All commands require that a database url is provided. This can be done either with the `--database-url` command line option or by setting `DATABASE_URL`, either in the environment or in a `.env` file
in the current working directory.

For more details, run `cargo sqlx <command> --help`.

```dotenv
# Postgres
DATABASE_URL=postgres://postgres@localhost/my_database
```

#### Create/drop the database at `DATABASE_URL`

```bash
cargo sqlx database create
cargo sqlx database drop
```

#### Create and run migrations

```bash
$ cargo sqlx migrate add <name>
```
Creates a new file in `migrations/<timestamp>-<name>.sql`. Add your database schema changes to
this new file.

---
```bash
$ cargo sqlx migrate run
```
Compares the migration history of the running database against the `migrations/` folder and runs
any scripts that are still pending.

#### Reverting Migrations

If you would like to create _reversible_ migrations with corresponding "up" and "down" scripts, you use the `-r` flag when creating new migrations:

```bash
$ cargo sqlx migrate add -r <name>
Creating migrations/20211001154420_<name>.up.sql
Creating migrations/20211001154420_<name>.down.sql
```

After that, you can run these as above:

```bash
$ cargo sqlx migrate run
Applied migrations/20211001154420 <name> (32.517835ms)
```

And reverts work as well:

```bash
$ cargo sqlx migrate revert
Applied 20211001154420/revert <name>
```

**Note**: attempting to mix "simple" migrations with reversible migrations with result in an error.

```bash
$ cargo sqlx migrate add <name1>
Creating migrations/20211001154420_<name>.sql

$ cargo sqlx migrate add -r <name2>
error: cannot mix reversible migrations with simple migrations. All migrations should be reversible or simple migrations
```

#### Enable building in "offline mode" with `query!()`

Note: must be run as `cargo sqlx`.

```bash
cargo sqlx prepare
```

Saves query metadata to `sqlx-data.json` in the current directory; check this file into version
control and an active database connection will no longer be needed to build your project.

Has no effect unless the `offline` feature of `sqlx` is enabled in your project. Omitting that
feature is the most likely cause if you get a `sqlx-data.json` file that looks like this:

```json
{
    "database": "PostgreSQL"
}
```

---

```bash
cargo sqlx prepare --check
```

Exits with a nonzero exit status if the data in `sqlx-data.json` is out of date with the current
database schema and queries in the project. Intended for use in Continuous Integration.

#### Force building in offline mode

To make sure an accidentally-present `DATABASE_URL` environment variable or `.env` file does not
result in `cargo build` (trying to) access the database, you can set the `SQLX_OFFLINE` environment
variable to `true`.

If you want to make this the default, just add it to your `.env` file. `cargo sqlx prepare` will
still do the right thing and connect to the database.

#### Include queries behind feature flags (such as queryies inside of tests)

In order for sqlx to be able to find queries behind certain feature flags you need to turn them
on by passing arguments to rustc.

This is how you would turn all targets and features on.
```bash
cargo sqlx prepare -- --all-targets --all-features
```

## Sqlite

- dump: 3.37.0 released 2021-11-27, no unixepoch()
- 2022-02-22 (3.38.0) has unixepoch()
- 2022-03-12 (3.38.1) most recent version
- libsqlite3-sys 0.24.1 should have bundled sqlite 3.38.0
- libsqlite3-sys/sqlite3/bindgen_bundled_version.rs
- bumped to 3.38.1 but not released yet https://github.com/rusqlite/rusqlite/commit/c3b419b1e53925c02e35a0dde019727153e1e6a8
- sqlx has libsqlite3-sys 0.23.2
- https://crates.io/crates/libsqlite3-sys/0.23.2
-  currently SQLite 3.36.0 (as of rusqlite 0.26.0 / libsqlite3-sys 0.23.0).
- https://github.com/rusqlite/rusqlite/releases
-  libsqlite3-sys 0.24.1 (latest) has sqlite 3.38.0 bundled
- libsqlite3-sys-v0.23.1: SQLITE_VERSION_NUMBER: i32 = 3036000;
- 11/28/2021: https://github.com/rusqlite/rusqlite/commit/795a53d3682d5daf0b31f9a37eac4052c55558ca
-  https://github.com/rusqlite/rusqlite/commit/795a53d3682d5daf0b31f9a37eac4052c55558ca
- debian bullseye sqlite version: 3.34.1