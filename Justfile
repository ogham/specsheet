all: build test
all-release: build-release test-release

check_types := "apt cmd defaults dns fs gem group hash homebrew http npm ping systemd tap tcp udp ufw user"

# compiles the specsheet binary
@build:
    cargo build

# compiles the specsheet binary (in release mode)
@build-release:
    cargo build --release --verbose
    strip "${CARGO_TARGET_DIR:-target}/release/specsheet"


# runs unit tests
@test:
    cargo test --workspace -- --quiet

# runs unit tests (in release mode)
@test-release:
    cargo test --release --workspace --verbose


# lints the code
@clippy:
    touch spec_analysis/src/lib.rs
    touch spec_exec/src/lib.rs
    cargo clippy

# generates a code coverage report using tarpaulin via docker
@coverage-docker:
    docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin cargo tarpaulin --workspace --out Html

# updates dependency versions, and checks for outdated ones
@update-deps:
    cargo update
    command -v cargo-outdated >/dev/null || (echo "cargo-outdated not installed" && exit 1)
    cargo outdated

# lists unused dependencies
@unused-deps:
    command -v cargo-udeps >/dev/null || (echo "cargo-udeps not installed" && exit 1)
    cargo +nightly udeps

# prints versions of the necessary build tools
@versions:
    rustc --version
    cargo --version


# builds the man pages
man:
    #!/bin/sh
    mkdir -p "${CARGO_TARGET_DIR:-target}/man"
    pandoc --standalone -f markdown -t man man/specsheet.1.md          > "${CARGO_TARGET_DIR:-target}/man/specsheet.1"
    pandoc --standalone -f markdown -t man man/specsheet.5.md          > "${CARGO_TARGET_DIR:-target}/man/specsheet.5"
    for ct in {{ check_types }}; do
        pandoc --standalone -f markdown -t man man/specsheet_$ct.5.md  > "${CARGO_TARGET_DIR:-target}/man/specsheet_$ct.5"
    done


# creates a distributable package
zip desc exe="specsheet":
    #!/usr/bin/env perl
    use Archive::Zip;
    -e 'target/release/{{ exe }}' || die 'Release binary not built!';
    -e 'target/man/specsheet.1' || die 'Man pages not built!';
    my $zip = Archive::Zip->new();
    $zip->addFile('completions/specsheet.bash');
    $zip->addFile('completions/specsheet.zsh');
    $zip->addFile('completions/specsheet.fish');
    $zip->addFile('target/man/specsheet.1', 'man/specsheet.1');
    $zip->addFile('target/man/specsheet.5', 'man/specsheet.5');
    for (qw[{{ check_types }}]) {
        $zip->addFile("target/man/specsheet_$_.5", "man/specsheet_$_.5");
    }
    $zip->addFile('target/release/{{ exe }}', 'bin/{{ exe }}');
    $zip->writeToFileNamed('specsheet-{{ desc }}.zip') == AZ_OK || die 'Zip write error!';
    system 'unzip -l "specsheet-{{ desc }}".zip'
