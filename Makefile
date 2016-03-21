# Copyright 2015 Chef Software, Inc.
#
# Author: Jon Anderson (janderson@chef.io)

CARGO_OPTS ?=
DELIV_CLI_GIT_SHA = $(shell git rev-parse --short HEAD)
DELIV_CLI_TIME = $(shell date -u "+%Y-%m-%dT%H:%M:%SZ")
RUSTC_VERSION = $(shell rustc --version)
CARGO_ENV = DELIV_CLI_GIT_SHA="$(DELIV_CLI_GIT_SHA)"
CARGO_ENV += RUSTC_VERSION="$(RUSTC_VERSION)"
CARGO_ENV += DELIV_CLI_TIME="$(DELIV_CLI_TIME)"

UNAME = $(shell uname)

ifeq ($(UNAME),Darwin)
    OPENSSL_PREFIX = /usr/local/opt/openssl
    CARGO_ENV += OPENSSL_INCLUDE_DIR=$(OPENSSL_PREFIX)/include
    CARGO_ENV += OPENSSL_LIB_DIR=$(OPENSSL_PREFIX)/lib
    CARGO_ENV += OPENSSL_STATIC=1
else ifeq ($(UNAME),Linux)
    OPENSSL_PREFIX = /opt/delivery-cli-build-deps/openssl
    CARGO_ENV += OPENSSL_INCLUDE_DIR=$(OPENSSL_PREFIX)/include
    CARGO_ENV += OPENSSL_LIB_DIR=$(OPENSSL_PREFIX)
    CARGO_ENV += OPENSSL_STATIC=1
else ifeq ($(UNAME),MINGW32_NT-6.2)
    OPENSSL_PREFIX ?= C:/chef/delivery-cli/embedded
    CARGO_ENV += OPENSSL_INCLUDE_DIR=$(OPENSSL_PREFIX)/include
    CARGO_ENV += OPENSSL_LIB_DIR=$(OPENSSL_PREFIX)/bin
endif

CARGO = $(CARGO_ENV) cargo

all:
	$(MAKE) build

build: openssl
	$(CARGO) $(CARGO_OPTS) build --release

openssl:
	@test -d $(OPENSSL_PREFIX) || \
         (echo "MISSING DEP: $(OPENSSL_PREFIX)" && exit 101)

clean:
	@$(CARGO) $(CARGO_OPTS) clean

check:
	$(MAKE) build
	$(MAKE) test

test:
	$(CARGO) $(CARGO_OPTS) test


.PHONY: all build clean check test

bin/cucumber: Gemfile
	bundle install --binstubs=bin --path=vendor/bundle

# Our fake api server generates its own self-signed certificate, and
# outputs some noice on standard error; redirecting it to /dev/null
# cleans up the output a little bit.
#
# Depends on the target/release/delivery executable having been built
cucumber: build bin/cucumber
	bin/cucumber 2>/dev/null && rm -rf features/tmp

# Run the build cookbook's default recipe on the current machine. Use
# this to set up your workstation to build the CLI (e.g., to install
# the proper version of Rust)
#
# (If you had the CLI already, you could run `delivery job verify
# default`, but you're trying to build the CLI; we can't have
# Chefception *all* the time.)
vendor_cookbook_deps:
	berks vendor --berksfile=cookbooks/delivery_rust/Berksfile vendor/cookbooks

setup: vendor_cookbook_deps
	chef-client --local-mode --override-runlist delivery_rust --config cli_setup_client.rb

dev: vendor_cookbook_deps
	chef-client --local-mode --override-runlist delivery_rust::dev --config cli_setup_client.rb
