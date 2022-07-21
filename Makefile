ifeq ($(shell uname),Darwin)
    EXT := dll
    PLATFORM := WIN
else
    EXT := so
    PLATFORM := UNIX
endif


# Development
release: build_parser copy_parser

build_parser: src/lib.rs Cargo.toml
	cargo build --release

copy_parser:
	(cd target && cp release/*.$(EXT) ../../src/main/resources/native)


debug: build_parser_debug copy_parser_debug

build_parser_debug: src/lib.rs Cargo.toml
	cargo build

copy_parser_debug:
	(cd target && cp debug/*.$(EXT) ../../src/main/resources/native)

clean_debug:
	rm -rf target/debug

# Production
all: x86_64-pc-windows_gnu x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-apple-darwin

aarch64-unknown-linux-gnu: src/lib.rs Cargo.toml
	cross build --release --target aarch64-unknown-linux-gnu

x86_64-unknown-linux-gnu: src/lib.rs Cargo.toml
	cross build --release --target x86_64-unknown-linux-gnu

x86_64-pc-windows_gnu: src/lib.rs Cargo.toml
	cross build --release --target x86_64-pc-windows-gnu

x86_64-apple-darwin: src/lib.rs Cargo.toml
	echo "x86_64-apple-darwin target not implemented yet"

copy: copy_x86_64-pc-windows_gnu copy_x86_64-unknown-linux-gnu copy_aarch64-unknown-linux-gnu copy_x86_64-apple-darwin

copy_x86_64-pc-windows_gnu:
	(cd target && cp x86_64-pc-windows-gnu/release/*.dll ../../src/main/resources/native/x86_64-pc-windows-gnu-rust-parser.dll)

copy_x86_64-unknown-linux-gnu:
	(cd target && cp x86_64-unknown-linux-gnu/release/*.so ../../src/main/resources/native/x86_64-unknown-linux-gnu-rust-parser.so)

copy_aarch64-unknown-linux-gnu:
	(cd target && cp aarch64-unknown-linux-gnu/release/*.so ../../src/main/resources/native/aarch64-unknown-linux-gnu-rust-parser.so)

copy_x86_64-apple-darwin:
	echo "x86_64-apple-darwin target not implemented yet"

clean:
	rm -rf target
