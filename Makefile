all:
	echo "No Target. Try make run"

build: src/main.rs src/style.toml
	cargo build

release: clean
	mkdir -p bin
	rm -rf bin/*
	cargo build --release
	cp target/release/ohme-ssm bin/ohme-ssm

run: build
	./target/debug/ohme-ssm

clean:
	cargo clean
	rm -rf bin/
