pkg_name=delivery-cli
pkg_origin=chef
pkg_version=0.0.1
pkg_maintainer="Salim Afiune <afiune@chef.io>"
pkg_license=('Apache-2.0')
pkg_source=nosuchfile.tar.gz
# (Salim) TODO: We want the result to be a portable, static binary
# in a zero-dependency package. At the moment I cound't make it to
# compile because we need to figure out how to tell zlib that we are
# building in static mode, so for now we are packaging the dependencies.
pkg_deps=(
  core/gcc
  core/zlib-musl
  core/openssl-musl
)
pkg_build_deps=(
  core/musl
  core/zlib-musl
  core/openssl-musl
  core/coreutils
  core/rust core/gcc
)
pkg_bin_dirs=(bin)

# Name of the resulting binary
_bin="delivery"

do_prepare() {
  # Can be either `--release` or `--debug` to determine cargo build strategy
  build_type="--release"
  build_line "Building artifacts with \`${build_type#--}' mode"

  # Used by Cargo to use a pristine, isolated directory for all compilation
  export CARGO_TARGET_DIR="$HAB_CACHE_SRC_PATH/$pkg_dirname"
  build_line "Setting CARGO_TARGET_DIR=$CARGO_TARGET_DIR"

  export rustc_target="x86_64-unknown-linux-musl"
  build_line "Setting rustc_target=$rustc_target"

  export OPENSSL_LIB_DIR=$(pkg_path_for openssl-musl)/lib
  export OPENSSL_INCLUDE_DIR=$(pkg_path_for openssl-musl)/include
  export OPENSSL_STATIC=true

  rustflags="-L$(pkg_path_for zlib-musl)/lib -lz"
  rustflags="$rustflags -L$(pkg_path_for openssl-musl)/lib -lssl -lcrypto"
  export RUSTFLAGS="$rustflags"

  export LD_LIBRARY_PATH=$(pkg_path_for gcc)/lib
  build_line "Setting LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
}

do_build() {
  pushd "$PLAN_CONTEXT" > /dev/null
  cargo build ${build_type#--debug} --target=$rustc_target --verbose
  popd > /dev/null
}

do_install() {
  install -v -D $CARGO_TARGET_DIR/$rustc_target/${build_type#--}/$_bin \
    $pkg_prefix/bin/$_bin
}

do_strip() {
  if [[ "$build_type" != "--debug" ]]; then
    do_default_strip
  fi
}

# Turn the remaining default phases into no-ops
do_download() {
  return 0
}

do_verify() {
  return 0
}

do_unpack() {
  return 0
}
