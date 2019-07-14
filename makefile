

local:
	cargo run --bin server & \
	toshi -c toshi_config.toml & \
	cd admin-ui && yarn start & \
	caddy

.PHONY: clean

clean:
