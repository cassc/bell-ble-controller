# sudo apt-get install -y libdbus-1-dev
arm:
	cross build --target armv7-unknown-linux-gnueabihf --release
deploy: arm
		rsync -av target/armv7-unknown-linux-gnueabihf/release/bell-ble-controller firefly-prod:
sync:
	rsync -av src firefly:bell-controller/
	rsync -av Cargo.toml firefly:bell-controller/
dev-watch:
	cargo watch -w src/ -x build
