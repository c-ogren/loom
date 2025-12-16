#!/bin/bash
source .env

for f in db/seeds/*.sql; do
    echo "Running $f..."
    mysql -h 127.0.0.1 -P 3307 -u "$DATABASE_ROOT" -p"$DATABASE_PASS" loom < "$f"
done