extern crate bindgen;
extern crate clang;
extern crate gcc;

mod api;

use api::*;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path;

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

    println!("cargo:warning={:?}", parse_api());

    link_v8();

    let header_path = out_dir_path.join("v8-glue-generated.h");
    write_header(&mut fs::File::create(&header_path).unwrap()).unwrap();

    let cc_file_path = out_dir_path.join("v8-glue-generated.cc");
    write_cc_file(&mut fs::File::create(&cc_file_path).unwrap()).unwrap();

    build_glue(out_dir_path);

    let ffi_path = out_dir_path.join("ffi.rs");
    gen_bindings(out_dir_path, &ffi_path);
}

fn parse_api() -> api::Api {
    let v8_header_path = if let Some(dir_str) = env::var_os("V8_SOURCE") {
        path::Path::new(&dir_str).join("include").join("v8.h")
    } else {
        unimplemented!()
    };

    api::read(&v8_header_path)
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
        if fs::metadata("/usr/lib/x86_64-linux-gnu/libicudata.a").map(|m| m.is_file()).unwrap_or(false) {
            println!("cargo:rustc-link-lib=static=icudata");
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
                        "{retty} {ns}_{class}_{method}(RustContext c, {class} *self",
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
                      "void {ns}_{class}_Destroy({class} *self);",
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
                        "{retty} {ns}_{class}_{method}(RustContext c, {class} *self",
                        ns = NS,
                        retty = method.2,
                        class = class.0,
                        method = method.0));

            for arg in method.1.iter() {
                try!(write!(out, ", {arg}", arg = arg));
            }
            try!(writeln!(out, ") {{"));
            try!(writeln!(out, "  v8::HandleScope scope(c.isolate);"));
            try!(writeln!(out, "  v8::TryCatch try_catch(c.isolate);"));
            if let Some(&Arg(ctx, Type::Ptr("Context"))) = method.1.iter().next() {
                try!(writeln!(out, "  v8::Context::Scope {ctx}_scope(wrap(c.isolate, {ctx}));", ctx=ctx));
            }
            try!(write!(out,
                        "  auto result = self->Get(c.isolate)->{method}(",
                        method = method.0));
            let mut needs_sep = false;
            for arg in method.1.iter() {
                if needs_sep {
                    try!(write!(out, ", "));
                }
                needs_sep = true;
                try!(write!(out, "wrap(c.isolate, {arg})", arg = arg.0));
            }
            try!(writeln!(out, ");"));
            try!(writeln!(out, "  handle_exception(c, try_catch);"));
            try!(writeln!(out, "  return {retunwrap}(c.isolate, result);", retunwrap = method.2.unwrap_fun()));
            try!(writeln!(out, "}}"));
        }

        try!(writeln!(out, ""));
        try!(writeln!(out,
                      "void {ns}_{class}_Destroy({class} *self) {{",
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

impl RetType {
    fn unwrap_fun(&self) -> &'static str {
            match *self {
                RetType::Direct(_) => "unwrap",
                RetType::Maybe(Type::ValBool) =>"unwrap_bool",
                RetType::Maybe(Type::ValF64) => "unwrap_double",
                RetType::Maybe(Type::ValU32) => "unwrap_uint32_t",
                RetType::Maybe(Type::ValI32) => "unwrap_int32_t",
                RetType::Maybe(Type::ValU64) => "unwrap_uint64_t",
                RetType::Maybe(Type::ValI64) => "unwrap_int64_t",
                RetType::Maybe(Type::ValInt) => "unwrap_int",
                RetType::Maybe(Type::Ptr(_)) => "unwrap",
            }
    }
}

impl fmt::Display for RetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RetType::Direct(ref t) => t.fmt(f),
            RetType::Maybe(Type::ValBool) => write!(f, "struct MaybeBool"),
            RetType::Maybe(Type::ValF64) => write!(f, "struct MaybeF64"),
            RetType::Maybe(Type::ValU32) => write!(f, "struct MaybeU32"),
            RetType::Maybe(Type::ValI32) => write!(f, "struct MaybeI32"),
            RetType::Maybe(Type::ValU64) => write!(f, "struct MaybeU64"),
            RetType::Maybe(Type::ValI64) => write!(f, "struct MaybeI64"),
            RetType::Maybe(Type::ValInt) => write!(f, "struct MaybeInt"),
            RetType::Maybe(Type::Ptr(target)) => write!(f, "{} *", target),
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
            Type::ValU64 => write!(f, "uint64_t"),
            Type::ValI64 => write!(f, "int64_t"),
            Type::ValInt => write!(f, "int"),
            Type::Ptr(target) => write!(f, "{} *", target),
        }
    }
}

const API: &'static [Class] =
    &[Class("ScriptOrigin",
            &[Method("ResourceName", &[], RetType::Direct(Type::Ptr("Value"))),
              Method("ResourceLineOffset", &[], RetType::Direct(Type::Ptr("Integer"))),
              Method("ResourceColumnOffset", &[], RetType::Direct(Type::Ptr("Integer"))),
              Method("ScriptID", &[], RetType::Direct(Type::Ptr("Integer"))),
              Method("SourceMapUrl", &[], RetType::Direct(Type::Ptr("Value")))
              // TODO: add Options
            ]),
      Class("UnboundScript",
            &[Method("GetId", &[], RetType::Direct(Type::ValInt)),
              Method("GetScriptName", &[], RetType::Direct(Type::Ptr("Value"))),
              Method("GetSourceURL", &[], RetType::Direct(Type::Ptr("Value"))),
              Method("GetSourceMappingURL", &[], RetType::Direct(Type::Ptr("Value"))),
            Method("GetLineNumber", &[Arg("code_pos", Type::ValInt)], RetType::Direct(Type::ValInt))]),
      Class("Script",
            &[Method("Run",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Value"))),
              Method("GetUnboundScript", &[], RetType::Maybe(Type::Ptr("UnboundScript")))]),
      Class("ScriptCompiler", &[
          // TODO: methods
      ]),
      Class("Message", &[
          Method("Get", &[], RetType::Direct(Type::Ptr("String"))),
          Method("GetSourceLine", &[Arg("context", Type::Ptr("Context"))], RetType::Maybe(Type::Ptr("String"))),
          // Method("GetScriptOrigin", &[], RetType::Direct(Type::Ptr("ScriptOrigin"))),
          Method("GetScriptResourceName", &[], RetType::Direct(Type::Ptr("Value"))),
          Method("GetStackTrace", &[], RetType::Direct(Type::Ptr("StackTrace"))),
          Method("GetLineNumber", &[Arg("context", Type::Ptr("Context"))], RetType::Maybe(Type::ValInt)),
          Method("GetStartPosition", &[], RetType::Direct(Type::ValInt)),
          Method("GetEndPosition", &[], RetType::Direct(Type::ValInt)),
          Method("GetStartColumn", &[Arg("context", Type::Ptr("Context"))], RetType::Maybe(Type::ValInt)),
          Method("GetEndColumn", &[Arg("context", Type::Ptr("Context"))], RetType::Maybe(Type::ValInt)),
          Method("IsSharedCrossOrigin", &[], RetType::Direct(Type::ValBool)),
          Method("IsOpaque", &[], RetType::Direct(Type::ValBool)),
      ]),
      Class("StackTrace", &[
          Method("GetFrame", &[Arg("index", Type::ValU32)], RetType::Direct(Type::Ptr("StackFrame"))),
          Method("GetFrameCount", &[], RetType::Direct(Type::ValInt)),
          Method("AsArray", &[], RetType::Direct(Type::Ptr("Array"))),
      ]),
      Class("StackFrame", &[
          Method("GetLineNumber", &[], RetType::Direct(Type::ValInt)),
          Method("GetColumn", &[], RetType::Direct(Type::ValInt)),
          Method("GetScriptId", &[], RetType::Direct(Type::ValInt)),
          Method("GetScriptName", &[], RetType::Direct(Type::Ptr("String"))),
          Method("GetScriptNameOrSourceURL", &[], RetType::Direct(Type::Ptr("String"))),
          Method("GetFunctionName", &[], RetType::Direct(Type::Ptr("String"))),
          Method("IsEval", &[], RetType::Direct(Type::ValBool)),
          Method("IsConstructor", &[], RetType::Direct(Type::ValBool)),
      ]),
      Class("JSON", &[
          Method("Parse", &[Arg("context", Type::Ptr("Context")), Arg("json_string", Type::Ptr("String"))], RetType::Maybe(Type::Ptr("Value"))),
          Method("Stringify", &[Arg("context", Type::Ptr("Context")), Arg("json_object", Type::Ptr("Object"))], RetType::Maybe(Type::Ptr("String"))),
      ]),
      Class("NativeWeakMap", &[
          // TODO: methods
      ]),
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
              Method("IsDate", &[], RetType::Direct(Type::ValBool)),
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
              Method("IsDataView", &[], RetType::Direct(Type::ValBool)),
              Method("IsSharedArrayBuffer", &[], RetType::Direct(Type::ValBool)),
              Method("IsProxy", &[], RetType::Direct(Type::ValBool)),
              Method("IsWebAssemblyCompiledModule",
                     &[],
                     RetType::Direct(Type::ValBool)),
              Method("ToBoolean",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Boolean"))),
              Method("ToNumber",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Number"))),
              Method("ToString",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("String"))),
              Method("ToDetailString",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("String"))),
              Method("ToObject",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Object"))),
              Method("ToInteger",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Integer"))),
              Method("ToUint32",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Uint32"))),
              Method("ToInt32",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Int32"))),
              Method("ToArrayIndex",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::Ptr("Uint32"))),
              Method("BooleanValue",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::ValBool)),
              Method("NumberValue",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::ValF64)),
              Method("IntegerValue",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::ValI64)),
              Method("Uint32Value",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::ValU32)),
              Method("Int32Value",
                     &[Arg("context", Type::Ptr("Context"))],
                     RetType::Maybe(Type::ValI32)),
              Method("Equals",
                     &[Arg("context", Type::Ptr("Context")), Arg("that", Type::Ptr("Value"))],
                     RetType::Maybe(Type::ValBool)),
              Method("StrictEquals",
                     &[Arg("that", Type::Ptr("Value"))],
                     RetType::Direct(Type::ValBool)),
              Method("SameValue",
                     &[Arg("that", Type::Ptr("Value"))],
                     RetType::Direct(Type::ValBool)),
            ]),
      Class("Primitive", &[]),
      Class("Boolean", &[
          Method("Value", &[], RetType::Direct(Type::ValBool))
      ]),
      Class("Name", &[
          Method("GetIdentityHash", &[], RetType::Direct(Type::ValInt))
      ]),
      Class("String", &[
          Method("Length", &[], RetType::Direct(Type::ValInt)),
          Method("Utf8Length", &[], RetType::Direct(Type::ValInt)),
          Method("IsOneByte", &[], RetType::Direct(Type::ValBool)),
          Method("ContainsOnlyOneByte", &[], RetType::Direct(Type::ValBool)),
          Method("IsExternal", &[], RetType::Direct(Type::ValBool)),
          Method("IsExternalOneByte", &[], RetType::Direct(Type::ValBool)),
          Method("Concat", &[Arg("left", Type::Ptr("String")), Arg("right", Type::Ptr("String"))], RetType::Direct(Type::Ptr("String"))),
      ]),
      Class("Symbol", &[
      ]),
      Class("Private", &[
          Method("Name", &[], RetType::Direct(Type::Ptr("Value")))
      ]),
      Class("Number", &[
          Method("Value", &[], RetType::Direct(Type::ValF64))
      ]),
      Class("Integer", &[
          Method("Value", &[], RetType::Direct(Type::ValI64))
      ]),
      Class("Int32", &[
          Method("Value", &[], RetType::Direct(Type::ValI32))
      ]),
      Class("Uint32", &[
          Method("Value", &[], RetType::Direct(Type::ValU32))
      ]),
      Class("Object", &[
          // TODO: add index things
          Method("Set", &[Arg("context", Type::Ptr("Context")),
                          Arg("key", Type::Ptr("Value")),
                          Arg("value", Type::Ptr("Value"))], RetType::Maybe(Type::ValBool)),
          Method("CreateDataProperty", &[Arg("context", Type::Ptr("Context")),
                                         Arg("key", Type::Ptr("Name")),
                                         Arg("value", Type::Ptr("Value"))], RetType::Maybe(Type::ValBool)),
          Method("Get", &[Arg("context", Type::Ptr("Context")),
                          Arg("key", Type::Ptr("Value"))], RetType::Maybe(Type::Ptr("Value"))),
          Method("GetOwnPropertyDescriptor", &[Arg("context", Type::Ptr("Context")),
                                               Arg("key", Type::Ptr("String"))], RetType::Maybe(Type::Ptr("Value"))),
          Method("Has", &[Arg("context", Type::Ptr("Context")),
                          Arg("key", Type::Ptr("Value"))], RetType::Maybe(Type::ValBool)),
          Method("Delete", &[Arg("context", Type::Ptr("Context")),
                             Arg("key", Type::Ptr("Value"))], RetType::Maybe(Type::ValBool)),
          Method("GetPropertyNames", &[Arg("context", Type::Ptr("Context"))], RetType::Maybe(Type::Ptr("Array"))),
          Method("GetOwnPropertyNames", &[Arg("context", Type::Ptr("Context"))], RetType::Maybe(Type::Ptr("Array"))),
          Method("GetPrototype", &[], RetType::Direct(Type::Ptr("Value"))),
          Method("SetPrototype", &[Arg("context", Type::Ptr("Context")),
                                   Arg("value", Type::Ptr("Value"))], RetType::Maybe(Type::ValBool)),
          Method("ObjectProtoToString", &[Arg("context", Type::Ptr("Context"))], RetType::Maybe(Type::Ptr("String"))),
          Method("GetConstructorName", &[], RetType::Direct(Type::Ptr("String"))),
          Method("HasOwnProperty", &[Arg("context", Type::Ptr("Context")),
                                     Arg("key", Type::Ptr("Name"))], RetType::Maybe(Type::ValBool)),
          Method("HasRealNamedProperty", &[Arg("context", Type::Ptr("Context")),
                                           Arg("key", Type::Ptr("Name"))], RetType::Maybe(Type::ValBool)),
          Method("HasRealIndexedProperty", &[Arg("context", Type::Ptr("Context")),
                                             Arg("key", Type::ValU32)], RetType::Maybe(Type::ValBool)),
          Method("GetIdentityHash", &[], RetType::Direct(Type::ValInt)),
          Method("Clone", &[], RetType::Direct(Type::Ptr("Object"))),
          Method("CreationContext", &[], RetType::Direct(Type::Ptr("Context"))),
          Method("IsCallable", &[], RetType::Direct(Type::ValBool)),
          Method("IsConstructor", &[], RetType::Direct(Type::ValBool)),
      ]),
      Class("Array", &[
          Method("Length", &[], RetType::Direct(Type::ValU32))
      ]),
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
