pkg_name=limitless
pkg_origin=edavis
pkg_maintainer="Elliott Davis <elliott@excellent.io>"
pkg_license=('Apache-2.0')
pkg_bin_dirs=(bin)
pkg_deps=(core/glibc core/openssl core/gcc-libs)
pkg_build_deps=(core/coreutils core/rust-nightly core/gcc core/git core/make core/pkg-config)
bin="limitless"

pkg_version() {
  echo "$(($(git rev-list master --count)))"
}

do_before() {
  update_pkg_version
}

do_prepare() {
  export build_type="${build_type:---release}"
  # Can be either `--release` or `--debug` to determine cargo build strategy
  build_line "Building artifacts with \`${build_type#--}' mode"

  export rustc_target="x86_64-unknown-linux-gnu"
  build_line "Setting rustc_target=$rustc_target"

  # Used by the `build.rs` program to set the version of the binaries
  export PLAN_VERSION="${pkg_version}/${pkg_release}"
  build_line "Setting PLAN_VERSION=$PLAN_VERSION"

  # Used by Cargo to use a pristine, isolated directory for all compilation
  export CARGO_TARGET_DIR="$HAB_CACHE_SRC_PATH/$pkg_dirname"
  build_line "Setting CARGO_TARGET_DIR=$CARGO_TARGET_DIR"
}

do_build() {
  pushd "$PLAN_CONTEXT"/.. > /dev/null
  cargo build "${build_type#--debug}" --target="$rustc_target" --verbose
  popd > /dev/null
}

do_install() {
  install -v -D "$CARGO_TARGET_DIR/$rustc_target/${build_type#--}/$bin" \
    "$pkg_prefix/bin/$bin"
}

do_strip() {
  return 0
}