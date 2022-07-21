# Rust Tree-sitter parser implementation

### Description

[Tree-sitter](https://tree-sitter.github.io/tree-sitter/) parser implementation for code-navigation bitbucket plugin

### Requirements

#### Compile
Make sure that you have:
* [RustUp](https://rustup.rs/)
* [Docker](https://docs.docker.com/engine/install/ubuntu/) (for cross-platform compilation)
* [Cross](https://github.com/rust-embedded/cross) (for cross-platform compilation)

#### Installation

RustUp: 
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Cross:
```
$ cargo install cross
```

### Usage
To compile rust code for local platform ``` cargo build ``` is used

```
$ cd /rust-project
$ cargo build
$ cp debug/librust_parser.so ../../src/main/resources/native
```
or
```
$ cd /rust-project
$ make -f Makefile debug
```

Cross compilation is achieved by ```Cross ``` and ``` Makefile ```

To compile source code for all platforms:
```
$ make -f Makefile all
```
To compile for specified platform:
```
& make -f Makefile x86_64-pc-windows_gnu
```

To copy binaries into java resource directory for all platforms:

```
$ make -f Makefile copy
```

To copy binaries into java resource dir for all specified platform:

```
$ make -f Makefile copy_x86_64-pc-windows_gnu
```
