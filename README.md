# ahub - Access Hub CLI

## Setup

- Create .env in root and put DATABASE_URL in it for rust-analyzer.
    ```
    DATABASE_URL="sqlite://db/dev.db"
    ```

- Declare DATABASE_URL for cargo (relative path)
    ```
    export DATABASE_URL="sqlite://db/dev.db"
    ```
- Launch config for debugging
    ```
    "env": {
                "DATABASE_URL": "sqlite://db/dev.db",
            }
    ```

- WSL - use "$(hostname).local" or nameserver ip to connect to Windows localhost. Must open port on Windows.
    ```
    echo "$(hostname).local"
    cat /etc/resolv.conf
    ```    

- Cargo in root
    ```
    cargo run -- --help
    cargo run dump sqlite-version   
    cargo run dump events
    cargo run dump users -t2
    cargo run heartbeat --host "http://$(hostname).local:3000"
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