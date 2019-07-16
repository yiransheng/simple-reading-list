build-server-docker:
	docker build -t reads.yiransheng.com/server:latest .

build-toshi-docker:
	cp toshi_config.toml __toshi__version ./Toshi/ && \
	cp toshi.Dockerfile ./Toshi/Dockerfile && \
	pushd ./Toshi && \
	git checkout $(cat __toshi__version) &&
	docker build -t reads.yiransheng.com/toshi_bin:latest . && \
	popd


build-dev:build-js

build-js:
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
