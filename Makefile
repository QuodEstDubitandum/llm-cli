#!/usr/bin/make -f
export PATH := $(HOME)/.cargo/bin:$(PATH)

# Config
LOCAL_CONFIG_FILE = llm_cli_config.json
CONFIG_FILE = /etc/llm_cli_config.json
BIN_DIR = /usr/local/bin

APP_NAME = llm-cli
BUILD_DIR = target/release

# Targets
.PHONY: build install symlink uninstall

install: build symlink
	cp $(BUILD_DIR)/$(APP_NAME) $(BIN_DIR)/$(APP_NAME)
	cp $(LOCAL_CONFIG_FILE) $(CONFIG_FILE)

build:
	cargo build --release

symlink:
	ln -sf $(BIN_DIR)/llm-cli $(BIN_DIR)/gpt
	ln -sf $(BIN_DIR)/llm-cli $(BIN_DIR)/claude
	ln -sf $(BIN_DIR)/llm-cli $(BIN_DIR)/mistral
	

uninstall:
	rm -f $(BIN_DIR)/$(APP_NAME)
	rm -f $(CONFIG_FILE)
	rm -f $(BIN_DIR)/gpt
	rm -f $(BIN_DIR)/claude
	rm -f $(BIN_DIR)/mistral