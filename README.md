# `v8-rs` [![Build Status](https://travis-ci.org/dflemstr/v8-rs.svg?branch=master)](https://travis-ci.org/dflemstr/v8-rs) [![Crates.io](https://img.shields.io/crates/v/v8.svg?maxAge=3600)](https://crates.io/crates/v8) [![codecov](https://codecov.io/gh/dflemstr/v8-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/dflemstr/v8-rs) [![Language (Rust)](https://img.shields.io/badge/powered_by-Rust-blue.svg)](http://www.rust-lang.org/)

**Note:** This library is still in pre-release (`0.x.y`) state.
It is usable but might still segfault and leak memory, and the API
might change between every version.  Use at your own risk.

This is a wrapper around the [V8](https://developers.google.com/v8/)
Javascript engine, used for example in
the [Google Chrome browser](https://www.google.com/chrome/browser)
or [Node.js](https://nodejs.org/en/).

[Documentation](https://dflemstr.github.io/v8-rs)

## Building

It is quite complicated to build V8.  This library has been tested
against V8 5.4.x with GCC 6.x, but later versions might work.

### Static / Shared

By default, this library links V8 statically.  There is a feature
called `shared` that builds it by linking to `libv8.so` (and related
libraries like `libicu-i10n.so`) instead.  There's usually little
reason to link dynamically since the V8 ABI changes fairly frequently.

### Ubuntu / Travis CI

The easiest way to build this library on Ubuntu or Travis CI is to use
a pre-packaged version of V8.  You need both `sudo` and Ubuntu Trusty
or later to install a compatible one:

``` yaml
sudo: true
dist: trusty
language: rust

addons:
  apt:
    sources:
      - sourceline: 'ppa:pinepain/libv8-5.4'
      - ubuntu-toolchain-r-test
    packages:
      # Modern compilers
      - gcc-6
      - g++-6
      # The V8 version that we want to bind
      - libv8-5.4-dev
      - libicu-dev

env:
  global:
    - CC=gcc-6
    - CXX=g++-6
```

### Build tree

You can build a build tree using any supported build method that uses
any combination of `depot_tools`, `make`, `gyp`, `ninja` and/or `gn`,
but `gn` hasn't been tested that extensively.

You should set `v8_use_snapshot=false`, loading snapshots is currently
not supported.

You should also not disable `i10n` support; this library assumes
`libicu` was built at the same time as V8 or is compatible with V8.

You should build using `shared_library` if you want to build with the
`shared` feature.

Simply set the environment variable `V8_SOURCE` to the root of the
`v8` checkout, and `V8_BUILD` to the build output in the tree (for
example `$V8_SOURCE/out/Release`) and the build Should Work®.  If not,
please figure out how to fix it and send a PR, it'll be impossible for
me to test all of the V8 build configurations :)
