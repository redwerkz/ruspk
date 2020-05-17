# spkrepo-rs
ruspk is a simple and fast synology repository server. It uses the existing database structure from [spkrepo](https://github.com/SynoCommunity/spkrepo)

Only the GET API for the synology devices is supported. You have to update the database yourself or run it in conjunction with spkrepo and a reverse proxy like nginx.

## Install

```sh
cargo install diesel_cli
cargo install ruspk --features postgres
echo 'DATABASE_URL=postgresql://user:pass@localhost/dbname' > .env
diesel migration --migration-dir migrations/postgres/ run
ruspk
```

Avaliable Features: `mysql`, `postgres` and `sqlite`

### Test the API

```sh
curl -sv 'http://127.0.0.1:8080/?build=24922&language=enu&major=6&micro=2&arch=x86&minor=2' | jq

```

## Configuration (`.env` file)

```env
RUST_LOG="ruspk=info,actix_web=info,actix_server=info,diesel=info"
DATABASE_URL=file:db/database.sqlite
# DATABASE_URL=mysql://user:pass@localhost/dbname
# DATABASE_URL=postgresql://user:pass@localhost/dbname
LISTEN=127.0.0.1
PORT=80
```

# Dev Guides

https://diesel.rs/

https://actix.rs/
