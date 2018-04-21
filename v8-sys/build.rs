extern crate bindgen;
extern crate cc;
extern crate env_logger;
extern crate pkg_config;

use std::env;
use std::path;

fn main() {
    env_logger::init().unwrap();

    pkg_config::Config::new()
        .atleast_version("6.0.0.0")
        .probe("v8")
        .expect("unable to locate V8 via pkg-config");

    cc::Build::new()
        .cpp(true)
        .warnings(true)
        .flag("--std=c++11")
        // So that we can use all functions via FFI
        .flag("-fkeep-inline-functions")
        .file("src/allocator.cpp")
        .file("src/isolate.cpp")
        .file("src/platform.cpp")
        .compile("librust-v8-impls.a");

    let bindings = bindgen::Builder::default()
        .header("src/wrapper.hpp")
        .rust_target(bindgen::RustTarget::Nightly)
        .clang_arg("--std=c++11")
        .whitelist_type("v8::.*")
        .whitelist_type("rust_v8_impls::.*")
        .whitelist_function("v8::.*")
        .whitelist_function("rust_v8_impls::.*")
        .whitelist_var("v8::.*")
        .whitelist_var("rust_v8_impls::.*")
        // Because there are some layout problems with these
        .opaque_type("std::.*")
        // For some reason bindgen output is corrupt (syntax errors) for these types
        .blacklist_type("v8::JitCodeEvent__bindgen.*")
        .blacklist_type(".*DisallowJavascriptExecutionScope.*")
        .blacklist_type(".*SuppressMicrotaskExecutionScope.*")
        // We want to re-structure the modules a bit and hide the "root" module
        .raw_line("#[doc(hidden)]")
        .generate_inline_functions(true)
        .enable_cxx_namespaces()
        .derive_debug(true)
        .derive_hash(true)
        .derive_eq(true)
        .derive_partialeq(true)
        .generate()
        .expect("unable to generate v8 bindings");

    let out_path = path::PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR env var not set"));

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("unable to write bindings file");
}
