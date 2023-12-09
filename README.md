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
