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

- WSL - use nameserver ip to connect to Windows localhost. Must open port on Windows.
    ```
    cat /etc/resolv.conf
    ```    

- Cargo in root
    ```
    cargo run -- --help
    cargo run -- dump events
    cargo run -- dump users -t2
    ```