set default-list := true
doc_features := "debug,user_properties,physics,avian,rapier"

# Control simultaneous linking operations when building
jobs_arg := if env("BUILD_NJOBS", "0") != "0" {
    "--jobs $BUILD_NJOBS"
} else {
    ""
}

# Comment out this if not using rust nightly
export RUSTFLAGS := "-Dwarnings -Zshare-generics=y -Zthreads=0 -C debuginfo=line-tables-only"
export RUSTDOCFLAGS := "-Dwarnings -Zshare-generics=y -Zthreads=0 -C debuginfo=line-tables-only"

#### High-level checks jobs

check: clippy fmt build

check-ci: check doc-test build-all test doc-build book-build

check-all: check-ci check-all-features

check-release: check-all
    cargo publish --dry-run

#### Low-level checks jobs

clippy:
    cargo clippy --workspace --all-targets --all-features -- --deny warnings

fmt:
    cargo fmt --all -- --check

test: build-tests
    cargo test {{ jobs_arg }} --workspace --all-features

#### Build jobs

build: build-lib

build-all: build-lib build-tests build-examples

build-lib:
    cargo build --workspace --all-features --lib

build-examples: build-lib
    cargo build {{ jobs_arg }} --workspace --all-features --examples

build-tests: build-lib
    cargo build {{ jobs_arg }} --workspace --all-features --tests
check-all-features:
    cargo check-all-features

#### Documentation jobs

doc-test:
    cargo test --workspace --doc --all-features

doc-build:
    cargo doc --features={{ doc_features }}

book-build:
    mdbook --version &>/dev/null && (cd ./book && mdbook build)

book-serve:
    mdbook --version &>/dev/null && (cd ./book && mdbook serve)
