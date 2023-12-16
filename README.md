[![license](https://img.shields.io/github/license/joshuamegnauth54/fantasia)](https://github.com/joshuamegnauth54/fantasia/blob/main/LICENSE)
[![tokei](https://tokei.rs/b1/github/joshuamegnauth54/fantasia)](https://github.com/XAMPPRocky/tokei)

# Fantasia

# Hosting a server

# Clients

# Configuration

`Fantasia` is configured through CLI options, environmental variables, and a config file.
This is also the order of precedence for configs. CLI opts override env vars which overrides the config.

## CLI options

## Environmental variables

| Variables                         | Description           |
| ---                               | ---                   |
| `POSTGRES_USER` `PGUSER`          | Superuser             |
| `POSTGRES_PASSWORD` `PGPASSWORD`  | Superuser password    |
| `PGHOST`                          | Postgres host         |
| `PGPORT`                          | Port for host         |
| `POSTGRES_DB` `PGDATABASE`        | Database name         |

## Config file

`Fantasia`'s config file is a [TOML](https://toml.io/en/) with the following tables and keys.

```toml
[fantasia]
# Interface IP to bind the server instance
host = "localhost"
# Port to bind on host
port = 8000
# Override `.env` file
env_file = ".env"

[postgres]
# Postgres superuser
user = "postgres"
# Postgres superuser password
password = "postgres"
# Postgres server host
host = "localhost"
# Postgres server port
port = 5432
# Postgres database name
database = "pgdb"

# See: https://docs.rs/sqlx/latest/sqlx/pool/struct.PoolOptions.html
[postgres.options]
test_before_acquire = true
acquire_timeout_seconds = 10
min_connections = 0
max_connections = 10
max_lifetime_seconds = 180
idle_timeout_seconds = 600
```
