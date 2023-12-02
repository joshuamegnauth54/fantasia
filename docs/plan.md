# Finished
* Port to [axum 0.7](https://tokio.rs/blog/2023-11-27-announcing-axum-0-7-0)

# Unfinished
* Better handling of `DATABASE_URL` for logs. Currently it's _all_ `redacted` instead of just the secret
* Bind to multiple IP addresses instead of just the first
* Better logging (log to file et cetera).
* Better app spawning in tests
* TLS