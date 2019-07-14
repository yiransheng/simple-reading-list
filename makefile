

build-dev:
	mkdir -p ./assets/js && \
	cd sitejs && npm run build && \
	cp ./dist/*.js ../assets/js && \
	cp ./dist/*.js.map ../assets/js

dev: build-dev
	cargo run --bin server & \
	toshi -c toshi_config.toml & \
	cd admin-ui && yarn start & \
	caddy

.PHONY: clean

clean:
	rm -rf assets/js/*
