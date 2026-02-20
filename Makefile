USER_PATH = $(HOME)/.local/share/pop-launcher/plugins/app-profiles
SYSTEM_PATH = /etc/pop-launcher/plugins/app-profiles/app-profiles
CONFIG_DIR = config

PHONY := test_user test_system install_user install_system uninstall_user uninstall_system


test:
	cargo check
	cargo test


build_test: test
	cargo build

build_release: test
	cargo clean
	cargo build --release


cp_config_user:
	install -Dm0764 $(realpath .)/plugin.ron ${USER_PATH}/plugin.ron
	install -Dm0764 $(realpath ${CONFIG_DIR})/*.ron ${USER_PATH}/${CONFIG_DIR}/*.ron

cp_config_system:
	install -Dm0774 $(realpath .)/plugin.ron ${SYSTEM_PATH}/plugin.ron
	install -Dm0774 $(realpath ${CONFIG_DIR})/*.ron ${SYSTEM_PATH}/${CONFIG_DIR}/*.ron


test_user: build_test cp_config_user
	install -Dm0754 $(realpath target/debug)/app-profiles ${USER_PATH}/app-profiles

test_system: build_test cp_config_system
	install -Dm0774 $(realpath target/debug)/app-profiles ${SYSTEM_PATH}/app-profiles


install_user: build_release cp_config_user
	install -Dm0754 $(realpath target/release)/app-profiles ${USER_PATH}/app-profiles

install_system: build_release cp_config_system
	install -Dm0774 $(realpath target/release)/app-profiles ${SYSTEM_PATH}/app-profiles

console
uninstall_user:
	rm -r ${USER_PATH}

uninstall_system:
	rm -r ${SYSTEM_PATH}
