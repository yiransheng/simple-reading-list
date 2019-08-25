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

1. Build images

```
make docker
```

2. Edit `deployment/.env`

Particularly, set `CADDY_HOST` and `ALLOWED_ORIGIN` to production DNS. If these ENV variables are left to their defaults, `docker-compose` will serve on `localhost:3000` with TLS off.

3. Start with `docker-compose`

```
docker-compose up -d
```

4. Create admin user from command line

```
docker ps
# copy container id for server
docker exec -it <container_id> /bin/ash
# run in container
/create-admin-user -u <username> -p <password>
```

5. Login at `/admin`



Note: `docker-push-ssh` (`pip2 install docker-push-ssh`) can be used to push images to server over ssh.