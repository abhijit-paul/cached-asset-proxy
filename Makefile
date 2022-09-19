target_dir = target
build_mode = debug

debug:
	echo $(target_dir)
	cargo build --target-dir $(target_dir)

release:
	echo $(target_dir)
	cargo build --release --target-dir $(target_dir)

clippy:
	echo $(target_dir)
	cargo clippy --target-dir $(target_dir)

copy-bins:
	mkdir -p ./bin

	cp -v $(target_dir)/$(build_mode)/cached-asset-proxy ./bin/

clean:
	cargo clean --target-dir $(target_dir)

test:
	HOST=0.0.0.0 \
	PORT=8000 \
	ASSETS_URL="http://localhost:5000" \
	ALLOW_ORIGINS="https://example.com,https://example1.com" \
	cargo test --target-dir $(target_dir)

drone-test:

	cargo test --release --target-dir $(target_dir) -- test-threads 1

drone-build:	release copy-bins

local-build:	debug copy-bins
