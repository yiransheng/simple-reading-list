SERVER_BIN := reads.yiransheng.com/server
TOSHI_BIN := reads.yiransheng.com/toshi_bin

build-server-docker:
	docker build -t $(SERVER_BIN):latest .

build-toshi-docker: TOSHI_VERSION=$(shell cat __toshi_version)
build-toshi-docker: __toshi_version
	cd ./Toshi && \
	git checkout $(TOSHI_VERSION) && \
	cd .. && \
	docker build -f Dockerfile.toshi -t $(TOSHI_BIN):$(TOSHI_VERSION) . && \
	docker tag $(TOSHI_BIN):$(TOSHI_VERSION) $(TOSHI_BIN):latest


build-dev:build-js

build-js:
	mkdir -p ./assets/js && \
	cd sitejs && npm run build && \
	cp ./dist/*.js ../assets/js && \
	cp ./dist/*.js.map ../assets/js

dev: build-dev build-toshi-docker
	cargo run --bin server & \
	docker run --rm -p 7000:7000 -v $$(pwd)/data:/data $(TOSHI_BIN):latest & \
	cd admin-ui && yarn start & \
	caddy

.PHONY: clean

clean:
	rm -rf assets/js/*
