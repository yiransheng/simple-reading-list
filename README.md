# A Bookmark App

A very simple reading list/bookmark app.

## Components

* Server: actix-web + diesel, SSR
* DB: Postgresql
* [Toshi](https://github.com/toshi-search/Toshi): a full text search engine
* sitejs: small js enhancement over server-rendered webpages
* admin-ui: react app for creating bookmarks



## Dev

**Deps**:

* rust tool chain (rustc, cargo, rustup)
* docker
* node & yarn
* [Caddy](https://caddy.community)


### Setup Database (one time)

```
sudo -u postgres psql
postgres=# create database insightful;
postgres=# create user readsdev with encrypted password 'thepass';
postgres=# grant all privileges on insightful to readsdev;
```

Run server in dev mode, toshi inside docker and admin-ui with `yarn start`.

```
make dev
```

## Deployment

```
make docker
```

`docker-compose.yaml` under construction.
