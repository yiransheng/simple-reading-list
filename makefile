SHELL := /bin/bash
OUT := _out
SERVER_BIN := reads.yiransheng.com/server
TOSHI_BIN := reads.yiransheng.com/toshi_bin
CADDY_BIN := reads.yiransheng.com/caddy

RELEASE := $(shell git rev-parse --verify HEAD)
JS_SRC := $(shell find sitejs/src -name '*.ts')
ADMIN_SRC := $(shell find admin-ui/src -name '*')

docker: $(OUT)/build-toshi-docker $(OUT)/build-server-docker $(OUT)/build-caddy-docker

$(OUT):
	mkdir -p $(OUT)

$(OUT)/build-server-docker:
	( [[ -n $$(docker images -q $(SERVER_BIN):$(RELEASE)) ]] || \
	  docker build -f docker/Dockerfile.server -t $(SERVER_BIN):$(RELEASE) . ) && \
	docker tag $(SERVER_BIN):$(RELEASE) $(SERVER_BIN):latest && \
	echo $$(docker images -q $(SERVER_BIN):$(RELEASE)) >> $(OUT)/build-server

$(OUT)/build-toshi-docker: TOSHI_VERSION=$(shell cat conf/__toshi_version)
$(OUT)/build-toshi-docker: $(OUT) conf/__toshi_version
	pushd ./Toshi && \
	git checkout $(TOSHI_VERSION) && \
	popd && \
	( [[ -n $$(docker images -q $(TOSHI_BIN):$(TOSHI_VERSION)) ] || \
	  docker build -f docker/Dockerfile.toshi -t $(TOSHI_BIN):$(TOSHI_VERSION) . ) && \
	docker tag $(TOSHI_BIN):$(TOSHI_VERSION) $(TOSHI_BIN):latest && \
	echo $$(docker images -q $(TOSHI_BIN):$(TOSHI_VERSION)) > $(OUT)/build-toshi-docker

$(OUT)/build-admin: $(OUT) $(ADMIN_SRC)
	pushd admin-ui && yarn build && \
	popd && \
	echo "done" > $(OUT)/build-admin

$(OUT)/build-js: $(OUT) $(JS_SRC)
	mkdir -p ./assets/js && \
	pushd sitejs && npm run build && \
	cp ./dist/*.js ../assets/js && \
	cp ./dist/*.js.map ../assets/js && \
	popd && \
	echo "done" > $(OUT)/build-js

$(OUT)/build-caddy: $(OUT)/build-admin $(OUT)/build-js
	( [[ -n $$(docker images -q $(CADDY_BIN):$(RELEASE)) ]] || \
	  docker build -f docker/Dockerfile.caddy -t $(CADDY_BIN):$(RELEASE) . ) && \
	docker tag $(CADDY_BIN):$(RELEASE) $(CADDY_BIN):latest && \
	echo $$(docker images -q $(CADDY_BIN):$(RELEASE)) >> $(OUT)/build-caddy

build-dev: $(OUT)/build-js

dev: build-dev $(OUT)/build-toshi-docker
	cargo run --bin server & \
	docker run --rm -p 7000:7000 -v $$(pwd)/data:/data --name=toshi \
	  $$(cat $(OUT)/build-toshi-docker) & \
	cd admin-ui && yarn start & \
	caddy

.PHONY: clean build-dev dev

clean:
	rm -rf assets/js/*
	docker rmi $$(cat $(OUT)/build-caddy) --force || true
	docker rmi $$(cat $(OUT)/build-server) --force || true
	rm -rf $(OUT)/*
