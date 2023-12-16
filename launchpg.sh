#!/bin/sh

# Script based off of: https://github.com/LukeMathWalker/zero-to-production/blob/main/scripts/init_db.sh
# (from Zero to Production)

usage() {
	echo "$0 path-to-compose-conf path-to-env path-to-migrations"
	echo "Set SKIP_DOCKER=true to skip launching Docker or Podman"
}

stopDocker() {
	docker compose -f "$COMPOSE_CONF" --env-file "$ENV_PATH" down
}

# Set default arguments if needed
if [ "${1}" = "" ]; then
	COMPOSE_CONF="containers/postgres/compose.yaml"
	echo "Compose config path default: ${COMPOSE_CONF}"
else
	COMPOSE_CONF="${1}"
	echo "Compose config path override: ${COMPOSE_CONF}"
fi

if [ "${2}" = "" ]; then
	ENV_PATH="$(dirname "${0}")/.env"
	echo "Env file path default: ${ENV_PATH}"
else
	ENV_PATH="${2}"
	echo "Env file path override: ${ENV_PATH}"
fi

if [ "${3}" = "" ]; then
	MIGRATIONS_PATH="$(dirname "${0}")/migrations"
	echo "Migration path default: ${MIGRATIONS_PATH}"
else
	MIGRATIONS_PATH="${3}"
	echo "Migration path override: ${MIGRATIONS_PATH}"
fi

# Check for required binaries
if ! [ -x "$(command -v pg_isready)" ]; then
	printf "Missing: \"pg_isready\"\n"
	printf "Install the postgres package for your distro\n"
	exit 1
fi

# sqlx for migrations
if ! [ -x "$(command -v sqlx)" ]; then
	printf "Missing: \"sqlx\"\n"
	printf "Install Rust/cargo with:\n"
	printf "\tcargo install sqlx-cli --no-default-features --features rustls,postgres\n"
fi

# .env is required for containerized postgres
if ! [ -e "$ENV_PATH" ] && [ "$SKIP_DOCKER" = "" ] || [ "$SKIP_DOCKER" = "false" ]; then
	printf "Missing: \".env\"\n"
	printf "You need an environment file to pass to Docker for postgres\n"
	usage
	exit 1
fi

# Docker compose config is required if using Docker
if ! [ -e "$COMPOSE_CONF" ] && [ "$SKIP_DOCKER" = "" ] || [ "$SKIP_DOCKER" = false ]; then
	printf "Missing a Docker Compose configuration file\n"
	usage
	exit 1
fi

# Source .env for pg_isready if DATABASE_URL isn't set
if [ "$DATABASE_URL" = "" ]; then
	. "$ENV_PATH"
	export DATABASE_URL
fi

if [ "$SKIP_DOCKER" != true ]; then
	# Launch docker and detach so that it runs in the background
	if ! docker compose -f "$COMPOSE_CONF" --env-file "$ENV_PATH" up --detach; then
		echo "Unable to start postgres container via Docker"
		printf "\tCompose path: %s\n" "$COMPOSE_CONF"
		exit 1
	fi
fi

# Retry connecting to the postgres server 25 times
RETRY_COUNT=25
until pg_isready -d "$DATABASE_URL"; do
	if [ "$RETRY_COUNT" -eq 0 ]; then
		echo "Failed connecting to Postgres - retry limit reached"
		if [ "$SKIP_DOCKER" != true ]; then
			echo "Check Docker's logs for this container."
			docker compose -f "$COMPOSE_CONF" --env-file "$ENV_PATH" logs --timestamps
			stopDocker
		else
			echo "Make sure that the server is up and responsive"
		fi
		exit 1
	else
		echo "Waiting for postgres server to be ready (${PGHOST}:${PGPORT})"
		echo "Retry #${RETRY_COUNT}"
		RETRY_COUNT=$((RETRY_COUNT - 1))
		sleep 2
	fi
done

echo "Running sqlx migrations"
if ! sqlx database create --database-url "$DATABASE_URL"; then
	echo "Migrations: Failed to create database"
	stopDocker
	exit 1
fi

if ! sqlx migrate run --database-url "$DATABASE_URL" --source "$MIGRATIONS_PATH"; then
	echo "Migrations: Failed to run migrations"
	stopDocker
	exit 1
fi

# Server started; tail logs
echo "Postgres server is ready"
docker compose -f "$COMPOSE_CONF" --env-file "$ENV_PATH" logs --follow --timestamps
