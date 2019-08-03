SHELL := /bin/bash
OUT := _out
SERVER_BIN := reads.yiransheng.com/server
TOSHI_BIN := reads.yiransheng.com/toshi_bin

build-server-docker:
	docker build -f docker/Dockerfile.server -t $(SERVER_BIN):latest .

$(OUT):
	mkdir -p $(OUT)

$(OUT)/build-toshi-docker: TOSHI_VERSION=$(shell cat conf/__toshi_version)
$(OUT)/build-toshi-docker: $(OUT)
	pushd ./Toshi && \
	git checkout $(TOSHI_VERSION) && \
	popd && \
	docker build -f docker/Dockerfile.toshi -t $(TOSHI_BIN):$(TOSHI_VERSION) . && \
	docker tag $(TOSHI_BIN):$(TOSHI_VERSION) $(TOSHI_BIN):latest && \
	echo "$(TOSHI_BIN):$(TOSHI_VERSION)" > $(OUT)/build-toshi-docker

$(OUT)/build-js: $(OUT)
	mkdir -p ./assets/js && \
	pushd sitejs && npm run build && \
	cp ./dist/*.js ../assets/js && \
	cp ./dist/*.js.map ../assets/js && \
	popd && \
	echo "done" > $(OUT)/build-js

build-dev: $(OUT)/build-js

dev: build-dev $(OUT)/build-toshi-docker
	cargo run --bin server & \
	docker run --rm -p 7000:7000 -v $$(pwd)/data:/data $(TOSHI_BIN):latest & \
	cd admin-ui && yarn start & \
	caddy

.PHONY: clean build-dev dev

clean:
	rm -rf assets/js/*
	rm -rf $(OUT)/*
