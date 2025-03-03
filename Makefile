all:
	echo "No Target. Try make run"

build: src/main.rs src/style.toml
	cargo build

release: clean
	mkdir -p bin
	rm -rf bin/*
	cargo build --release
	cp target/release/rssm bin/rssm

run: build
	./target/debug/rssm

clean:
	cargo clean
	rm -rf bin/
