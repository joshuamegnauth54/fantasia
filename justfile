set dotenv-filename := "dev.env"
set dotenv-load

default: test

test:
    cargo test --all -- --nocapture

