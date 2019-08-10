SHELL := /bin/bash
OUT := _out
SERVER_BIN := reads.yiransheng.com/server
TOSHI_BIN := reads.yiransheng.com/toshi_bin
CADDY_BIN := reads.yiransheng.com/caddy

SERVER_SRC := $(shell find src -name '*')
RELEASE := $(shell git rev-parse --verify HEAD)
JS_SRC := $(shell find sitejs/src -name '*.ts')
ADMIN_SRC := $(shell find admin-ui/src -name '*')

docker: $(OUT)/build-toshi-docker $(OUT)/build-server-docker $(OUT)/build-caddy-docker

dev: $(OUT)/build-js dev-toshi
	cargo run --bin create-admin-user -- -u admin -p password & \
	cargo run --bin create-toshi-index -- conf/toshi_index.json & \
	cargo run --bin server & \
	cd admin-ui && yarn start & \
	caddy

dev-toshi: $(OUT)/build-toshi-docker
	docker run --rm -p 7000:7000 -v $$(pwd)/data:/data --name=toshi \
	  $$(cat $(OUT)/build-toshi-docker) &

$(OUT):
	mkdir -p $(OUT)

dummy_src: $(SERVER_SRC)
	mkdir -p dummy_src/bin
	echo 'fn main() {}' | tee $$(find src/bin -name '*.rs' | sed 's/src/dummy_src/') 
	echo 'syntax error' > dummy_src/lib.rs

$(OUT)/build-server-docker: dummy_src $(SERVER_SRC)
	( [[ -n $$(docker images -q $(SERVER_BIN):$(RELEASE)) ]] || \
	  docker build -f docker/Dockerfile.server -t $(SERVER_BIN):$(RELEASE) . ) && \
	docker tag $(SERVER_BIN):$(RELEASE) $(SERVER_BIN):latest && \
	echo $$(docker images -q $(SERVER_BIN):$(RELEASE)) >> $(OUT)/build-server-docker

$(OUT)/build-toshi-docker: TOSHI_VERSION=$(shell cat conf/__toshi_version)
$(OUT)/build-toshi-docker: $(OUT) conf/__toshi_version
	pushd ./Toshi && \
	git checkout $(TOSHI_VERSION) && \
	popd && \
	( [[ -n $$(docker images -q $(TOSHI_BIN):$(TOSHI_VERSION)) ]] || \
	  docker build -f docker/Dockerfile.toshi -t $(TOSHI_BIN):$(TOSHI_VERSION) . ) && \
	docker tag $(TOSHI_BIN):$(TOSHI_VERSION) $(TOSHI_BIN):latest && \
	echo $$(docker images -q $(TOSHI_BIN):$(TOSHI_VERSION)) > $(OUT)/build-toshi-docker

$(OUT)/build-admin: $(OUT) $(ADMIN_SRC)
	pushd admin-ui && yarn build && \
	popd && \
	echo "done" > $(OUT)/build-admin

$(OUT)/build-js: $(OUT) $(JS_SRC)
	mkdir -p ./assets/js && \
	pushd sitejs && yarn build && \
	cp ./dist/*.js ../assets/js && \
	cp ./dist/*.js.map ../assets/js && \
	popd && \
	echo "done" > $(OUT)/build-js

$(OUT)/build-caddy-docker: $(OUT)/build-admin $(OUT)/build-js
	( [[ -n $$(docker images -q $(CADDY_BIN):$(RELEASE)) ]] || \
	  docker build -f docker/Dockerfile.caddy -t $(CADDY_BIN):$(RELEASE) . ) && \
	docker tag $(CADDY_BIN):$(RELEASE) $(CADDY_BIN):latest && \
	echo $$(docker images -q $(CADDY_BIN):$(RELEASE)) >> $(OUT)/build-caddy-docker


.PHONY: clean dev dev-toshi docker

clean:
	rm -rf assets/js/*
	docker rmi $$(cat $(OUT)/build-caddy-docker) --force || true
	docker rmi $$(cat $(OUT)/build-server-docker) --force || true
	rm -rf $(OUT)/*
