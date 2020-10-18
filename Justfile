all: build test
all-release: build-release test-release

# compiles the specsheet binary
@build:
    cargo build

# compiles the specsheet binary (in release mode)
@build-release:
    cargo build --release --verbose

# runs unit tests
@test:
    cargo test --all -- --quiet

# runs unit tests (in release mode)
@test-release:
    cargo test --release --all --verbose

# prints versions of the necessary build tools
@versions:
    rustc --version
    cargo --version

# lints the code
@clippy:
    touch spec_analysis/src/lib.rs
    touch spec_exec/src/lib.rs
    cargo clippy

# generates a code coverage report using tarpaulin via docker
@coverage-docker:
    docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin cargo tarpaulin --all --out Html

# updates versions, and checks for outdated ones
@update:
    cargo update
    cargo outdated

# builds the man pages
@man:
    mkdir -p "${CARGO_TARGET_DIR:-target}/man"
    pandoc --standalone -f markdown -t man man/specsheet.1.md          > "${CARGO_TARGET_DIR:-target}/man/specsheet.1"
    pandoc --standalone -f markdown -t man man/specsheet_apt.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_apt.5"
    pandoc --standalone -f markdown -t man man/specsheet_cmd.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_cmd.5"
    pandoc --standalone -f markdown -t man man/specsheet_defaults.5.md > "${CARGO_TARGET_DIR:-target}/man/specsheet_defaults.5"
    pandoc --standalone -f markdown -t man man/specsheet_dns.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_dns.5"
    pandoc --standalone -f markdown -t man man/specsheet_fs.5.md       > "${CARGO_TARGET_DIR:-target}/man/specsheet_fs.5"
    pandoc --standalone -f markdown -t man man/specsheet_gem.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_gem.5"
    pandoc --standalone -f markdown -t man man/specsheet_group.5.md    > "${CARGO_TARGET_DIR:-target}/man/specsheet_group.5"
    pandoc --standalone -f markdown -t man man/specsheet_hash.5.md     > "${CARGO_TARGET_DIR:-target}/man/specsheet_hash.5"
    pandoc --standalone -f markdown -t man man/specsheet_homebrew.5.md > "${CARGO_TARGET_DIR:-target}/man/specsheet_homebrew.5"
    pandoc --standalone -f markdown -t man man/specsheet_http.5.md     > "${CARGO_TARGET_DIR:-target}/man/specsheet_http.5"
    pandoc --standalone -f markdown -t man man/specsheet_npm.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_npm.5"
    pandoc --standalone -f markdown -t man man/specsheet_ping.5.md     > "${CARGO_TARGET_DIR:-target}/man/specsheet_ping.5"
    pandoc --standalone -f markdown -t man man/specsheet_systemd.5.md  > "${CARGO_TARGET_DIR:-target}/man/specsheet_systemd.5"
    pandoc --standalone -f markdown -t man man/specsheet_tap.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_tap.5"
    pandoc --standalone -f markdown -t man man/specsheet_tcp.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_tcp.5"
    pandoc --standalone -f markdown -t man man/specsheet_udp.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_udp.5"
    pandoc --standalone -f markdown -t man man/specsheet_ufw.5.md      > "${CARGO_TARGET_DIR:-target}/man/specsheet_ufw.5"
    pandoc --standalone -f markdown -t man man/specsheet_user.5.md     > "${CARGO_TARGET_DIR:-target}/man/specsheet_user.5"
