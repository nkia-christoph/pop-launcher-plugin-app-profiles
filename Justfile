default: install_user


alias b := build
alias i := install
alias l := lint_dev
alias t := user_test


PLUGIN_NAME := "app-profiles"
PATH := / "pop-launcher" / "plugins" / PLUGIN_NAME
USER_PATH := data_local_directory() / PATH
SYSTEM_PATH := / "etc" / PATH
DIST_PATH := / "usr" / "lib" / PATH
CONFIG_DIR := "config"
BITMASK := "-Dm0764"


check:
    cargo fmt -- --check

lint:
    cargo clippy --all-targets --all-features -- -D warnings

# lint with pedantic warnings
lint-dev: check
    cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic
    cargo clippy --fix

user_test: test
    cargo build

build_release: test
    cargo clean
    cargo build --release


test:
    cargo test

cp_config_user:
    install -Dm0764 {{ justfile_directory() }}/plugin.ron {{ USER_PATH }}/plugin.ron
    install -Dm0764 {{ justfile_directory() }}/{{ CONFIG_DIR }}/*.ron {{ USER_PATH }}/{{ CONFIG_DIR }}/*.ron

cp_config_system:
    install -Dm0764 {{ justfile_directory() }}/plugin.ron {{ SYSTEM_PATH }}/plugin.ron
    install -Dm0764 {{ justfile_directory() }}/{{ CONFIG_DIR }}/*.ron {{ SYSTEM_PATH }}/{{ CONFIG_DIR }}/*.ron

# Installation command
[private]
install-cmd options=BITMASK source destination:
    install {{ options }} {{ join( justfile_directory(), source ) }} {{ join( justfile_directory(), target ) }}
    echo "installing config in {{ join( join( justfile_directory(), CONFIG_PATH ), target ) }}"
    install {{ options }} {{ join( join( justfile_directory(), CONFIG_PATH ), source ) }} {{ join( join( justfile_directory(), CONFIG_PATH ), target ) }}

[private]
install-bin src dest: (install-cmd '-Dm0755' src dest)

[private]
install-file src dest: (install-cmd '-Dm0644' src dest)


test_user: build_test
    @just install 764 plugin.ron plugin.ron
    @just install 764 "{{ justfile_directory() }}/plugin.ron" "{{ USER_PATH }}/plugin.ron"
    install -Dm0754 $(realpath target/debug)/app-profiles ${USER_PATH}/app-profiles

test_system: build_test cp_config_system
    install -Dm0774 $(realpath target/debug)/app-profiles ${SYSTEM_PATH}/app-profiles


install_user: build_release cp_config_user
    install -Dm0754 $(realpath target/release)/app-profiles ${USER_PATH}/app-profiles

install_system: build_release cp_config_system
    install -Dm0774 $(realpath target/release)/app-profiles ${SYSTEM_PATH}/app-profiles


uninstall_user:
    rm -r {{ USER_PATH }}

uninstall_system:
    rm -r {{ SYSTEM_PATH }}

uninstall_dist:
    rm -r {{ DIST_PATH }}
