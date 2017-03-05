extern crate v8_api;
extern crate bindgen;
extern crate clang;
extern crate clang_sys;
extern crate gcc;
extern crate pkg_config;

use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path;

const LIBS: [&'static str; 4] = ["v8_base", "v8_libbase", "v8_libsampler", "v8_nosnapshot"];

trait DisplayAsC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

struct C<'a, A>(&'a A) where A: 'a;

impl<'a, A> fmt::Display for C<'a, A>
    where A: DisplayAsC + 'a
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        DisplayAsC::fmt(self.0, f)
    }
}

fn main() {
    let out_dir_str = env::var_os("OUT_DIR").unwrap();
    let out_dir_path = path::Path::new(&out_dir_str);

    println!("cargo:rerun-if-changed=src/v8-trampoline.h");
    println!("cargo:rerun-if-changed=src/v8-glue.h");
    println!("cargo:rerun-if-changed=src/v8-glue.cc");

    let api = read_api();

    link_v8();

    let decl_header_path = out_dir_path.join("v8-glue-decl-generated.h");
    write_decl_header(&api, &mut fs::File::create(&decl_header_path).unwrap()).unwrap();

    let header_path = out_dir_path.join("v8-glue-generated.h");
    write_header(&api, &mut fs::File::create(&header_path).unwrap()).unwrap();

    let cc_file_path = out_dir_path.join("v8-glue-generated.cc");
    write_cc_file(&api, &mut fs::File::create(&cc_file_path).unwrap()).unwrap();

    build_glue(out_dir_path);

    let ffi_path = out_dir_path.join("ffi.rs");
    gen_bindings(out_dir_path, &ffi_path);
}

fn read_api() -> v8_api::Api {
    let mut extra_includes = vec![];

    if path::Path::new("v8-build").exists() {
        extra_includes.push(path::Path::new("v8-build/include").to_path_buf());
    } else if let Some(dir_str) = env::var_os("V8_SOURCE") {
        extra_includes.push(path::PathBuf::from(dir_str).join("include"));
    }

    let clang = clang_sys::support::Clang::find(None).expect("No clang found, is it installed?");
    extra_includes.extend_from_slice(&clang.c_search_paths);

    let trampoline_path = path::Path::new("src/v8-trampoline.h");

    v8_api::read(trampoline_path, &extra_includes)
}

fn link_v8() {
    if path::Path::new("v8-build").exists() {
        // rq build hack
        println!("cargo:rustc-link-search=native=v8-build");
        println!("cargo:rustc-link-lib=static=v8-uber");
    } else if let Ok(libs_str) = env::var("V8_LIBS") {
        println!("V8_LIBS={:?}", libs_str);
        for lib_str in libs_str.split(char::is_whitespace) {
            let path = path::Path::new(lib_str);
            if let Some(dir) = path.parent() {
                if dir.file_name().is_some() {
                    println!("cargo:rustc-link-search=native={}", dir.to_str().unwrap());
                }
            }
            let lib_name = path.file_name().unwrap().to_str().unwrap();
            if lib_name.starts_with("lib") && lib_name.ends_with(".a") {
                println!("cargo:rustc-link-lib=static={}",
                         &lib_name[3..lib_name.len() - 2]);
            }
        }
    } else if let Some(dir_str) = env::var_os("V8_BUILD") {
        println!("using V8_BUILD={:?}", dir_str);
        let dir = path::Path::new(&dir_str);

        maybe_search(dir);

        // make+gyp-based build tree
        maybe_search(dir.join("lib"));
        maybe_search(dir.join("obj.target/src"));
        maybe_search(dir.join("obj.target/third_party/icu"));

        // ninja+gyp-based build tree
        maybe_search(dir.join("lib"));
        maybe_search(dir.join("obj/src"));
        maybe_search(dir.join("obj/third_party/icu"));

        // TODO: for GN-based builds it doesn't seem like the build
        // produces static archives; maybe run ar here?

        blind_link_libraries();
    } else {
        let statik = !cfg!(feature = "shared");
        println!("preferring static linking: {}", statik);
        let result = pkg_config::Config::new()
            .statik(statik)
            .probe("v8");
        if result.is_ok() {
            println!("using pkg-config for library v8");
        } else {
            println!("cargo:warning=pkg-config failed, falling back to naïve lib search: {:?}",
                     result);

            maybe_search("/usr/lib");
            maybe_search("/usr/local/lib");
            // TODO: hack: lazy way to fix the Travis build
            maybe_search("/usr/lib/x86_64-linux-gnu");
            maybe_search("/usr/local/lib/x86_64-linux-gnu");
            maybe_search("/usr/lib/v8");
            maybe_search("/usr/local/lib/v8");
            maybe_search("/usr/local/opt/icu4c/lib"); // homebrew

            blind_link_libraries();
        }
    }
}

fn blind_link_libraries() {
    if cfg!(feature = "shared") {
        if cfg!(all(windows, target_env = "msvc")) {
            println!("cargo:rustc-link-lib=dylib=v8.dll");
            println!("cargo:rustc-link-lib=static=v8_base");
        } else {
            println!("cargo:rustc-link-lib=dylib=v8");
            println!("cargo:rustc-link-lib=dylib=icui18n");
            println!("cargo:rustc-link-lib=dylib=icuuc");
        }
    } else {
        for lib in LIBS.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        println!("cargo:rustc-link-lib=static=icui18n");
        println!("cargo:rustc-link-lib=static=icuuc");
        if fs::metadata("/usr/lib/x86_64-linux-gnu/libicudata.a")
            .map(|m| m.is_file())
            .unwrap_or(false) {
            println!("cargo:rustc-link-lib=static=icudata");
        }
        if fs::metadata("/usr/local/opt/icu4c/lib/libicudata.a")
            .map(|m| m.is_file())
            .unwrap_or(false) {
            println!("cargo:rustc-link-lib=static=icudata");
        }
    }
}

fn maybe_search<P>(dir: P)
    where P: AsRef<path::Path>
{
    let dir = dir.as_ref();
    if fs::metadata(dir).map(|m| m.is_dir()).unwrap_or(false) {
        println!("cargo:rustc-link-search=native={}", dir.to_string_lossy());
    }
}

fn gen_bindings(out_dir_path: &path::Path, bindings_path: &path::Path) {
    use std::io::Write;
    let mut bindings = bindgen::builder()
    .header("src/v8-glue.h")
    .emit_builtins()
    .no_unstable_rust()
    //bindings.remove_prefix("v8_");
    .clang_arg("-Isrc")
    .clang_arg(format!("-I{}", out_dir_path.to_string_lossy()));

    if let Some(dir_str) = env::var_os("V8_SOURCE") {
        println!("V8_SOURCE={:?}", dir_str);
        let dir = path::Path::new(&dir_str);
        bindings = bindings.clang_arg(format!("-I{}", dir.join("include").to_string_lossy()));
    } else {
        println!("V8_SOURCE not set, searching system paths");
    }

    let generated_bindings = bindings.generate().unwrap();

    let mut bindings_file = fs::File::create(bindings_path).unwrap();
    writeln!(bindings_file, "mod ffi {{").unwrap();
    generated_bindings.write(Box::new(&mut bindings_file)).unwrap();
    writeln!(bindings_file, "}}").unwrap();
}

fn build_glue(out_dir_path: &path::Path) {
    let mut config = gcc::Config::new();

    if let Some(dir_str) = env::var_os("V8_SOURCE") {
        let dir = path::Path::new(&dir_str);
        config.include(dir.join("include"));
    }

    if path::Path::new("v8-build").exists() {
        config.include("v8-build/include");
    }

    config.include("src");
    config.include(out_dir_path);
    config.cpp(true);

    if let Ok(target) = env::var("TARGET") {
        if target.contains("musl") {
            // This is a bit of a hack... we know this string is inserted as a
            // "cargo:rustc-link-lib={}" format argument, so we can force it to be linked statically
            // like this.
            config.cpp_link_stdlib(Some("static=stdc++"));
        }
    }

    config.flag("-std=c++11");
    config.flag("-Wall");
    config.file("src/v8-glue.cc");
    config.compile("libv8sysglue.a");
}

fn write_decl_header<W>(api: &v8_api::Api, mut out: W) -> io::Result<()>
    where W: io::Write
{
    try!(writeln!(out, "#pragma once"));

    for class in api.classes.iter() {
        try!(writeln!(out, ""));
        try!(writeln!(out, "#if defined __cplusplus"));
        try!(writeln!(out, "typedef v8::{class} *{class}Ptr;", class = class.name));
        try!(writeln!(out,
                      "typedef v8::Persistent<v8::{class}> *{class}Ref;",
                      class = class.name));
        try!(writeln!(out, "#else"));
        try!(writeln!(out,
                      "typedef struct _{class} *{class}Ptr;",
                      class = class.name));
        try!(writeln!(out,
                      "typedef struct _{class}Ref *{class}Ref;",
                      class = class.name));
        try!(writeln!(out, "#endif /* defined __cplusplus */"));
    }

    Ok(())
}

fn write_header<W>(api: &v8_api::Api, mut out: W) -> io::Result<()>
    where W: io::Write
{
    try!(writeln!(out, "#pragma once"));

    for class in api.classes.iter() {
        try!(writeln!(out, ""));

        for method in class.methods.iter() {
            try!(write!(out,
                        "{retty} v8_{class}_{method}(RustContext c",
                        retty = C(&method.ret_type),
                        class = class.name,
                        method = method.mangled_name));

            if !method.is_static {
                try!(write!(out, ", {class}Ref self", class = class.name));
            }

            for arg in method.args.iter() {
                try!(write!(out, ", {arg}", arg = C(arg)));
            }
            try!(writeln!(out, ");"));
        }

        try!(writeln!(out,
                      "{class}Ref v8_{class}_CloneRef(RustContext c, {class}Ref self);",
                      class = class.name));

        try!(writeln!(out,
                      "void v8_{class}_DestroyRef({class}Ref self);",
                      class = class.name));

        try!(writeln!(out,
                      "void v8_{class}_DestroyPtr({class}Ptr self);",
                      class = class.name));
    }

    Ok(())
}

fn write_cc_file<W>(api: &v8_api::Api, mut out: W) -> io::Result<()>
    where W: io::Write
{
    for class in api.classes.iter() {
        for method in class.methods.iter() {
            try!(writeln!(out, ""));
            try!(write!(out,
                        "{retty} v8_{class}_{method}(RustContext c",
                        retty = C(&method.ret_type),
                        class = class.name,
                        method = method.mangled_name));

            if !method.is_static {
                try!(write!(out, ", {class}Ref self", class = class.name));
            }

            for arg in method.args.iter() {
                try!(write!(out, ", {arg}", arg = C(arg)));
            }
            try!(writeln!(out, ") {{"));

            try!(writeln!(out, "  v8::Isolate::Scope __isolate_scope(c.isolate);"));
            try!(writeln!(out, "  v8::HandleScope __handle_scope(c.isolate);"));
            try!(writeln!(out, "  v8::TryCatch __try_catch(c.isolate);"));

            let context_type = v8_api::Type::Ref(Box::new(v8_api::Type::Class("Context"
                .to_owned())));
            if let Some(arg) = method.args.iter().find(|ref a| a.arg_type == context_type) {
                // There should only be one context but who knows
                try!(writeln!(out,
                              "  auto wrapped_{ctx} = wrap(c.isolate, {ctx});",
                              ctx = arg.name));
                try!(writeln!(out,
                              "  v8::Context::Scope {ctx}_scope(wrapped_{ctx});",
                              ctx = arg.name));
            }

            for arg in method.args.iter() {
                try!(writeln!(out,
                              "  auto {arg}_wrapped = wrap(c.isolate, {arg});",
                              arg = arg.name));
            }

            if let v8_api::RetType::Direct(v8_api::Type::Void) = method.ret_type {
                try!(write!(out, "  "));
            } else {
                try!(write!(out, "  auto result = "));
            }
            if method.is_static {
                try!(write!(out,
                            "v8::{class}::{method}(",
                            class = class.name,
                            method = method.name));
            } else {
                try!(write!(out, "self->Get(c.isolate)->{method}(", method = method.name));
            }

            let mut needs_sep = false;
            for arg in method.args.iter() {
                if needs_sep {
                    try!(write!(out, ", "));
                }
                needs_sep = true;
                try!(write!(out, "{arg}_wrapped", arg = arg.name));
            }
            try!(writeln!(out, ");"));
            if let v8_api::RetType::Direct(v8_api::Type::Void) = method.ret_type {
                try!(writeln!(out, "  handle_exception(c, __try_catch);"));
            } else {
                try!(writeln!(out, "  handle_exception(c, __try_catch);"));
                try!(writeln!(out,
                              "  return {unwrapper}(c.isolate, result);",
                              unwrapper = unwrapper(&method.ret_type)));
            }
            try!(writeln!(out, "}}"));
        }

        try!(writeln!(out, ""));
        try!(writeln!(out,
                      "{class}Ref v8_{class}_CloneRef(RustContext c, {class}Ref self) {{",
                      class = class.name));
        try!(writeln!(out, "  v8::HandleScope __handle_scope(c.isolate);"));
        try!(writeln!(out, "  return unwrap(c.isolate, wrap(c.isolate, self));"));
        try!(writeln!(out, "}}"));
        try!(writeln!(out, ""));
        try!(writeln!(out,
                      "void v8_{class}_DestroyRef({class}Ref self) {{",
                      class = class.name));
        try!(writeln!(out, "  self->Reset();"));
        try!(writeln!(out, "  delete self;"));
        try!(writeln!(out, "}}"));
        try!(writeln!(out, ""));
        try!(writeln!(out,
                      "void v8_{class}_DestroyPtr({class}Ptr self) {{",
                      class = class.name));
        try!(writeln!(out, "  delete self;"));
        try!(writeln!(out, "}}"));
    }

    Ok(())
}

fn unwrapper(ret_type: &v8_api::RetType) -> &str {
    use v8_api::*;
    match *ret_type {
        RetType::Maybe(Type::Bool) => "unwrap_maybe_bool",
        RetType::Maybe(Type::Int) => "unwrap_maybe_int",
        RetType::Maybe(Type::UInt) => "unwrap_maybe_uint",
        RetType::Maybe(Type::Long) => "unwrap_maybe_long",
        RetType::Maybe(Type::ULong) => "unwrap_maybe_ulong",
        RetType::Maybe(Type::U32) => "unwrap_maybe_u32",
        RetType::Maybe(Type::I32) => "unwrap_maybe_i32",
        RetType::Maybe(Type::U64) => "unwrap_maybe_u64",
        RetType::Maybe(Type::I64) => "unwrap_maybe_i64",
        RetType::Maybe(Type::F64) => "unwrap_maybe_f64",
        _ => "unwrap",
    }
}

impl DisplayAsC for v8_api::Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", C(&self.arg_type), self.name)
    }
}

impl DisplayAsC for v8_api::RetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use v8_api::*;
        match *self {
            RetType::Direct(ref t) => DisplayAsC::fmt(t, f),
            RetType::Maybe(Type::Bool) => write!(f, "MaybeBool"),
            RetType::Maybe(Type::Int) => write!(f, "MaybeInt"),
            RetType::Maybe(Type::UInt) => write!(f, "MaybeUInt"),
            RetType::Maybe(Type::Long) => write!(f, "MaybeLong"),
            RetType::Maybe(Type::ULong) => write!(f, "MaybeULong"),
            RetType::Maybe(Type::F64) => write!(f, "MaybeF64"),
            RetType::Maybe(Type::U64) => write!(f, "MaybeU64"),
            RetType::Maybe(Type::I64) => write!(f, "MaybeI64"),
            // TODO: potentially maybeify more types here
            RetType::Maybe(ref t) => DisplayAsC::fmt(t, f),
        }
    }
}

impl DisplayAsC for v8_api::Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use v8_api::*;
        match *self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "bool"),
            Type::UChar => write!(f, "char"),
            Type::ConstUChar => write!(f, "const char"),
            Type::Char => write!(f, "char"),
            Type::ConstChar => write!(f, "const char"),
            Type::UInt => write!(f, "unsigned int"),
            Type::Int => write!(f, "int"),
            Type::ULong => write!(f, "unsigned long"),
            Type::Long => write!(f, "long"),
            Type::U8 => write!(f, "uint8_t"),
            Type::I8 => write!(f, "int8_t"),
            Type::U16 => write!(f, "uint16_t"),
            Type::I16 => write!(f, "int16_t"),
            Type::U32 => write!(f, "uint32_t"),
            Type::I32 => write!(f, "int32_t"),
            Type::U64 => write!(f, "uint64_t"),
            Type::I64 => write!(f, "int64_t"),
            Type::F64 => write!(f, "double"),
            Type::USize => write!(f, "size_t"),
            Type::Class(ref name) => write!(f, "{}", name),
            Type::Enum(ref name) => write!(f, "{}", name),
            Type::Callback(ref name) => write!(f, "{}", name),
            Type::CallbackLValue(ref name) => write!(f, "{}", name),
            Type::Ref(ref target) => {
                match **target {
                    Type::Class(ref name) => write!(f, "{}Ref", name),
                    ref t => write!(f, "&{}", C(t)),
                }
            }
            Type::Ptr(ref target) => {
                match **target {
                    Type::Class(ref name) => write!(f, "{}Ptr", name),
                    ref t => write!(f, "{} *", C(t)),
                }
            }
            Type::Arr(ref target) => write!(f, "{}[]", C(&**target)),
        }
    }
}
