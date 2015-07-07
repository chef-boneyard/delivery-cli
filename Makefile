# Copyright 2015 Chef Software, Inc.
#
# Author: Jon Anderson (janderson@chef.io)

CARGO_OPTS ?=
DELIV_CLI_GIT_SHA = $(shell git rev-parse --short HEAD)
RUSTC_VERSION = $(shell rustc --version)
CARGO_ENV = DELIV_CLI_GIT_SHA="$(DELIV_CLI_GIT_SHA)"
CARGO_ENV += RUSTC_VERSION="$(RUSTC_VERSION)"

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
