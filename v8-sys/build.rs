extern crate bindgen;
extern crate gcc;

use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path;

struct Class(&'static str, &'static [Method]);

struct Method(&'static str, &'static [Arg], RetType);

enum RetType {
    Direct(Type),
    Maybe(Type),
}

struct Arg(&'static str, Type);

#[derive(Debug)]
enum Type {
    ValBool,
    ValInt,
    ValF64,
    ValU32,
    ValI32,

    Value,
    Boolean,
    Number,
    String,
    DetailString,
    Object,
    Integer,
    Uint32,
    Int32,
    Context,
    Script,
    UnboundScript,
}

const NS: &'static str = "v8";

const LIBS: [&'static str; 6] = ["v8_base",
                                 "v8_libbase",
                                 "v8_libsampler",
                                 "v8_nosnapshot",
                                 "icui18n",
                                 "icuuc"];

fn main() {
    let out_dir_str = env::var_os("OUT_DIR").unwrap();
    let out_dir_path = path::Path::new(&out_dir_str);

    link_v8();

    let header_path = out_dir_path.join("v8-glue-generated.h");
    write_header(&mut fs::File::create(&header_path).unwrap()).unwrap();

    let cc_file_path = out_dir_path.join("v8-glue-generated.cc");
    write_cc_file(&mut fs::File::create(&cc_file_path).unwrap()).unwrap();

    build_glue(out_dir_path);

    let ffi_path = out_dir_path.join("ffi.rs");
    gen_bindings(out_dir_path, &ffi_path);
}

fn link_v8() {
    if let Some(dir_str) = env::var_os("V8_BUILD") {
        println!("V8_BUILD={:?}", dir_str);
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
    } else {
        println!("V8_BUILD not set, searching system paths");
        maybe_search("/usr/lib");
        maybe_search("/usr/local/lib");
        // TODO: hack: lazy way to fix the Travis build
        maybe_search("/usr/lib/x86_64-linux-gnu");
        maybe_search("/usr/local/lib/x86_64-linux-gnu");
        maybe_search("/usr/lib/v8");
        maybe_search("/usr/local/lib/v8");
    }

    if cfg!(feature = "shared") {
        println!("cargo:rustc-link-lib=dylib=v8");
        println!("cargo:rustc-link-lib=dylib=icui18n");
        println!("cargo:rustc-link-lib=dylib=icuuc");
    } else {
        for lib in LIBS.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
    }
}

fn maybe_search<P>(dir: P) where P: AsRef<path::Path> {
    let dir = dir.as_ref();
    if fs::metadata(dir).map(|m| m.is_dir()).unwrap_or(false) {
        println!("cargo:rustc-link-search=native={}", dir.to_string_lossy());
    }
}

fn gen_bindings(out_dir_path: &path::Path, bindings_path: &path::Path) {
    use std::io::Write;

    println!("cargo:rerun-if-changed=src/v8-glue.h");
    let mut bindings = bindgen::Builder::new("src/v8-glue.h");
    bindings.remove_prefix(format!("{}_", NS));
    bindings.clang_arg("-Isrc");
    bindings.clang_arg(format!("-I{}", out_dir_path.to_string_lossy()));

    if let Some(dir_str) = env::var_os("V8_SOURCE") {
        println!("V8_SOURCE={:?}", dir_str);
        let dir = path::Path::new(&dir_str);
        bindings.clang_arg(format!("-I{}", dir.join("include").to_string_lossy()));
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

    println!("cargo:rerun-if-changed=src/v8-glue.cc");
    config.include("src");
    config.include(out_dir_path);
    config.cpp(true);
    config.flag("-std=c++11");
    config.file("src/v8-glue.cc");
    config.compile("libv8sysglue.a");
}

fn write_header<W>(mut out: W) -> io::Result<()>
    where W: io::Write
{
    try!(writeln!(out, "#pragma once"));

    for class in API.iter() {
        try!(writeln!(out, ""));
        try!(writeln!(out, "#if defined __cplusplus"));
        try!(writeln!(out,
                      "typedef v8::Persistent<v8::{class}> {class};",
                      class = class.0));
        try!(writeln!(out, "#else"));
        try!(writeln!(out, "typedef void {class};", class = class.0));
        try!(writeln!(out, "#endif /* defined __cplusplus */"));
    }

    for class in API.iter() {
        try!(writeln!(out, ""));

        for method in class.1.iter() {
            try!(write!(out,
                        "{retty} {ns}_{class}_{method}(Isolate *isolate, {class} *self",
                        ns = NS,
                        retty = method.2,
                        class = class.0,
                        method = method.0));

            for arg in method.1.iter() {
                try!(write!(out, ", {arg}", arg = arg));
            }
            try!(writeln!(out, ");"));
        }
        try!(writeln!(out,
                      "void {ns}_{class}_Destroy(Isolate *isolate, {class} *self);",
                      ns = NS,
                      class = class.0));
    }

    Ok(())
}

fn write_cc_file<W>(mut out: W) -> io::Result<()>
    where W: io::Write
{
    for class in API.iter() {
        for method in class.1.iter() {
            try!(writeln!(out, ""));
            try!(write!(out,
                        "{retty} {ns}_{class}_{method}(v8::Isolate *isolate, {class} *self",
                        ns = NS,
                        retty = method.2,
                        class = class.0,
                        method = method.0));

            for arg in method.1.iter() {
                try!(write!(out, ", {arg}", arg = arg));
            }
            try!(writeln!(out, ") {{"));
            try!(writeln!(out, "  v8::HandleScope scope(isolate);"));
            try!(write!(out,
                        "  return unwrap(isolate, self->Get(isolate)->{method}(",
                        method = method.0));
            let mut needs_sep = false;
            for arg in method.1.iter() {
                if needs_sep {
                    try!(write!(out, ", "));
                }
                needs_sep = true;
                try!(write!(out, "wrap(isolate, {arg})", arg = arg.0));
            }
            try!(writeln!(out, "));"));
            try!(writeln!(out, "}}"));
        }

        try!(writeln!(out, ""));
        try!(writeln!(out,
                      "void {ns}_{class}_Destroy(v8::Isolate *isolate, {class} *self) {{",
                      ns = NS,
                      class = class.0));
        try!(writeln!(out, "  delete self;"));
        try!(writeln!(out, "}}"));
    }

    Ok(())
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{type} {name}", name=self.0, type=self.1)
    }
}

impl fmt::Display for RetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RetType::Direct(ref t) => t.fmt(f),
            RetType::Maybe(ref t) => t.fmt(f),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Type::ValBool => write!(f, "bool"),
            Type::ValF64 => write!(f, "double"),
            Type::ValU32 => write!(f, "uint32_t"),
            Type::ValI32 => write!(f, "int32_t"),
            Type::ValInt => write!(f, "int"),
            ref r => write!(f, "{:?} *", r),
        }
    }
}

const API: &'static [Class] =
    &[Class("ScriptOrigin",
            &[Method("ResourceName", &[], RetType::Direct(Type::Value)),
              Method("ResourceLineOffset", &[], RetType::Direct(Type::Integer)),
              Method("ResourceColumnOffset", &[], RetType::Direct(Type::Integer)),
              Method("ScriptID", &[], RetType::Direct(Type::Integer)),
              Method("SourceMapUrl", &[], RetType::Direct(Type::Value))]),
      Class("UnboundScript",
            &[Method("GetId", &[], RetType::Direct(Type::ValInt)),
              Method("GetScriptName", &[], RetType::Direct(Type::Value)),
              Method("GetSourceURL", &[], RetType::Direct(Type::Value)),
              Method("GetSourceMappingURL", &[], RetType::Direct(Type::Value))]),
      Class("Script",
            &[Method("Run",
                     &[Arg("context", Type::Context)],
                     RetType::Maybe(Type::Value)),
              Method("GetUnboundScript", &[], RetType::Maybe(Type::UnboundScript))]),
      Class("ScriptCompiler", &[]),
      Class("Message", &[]),
      Class("StackTrace", &[]),
      Class("StackFrame", &[]),
      Class("JSON", &[]),
      Class("NativeWeakMap", &[]),
      // Values
      Class("Value",
            &[Method("IsUndefined", &[], RetType::Direct(Type::ValBool)),
              Method("IsNull", &[], RetType::Direct(Type::ValBool)),
              Method("IsTrue", &[], RetType::Direct(Type::ValBool)),
              Method("IsFalse", &[], RetType::Direct(Type::ValBool)),
              Method("IsName", &[], RetType::Direct(Type::ValBool)),
              Method("IsString", &[], RetType::Direct(Type::ValBool)),
              Method("IsSymbol", &[], RetType::Direct(Type::ValBool)),
              Method("IsFunction", &[], RetType::Direct(Type::ValBool)),
              Method("IsArray", &[], RetType::Direct(Type::ValBool)),
              Method("IsObject", &[], RetType::Direct(Type::ValBool)),
              Method("IsBoolean", &[], RetType::Direct(Type::ValBool)),
              Method("IsNumber", &[], RetType::Direct(Type::ValBool)),
              Method("IsExternal", &[], RetType::Direct(Type::ValBool)),
              Method("IsInt32", &[], RetType::Direct(Type::ValBool)),
              Method("IsUint32", &[], RetType::Direct(Type::ValBool)),
              Method("IsArgumentsObject", &[], RetType::Direct(Type::ValBool)),
              Method("IsBooleanObject", &[], RetType::Direct(Type::ValBool)),
              Method("IsNumberObject", &[], RetType::Direct(Type::ValBool)),
              Method("IsStringObject", &[], RetType::Direct(Type::ValBool)),
              Method("IsSymbolObject", &[], RetType::Direct(Type::ValBool)),
              Method("IsNativeError", &[], RetType::Direct(Type::ValBool)),
              Method("IsRegExp", &[], RetType::Direct(Type::ValBool)),
              Method("IsGeneratorFunction", &[], RetType::Direct(Type::ValBool)),
              Method("IsGeneratorObject", &[], RetType::Direct(Type::ValBool)),
              Method("IsPromise", &[], RetType::Direct(Type::ValBool)),
              Method("IsMap", &[], RetType::Direct(Type::ValBool)),
              Method("IsSet", &[], RetType::Direct(Type::ValBool)),
              Method("IsMapIterator", &[], RetType::Direct(Type::ValBool)),
              Method("IsSetIterator", &[], RetType::Direct(Type::ValBool)),
              Method("IsWeakMap", &[], RetType::Direct(Type::ValBool)),
              Method("IsWeakSet", &[], RetType::Direct(Type::ValBool)),
              Method("IsArrayBuffer", &[], RetType::Direct(Type::ValBool)),
              Method("IsArrayBufferView", &[], RetType::Direct(Type::ValBool)),
              Method("IsTypedArray", &[], RetType::Direct(Type::ValBool)),
              Method("IsUint8Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsUint8ClampedArray", &[], RetType::Direct(Type::ValBool)),
              Method("IsInt8Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsUint16Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsInt16Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsUint32Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsInt32Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsFloat32Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsFloat64Array", &[], RetType::Direct(Type::ValBool)),
              Method("IsFloat32x4", &[], RetType::Direct(Type::ValBool)),
              Method("IsDataView", &[], RetType::Direct(Type::ValBool)),
              Method("IsSharedArrayBuffer", &[], RetType::Direct(Type::ValBool)),
              Method("IsProxy", &[], RetType::Direct(Type::ValBool)),
              Method("IsWebAssemblyCompiledModule",
                     &[],
                     RetType::Direct(Type::ValBool)),
              Method("ToBoolean",
                     &[Arg("context", Type::Context)],
                     RetType::Maybe(Type::Boolean)),
              Method("ToString",
                     &[Arg("context", Type::Context)],
                     RetType::Maybe(Type::String))]),
      Class("Boolean", &[]),
      Class("Name", &[]),
      Class("String", &[
          Method("Length", &[], RetType::Direct(Type::ValInt)),
          Method("Utf8Length", &[], RetType::Direct(Type::ValInt))]),
      Class("Symbol", &[]),
      Class("Private", &[]),
      Class("Number", &[]),
      Class("Integer", &[]),
      Class("Int32", &[]),
      Class("Uint32", &[]),
      Class("Object", &[]),
      Class("Array", &[]),
      Class("Map", &[]),
      Class("Set", &[]),
      Class("Function", &[]),
      Class("Promise", &[]),
      Class("Proxy", &[]),
      Class("WasmCompiledModule", &[]),
      Class("ArrayBuffer", &[]),
      Class("ArrayBufferView", &[]),
      Class("TypedArray", &[]),
      Class("Uint8Array", &[]),
      Class("Uint8ClampedArray", &[]),
      Class("Int8Array", &[]),
      Class("Uint16Array", &[]),
      Class("Int16Array", &[]),
      Class("Uint32Array", &[]),
      Class("Int32Array", &[]),
      Class("Float32Array", &[]),
      Class("Float64Array", &[]),
      Class("DataView", &[]),
      Class("SharedArrayBuffer", &[]),
      Class("Date", &[]),
      Class("NumberObject", &[]),
      Class("BooleanObject", &[]),
      Class("StringObject", &[]),
      Class("SymbolObject", &[]),
      Class("RegExp", &[]),
      Class("External", &[]),
      // Templates
      Class("Template", &[]),
      Class("FunctionTemplate", &[]),
      Class("Signature", &[]),
      Class("AccessorSignature", &[]),
      // Tracing
      Class("Exception", &[]),
      // Context
      Class("Context", &[])];
