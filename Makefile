# Copyright 2015 Chef Software, Inc.
#
# Author: Jon Anderson (janderson@chef.io)
#
# Bare-bones Makefile for deliver-cli on OSX. Checks install rustc and
# if it is not the correct version (as of 2015-03-31), complains.
#
# 'make rustup' will installed the pinned version with the nightly channel

CARGO = cargo
PINNED_RUST_VERSION = c4b23aec4 2015-04-29
RUST_VERSION := $(shell rustc --version | tr -d '()' | awk '{ print $$3 " " $$4 }')
RUST_UP_COMMAND = sudo ./rustup.sh --date=2015-04-29 --channel=nightly
CARGO_OPTS =

# If the installed version matches the pinning above, the codebase should be compatible.
ifeq "$(RUST_VERSION)" "$(PINNED_RUST_VERSION)"
	RUST_COMPAT=true
else
	RUST_COMPAT=false
endif

all:
	$(MAKE) build

build:
ifeq ($(RUST_COMPAT),true)
	$(CARGO) $(CARGO_OPTS) build --release
else
	@echo "Rust version ($(RUST_VERSION)) not at pinned version ($(PINNED_RUST_VERSION))"
	@echo "'make rustup' will install the pinned version, or run 'cargo build' with another rustc."
endif

clean:
	$(CARGO) $(CARGO_OPTS) clean

check:
	$(MAKE) build
	$(MAKE) test

test:
	$(CARGO) $(CARGO_OPTS) test

rustup:
	$(RUST_UP_COMMAND)

.PHONY: all build clean check test rustcheck
