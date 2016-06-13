# Copyright 2015 Chef Software, Inc.
#
# Author: Jon Anderson (janderson@chef.io)

RUST_VERSION ?= 1.9.0

CARGO_OPTS ?=
DELIV_CLI_VERSION = $(shell git describe --abbrev=0 --tags)
DELIV_CLI_GIT_SHA = $(shell git rev-parse --short HEAD)
DELIV_CLI_TIME = $(shell date -u "+%Y-%m-%dT%H:%M:%SZ")
RUSTC_VERSION = $(shell rustc --version)
CARGO_ENV += DELIV_CLI_VERSION="$(DELIV_CLI_VERSION)"
CARGO_ENV += DELIV_CLI_GIT_SHA="$(DELIV_CLI_GIT_SHA)"
CARGO_ENV += RUSTC_VERSION="$(RUSTC_VERSION)"
CARGO_ENV += DELIV_CLI_TIME="$(DELIV_CLI_TIME)"

UNAME = $(shell uname)

ifeq ($(USE_CHEFDK_LIBS), true)
	OPENSSL_PREFIX ?= /opt/chefdk/embedded
        CARGO_ENV += OPENSSL_INCLUDE_DIR=$(OPENSSL_PREFIX)/include
        CARGO_ENV += OPENSSL_LIB_DIR=$(OPENSSL_PREFIX)
else ifeq ($(UNAME),Darwin)
	OPENSSL_PREFIX ?= /usr/local/opt/openssl
	CARGO_ENV += OPENSSL_INCLUDE_DIR=$(OPENSSL_PREFIX)/include
	CARGO_ENV += OPENSSL_LIB_DIR=$(OPENSSL_PREFIX)
else ifeq ($(UNAME),Linux)
	OPENSSL_PREFIX ?= /usr/lib/x86_64-linux-gnu
        CARGO_ENV += OPENSSL_INCLUDE_DIR=$(OPENSSL_PREFIX)/include
        CARGO_ENV += OPENSSL_LIB_DIR=$(OPENSSL_PREFIX)
endif

CARGO = $(CARGO_ENV) cargo

all:
	$(MAKE) build

# --release takes longer to compile but is slightly more optimized.
# For dev iterations (which is the only thing this Makefile is used for)
# we should leave off the --release flag.
build: check_deps
	$(CARGO) $(CARGO_OPTS) build

release: check_deps
	$(CARGO) $(CARGO_OPTS) build --release

# Updates all cargo deps.
# Should be run periodically to pull in new deps.
update_deps: clean
	$(CARGO) $(CARGO_OPTS) update

clean:
	@$(CARGO) $(CARGO_OPTS) clean

check_deps: openssl_check rust_check cargo_check ruby_check

check:
	$(MAKE) build
	$(MAKE) test

test:
	$(CARGO) $(CARGO_OPTS) test


.PHONY: all build update_deps release clean check_deps check test

bin/cucumber: Gemfile
	bundle install --binstubs=bin --path=vendor/bundle

# Our fake api server generates its own self-signed certificate, and
# outputs some noice on standard error; redirecting it to /dev/null
# cleans up the output a little bit.
#
# Depends on the target/release/delivery executable having been built
cucumber: release bin/cucumber
	bin/cucumber 2>/dev/null && rm -rf features/tmp

openssl_check:
	@ls $(OPENSSL_PREFIX) >> /dev/null || \
	(echo "\nWe could not find openssl on your local development machine.\n"\
	"If you are developing on OS X try:\n\n"\
	"brew install openssl\n\n"\
	"And run this command again.\n\n"\
	"If you are still hitting this error after that, it is likely you have installed openssl somewhere custom or are not developing on OS X.\n"\
	"This script assumes /usr/local/opt/openssl is the path to folder containing your openssl libaries and headers.\n"\
	"If you have put them somewhere custom, please set OPENSSL_PREFIX to the openssl folder that contains (lib, include, etc.) and run make again.\n"\
	&& exit 1)

# Check if rust is installed at all and instruct user if not.
# Check if the proper version of rust is installed,
# and if not, prompt the user to update via homebrew.
# If the project if it is out of date with latest homebrew,
# instruct user to follow readme docs on how to update rust.
rust_check:
	@which rustc >> /dev/null || \
	(echo "Rust is not installed.\n"\
	"We recommend installing with brew by running:\n\n"\
	"brew install rust\n" && exit 1)
	@rustc --version | grep $(RUST_VERSION) >> /dev/null || \
	(echo "\nRust is not installed at the proper version ($(RUST_VERSION)) on your machine.\n"\
	"Please install at the right version (we recommend brew install rust).\n"\
	"If the default version from homebrew is out of date,\n"\
	"Please update the version of rust we ship with delivery-cli by following the instuctions in the readme under Updating Rust Version.\n")

cargo_check:
	@which cargo >> /dev/null || \
	(echo "Cargo is not installed but rust is.\n"\
	"If you used to develop for delivery-cli and used the automated rust installer, it installed a old version of rust without cargo.\n"\
	"You should uninstall rust via:\n\n"\
	"sudo /usr/local/lib/rustlib/uninstall.sh\n\n"\
	"Then installing via brew with:\n\n"\
	"brew install rust\n" && exit 1)

ruby_check:
	@which ruby >> /dev/null || \
	(echo "Ruby is not installed. Install via your preferred method, or use rbenv if you are unsure how to get started.")
