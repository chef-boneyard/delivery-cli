# Delivery CLI

The CLI for Chef Delivery. Written in Rust, super experimental, will probably hurt your kittens.

While the Rust Language is now moving towards 1.0, and things should begin to stabilize, follow-on releases sometimes introduce non-backwardly-compatable changes, which can break this build. Until Rust truly stabilizes, you'll need to install rust (the easiest way on mac):

```bash
$ curl -s https://static.rust-lang.org/rustup.sh | sudo sh
```bash

If this repo fails to build, using the instructions below, you might try:

```bash
$ cargo clean
$ cargo update
```bash

This may update the Cargo.lock file, which is currently checked in. If there are changes, they should likely be included in your CR.

If there are syntax or other errors, well, good luck!

## Build me

```bash
cargo build
```

## Test me

```bash
cargo test
```

## Develop me

Hack about, then:

```bash
cargo run -- review ...
```

Where "review" and friends are the arguments you would pass to the delivery cli.
