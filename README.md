# Loom
A rusty version of a federated oauth2 server.

## Prereqs
I use `wsl` for this. Docker is required.

sqlx is also required: `cargo install sqlx-cli`

During development make sure start redis and db daemon with
`docker compose up -d db redis`

Then you will need to run migrations.

## .env
Make sure you have a .env file.

Bare minimum out of the box is:
```bash
DATABASE_URL=mysql://root:example@127.0.0.1:3307/loom
REDIS_URL=redis://127.0.0.1:6379
DATABASE_PASS=example
DATABASE_ROOT=root
RUST_LOG=info
```

** IMPORTANT ** run `source .env`

## Migrations / Seeding

`sqlx migrate run --source ./db/migrations`
I am running on mysql v8, so make sure `mysql-cli` is installed.

Seed with `/bin/bash migrator.sh`

## Startup
`docker compose up -d db redis`

`docker compose up --build`

## Is it working?

go to http://localhost:3000/health in your browser, it should show data.