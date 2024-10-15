#!/bin/bash

# Run cargo watch for the admin service in the background
cargo watch --ignore "services/worker" \
    --shell "cross build --bin admin" \
    --shell "just compose up --quiet-pull --detach --build admin" &
ADMIN_PID=$!

# Run cargo watch for the worker service in the background
cargo watch --ignore "services/admin" \
    --shell "cross build --bin worker" \
    --shell "just compose up --quiet-pull --detach --build worker" &
WORKER_PID=$!

cleanup() {
    echo "Stopping background tasks..."
    kill $ADMIN_PID $WORKER_PID
    wait $ADMIN_PID $WORKER_PID 2>/dev/null
    echo "Background tasks stopped."
}
trap cleanup SIGINT

# Wait for background tasks to exit or until Ctrl+C is pressed
wait $ADMIN_PID $WORKER_PID
