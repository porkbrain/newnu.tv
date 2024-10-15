# This justfile contains recipes for convieniently running the project locally
# in docker containers.

[private]
default:
  @just --list --no-aliases

# Runs a docker compose command with the local docker compose file.
[no-cd]
@compose +args:
  docker compose -f docker-compose.local.yml {{args}}

# Compiles the project and starts the docker containers with docker compose.
[no-cd]
@up:
    cross build --bin worker
    cross build --bin admin
    just compose up --build

# Checks the project for file changes and reloads containers that have changed.
[no-cd]
@watch:
  ./dev/watch_and_compose_up.sh
