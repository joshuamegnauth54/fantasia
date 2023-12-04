# Finished
* Port to [axum 0.7](https://tokio.rs/blog/2023-11-27-announcing-axum-0-7-0)
* Bind to multiple IP addresses instead of just the first
* Better app spawning in tests
* Wrap `axum`'s `Serve` future in a struct that includes the bound socket address

# Unfinished
* Better handling of `DATABASE_URL` for logs. Currently it's _all_ `redacted` instead of just the secret
* Better logging (log to file et cetera).
* TLS
* Update `launchpg.sh` to handle skipping Docker (like in `Zero2Prod`)
* Update CI/CD to skip Docker
* Serde for PgPoolOptions
* Add `Tower`'s tracing middleware
