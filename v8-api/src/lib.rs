extern crate clang;
#[macro_use]
extern crate log;

use std::env;
use std::fmt;
use std::path;

/// A description of the V8 API.
#[derive(Debug)]
pub struct Api {
    /// The classes that the API consists of.
    pub classes: Vec<Class>,
}

/// A C++ class,
#[derive(Debug)]
pub struct Class {
    /// The simple name of the class (without the `v8::` prefix).
    pub name: String,
    /// The methods of this class, in declaration order.
    pub methods: Vec<Method>,
}

/// A C++ method
#[derive(Debug)]
pub struct Method {
    /// Whether the method is static.
    pub is_static: bool,
    /// The name of the method.
    pub name: String,
    /// A mangled version of the method that makes it unique among all
    /// of the methods of its class.
    pub mangled_name: String,
    /// The arguments that the method takes, in declaration order.
    pub args: Vec<Arg>,
    /// The return type of the method.
    pub ret_type: RetType,
}

/// The return type of a method.
#[derive(Debug)]
pub enum RetType {
    /// The type is directly returned.  For primitives `T`, this means
    /// just `T` (e.g. `int`).  For references to `T`, this means
    /// `Local<T>` (e.g. `Local<String>`).  For pointers to `T`, this
    /// means a non-null pointer `T *`.
    Direct(Type),
    /// The type might be absent.  For primitives `T`, this means
    /// `Maybe<T>` (e.g. `Maybe<int>`).  For references to `T`, this
    /// means `MaybeLocal<T>`.  For pointers to `T`, this means a
    /// nullable pointer `T *`.
    Maybe(Type),
}

/// A method argument.
#[derive(Debug)]
pub struct Arg {
    /// The argument name.
    pub name: String,
    /// The type of the argument.
    pub arg_type: Type,
}

/// The types used in V8.
#[derive(Debug, Eq, PartialEq)]
pub enum Type {
    /// The `void` type.
    Void,
    /// The `bool` type.
    Bool,

    /// The `char` type.
    Char,
    /// The `const char` type.
    ConstChar,
    /// The `unsigned int` type.
    UInt,
    /// The `int` type.
    Int,
    /// The `unsigned long` type.
    ULong,
    /// The `long` type.
    Long,

    /// The `uint8_t` type.
    U8,
    /// The `int8_t` type.
    I8,
    /// The `uint16_t` type.
    U16,
    /// The `int16_t` type.
    I16,
    /// The `uint32_t` type.
    U32,
    /// The `int32_t` type.
    I32,
    /// The `uint64_t` type.
    U64,
    /// The `int64_t` type.
    I64,
    /// The `double` type.
    F64,

    /// The `size_t` type.
    USize,

    /// A class with the specified name, without the `v8::` prefix.
    Class(String),
    /// An enum with the specified name, without the `v8::` prefix.
    Enum(String),
    /// A callback function pointer name, without the `v8::` prefix.
    Callback(String),
    /// An argument to a callback
    CallbackLValue(String),

    /// A reference to the specified type, meaning a `Local<T>` or
    /// `MaybeLocal<T>`.
    Ref(Box<Type>),
    /// A pointer to the specified type, i.e. `T *`.
    Ptr(Box<Type>),
    /// An array of the specified type, i.e. `T[]`.
    Arr(Box<Type>),
}

/// A method mangle rule.
struct MethodMangle {
    /// The exact name of the method to mangle.
    name: &'static str,
    /// A unique part of the method signature.
    signature: &'static str,
    /// The mangled name of the method.
    mangle: &'static str,
}

/// Classes that we should not return because they are special.
#[cfg_attr(rustfmt, rustfmt_skip)]
const SPECIAL_CLASSES: &'static [&'static str] = &[
    // v8.h
    // Allocation stuff
    "Isolate", // Because part of RustContext (so chicken and egg methods)
    "HandleScope", // Because stack local
    "EscapableHandleScope", // Because stack local
    "SealHandleScope", // Because stack local
    "TryCatch", // Because stack local

    // Annoying classes
    "ScriptOrigin", // Because it's sent around by-value
    "PersistentHandleVisitor", // Only used on Isolate?
    "EmbedderHeapTracer", // Can't be heap allocated?
    "ValueSerializer", // Weird API and only on modern V8's
    "ValueDeserializer", // Weird API and only on modern V8's
    "ExtensionConfiguration", // Weird API
    "Module", // Too experimental
    "SnapshotCreator", // Snapshots not supported

    // v8-platform.h
    // These are all pre-requisites for creating an Isolate so
    // hand-roll them
    "Platform",
    "Task",
    "IdleTask",
];

/// Methods that we should not return because they are special.
#[cfg_attr(rustfmt, rustfmt_skip)]
const SPECIAL_METHODS: &'static [(&'static str, &'static str)] = &[
    ("Script", "Compile"), // Because ScriptOrigin param
    ("Message", "GetScriptOrigin"), // Because ScriptOrigin
    ("String", "WriteUtf8"), // Because annoying-to-map signature
    ("Object", "SetAlignedPointerInInternalFields"), // Because annoying-to-map signature
    ("Object", "CallAsFunction"), // Because annoying-to-map signature
    ("Object", "CallAsConstructor"), // Because annoying-to-map signature
    ("Object", "NewInstance"), // Because annoying-to-map signature
    ("Object", "Call"), // Because annoying-to-map signature
    ("Function", "New"), // Because annoying-to-map signature
    ("Function", "GetScriptOrigin"), // Because ScriptOrigin
    ("Function", "NewInstance"), // Because annoying-to-map signature
    ("Function", "Call"), // Because annoying-to-map signature
    ("Template", "SetNativeDataProperty"), // Because annoying-to-map signature
    ("Template", "SetLazyDataProperty"), // Because annoying-to-map signature
    ("FunctionTemplate", "New"), // Because annoying-to-map signature
    ("FunctionTemplate", "NewWithCache"), // Because annoying-to-map signature
    ("ObjectTemplate", "SetAccessor"), // Because annoying-to-map signature
    ("ObjectTemplate", "SetNamedPropertyHandler"), // Because annoying-to-map signature
    ("ObjectTemplate", "SetIndexedPropertyHandler"), // Because annoying-to-map signature
    ("ObjectTemplate", "SetCallAsFunctionHandler"), // Because annoying-to-map signature
    ("ObjectTemplate", "SetAccessCheckCallback"), // Because annoying-to-map signature
    ("ObjectTemplate", "SetAccessCheckCallbackAndHandler"), // Because annoying-to-map signature
    ("Value", "IsFloat32x4"), // Too experimental
    ("V8", "CreateSnapshotDataBlob"), // Because annoying-to-map signature
    ("V8", "WarmUpSnapshotDataBlob"), // Because annoying-to-map signature
    ("V8", "Initialize"), // V8::Initialize takes no context
    ("V8", "Dispose"), // V8::Dispose takes no context
    ("V8", "InitializePlatform"), // V8::InitializePlatform takes no context
    ("V8", "ShutdownPlatform"), // V8::ShutdownPlatform takes no context
];

/// Default mangle rules.
#[cfg_attr(rustfmt, rustfmt_skip)]
const METHOD_MANGLES: &'static [MethodMangle] = &[
    MethodMangle { name: "Set", signature: "index", mangle: "Set_Index"},
    MethodMangle { name: "Set", signature: "key", mangle: "Set_Key"},
    MethodMangle { name: "CreateDataProperty", signature: "index", mangle: "CreateDataProperty_Index"},
    MethodMangle { name: "CreateDataProperty", signature: "key", mangle: "CreateDataProperty_Key"},
    MethodMangle { name: "Get", signature: "index", mangle: "Get_Index"},
    MethodMangle { name: "Get", signature: "key", mangle: "Get_Key"},
    MethodMangle { name: "Has", signature: "index", mangle: "Has_Index"},
    MethodMangle { name: "Has", signature: "key", mangle: "Has_Key"},
    MethodMangle { name: "Delete", signature: "index", mangle: "Delete_Index"},
    MethodMangle { name: "Delete", signature: "key", mangle: "Delete_Key"},
    MethodMangle { name: "HasOwnProperty", signature: "index", mangle: "HasOwnProperty_Index"},
    MethodMangle { name: "HasOwnProperty", signature: "key", mangle: "HasOwnProperty_Key"},
    MethodMangle { name: "GetPropertyNames", signature: "mode", mangle: "GetPropertyNames_Filter"},
    MethodMangle { name: "GetOwnPropertyNames", signature: "filter", mangle: "GetOwnPropertyNames_Filter"},
    MethodMangle { name: "InitializeExternalStartupData", signature: "natives_blob", mangle: "InitializeExternalStartupData_Blobs"},
    MethodMangle { name: "InitializeExternalStartupData", signature: "directory_path", mangle: "InitializeExternalStartupData_Directory"},
    MethodMangle { name: "New", signature: "shared_array_buffer", mangle: "New_Shared"},
    MethodMangle { name: "New", signature: "array_buffer", mangle: "New_Owned"},
    MethodMangle { name: "New", signature: "mode", mangle: "New_Mode"},
    MethodMangle { name: "Set", signature: "isolate", mangle: "Set_Raw"},
    MethodMangle { name: "SetNativeDataProperty", signature: "v8::Name", mangle: "SetNativeDataProperty_Name"},
    MethodMangle { name: "SetAccessor", signature: "v8::Name", mangle: "SetAccessor_Name"},
    MethodMangle { name: "SetHandler", signature: "v8::Name", mangle: "SetHandler_Name"},
];

/// Reads the V8 API from the given file path pointing to a `v8.h`
/// file (or a file that includes `v8.h`), using the specified extra
/// includes if necessary.
///
/// # Panics
///
/// Since this library is supposed to be used in a build script,
/// panics if anything goes wrong whatsoever.
pub fn read<P1, P2>(file_path: P1, extra_includes: &[P2]) -> Api
    where P1: AsRef<path::Path>,
          P2: AsRef<path::Path>
{
    let clang = clang::Clang::new().unwrap();
    let index = clang::Index::new(&clang, false, true);

    let mut args = vec!["-x".to_owned(),
                        "c++".to_owned(),
                        "--std=c++11".to_owned(),
                        "-DV8_DEPRECATION_WARNINGS".to_owned(),
                        "-DV8_IMMINENT_DEPRECATION_WARNINGS".to_owned()];

    if cfg!(all(windows, target_env = "msvc")) {
        args.push("-fms-compatibility-version=19".to_owned());
    }

    if let Ok(target) = env::var("TARGET") {
        args.push("-target".to_owned());
        args.push(target.to_owned());
    }

    for include in extra_includes {
        println!("-I{:?}", include.as_ref());
        if let Some(include_str) = include.as_ref().to_str() {
            args.push(format!("-I{}", include_str));
        }
    }

    let translation_unit = index.parser(file_path.as_ref())
        .arguments(&args)
        .parse()
        .unwrap();

    build_api(&translation_unit.get_entity())
}

fn build_api(entity: &clang::Entity) -> Api {
    let namespaces = entity.get_children()
        .into_iter()
        .filter(|e| e.get_name().map(|n| n == "v8").unwrap_or(false));
    let classes = namespaces.flat_map(|n| build_classes(&n).into_iter()).collect();
    Api { classes: classes }
}

fn build_classes(entity: &clang::Entity) -> Vec<Class> {
    entity.get_children()
        .into_iter()
        // Is a class
        .filter(|e| e.get_kind() == clang::EntityKind::ClassDecl)
        // Is not just a declaration
        .filter(|e| !e.get_children().is_empty())
        // Is not nameless or special
        .filter(|e| {
            e.get_name().map(|ref n| !SPECIAL_CLASSES.contains(&n.as_str())).unwrap_or(false)
        })
        .map(|e| build_class(&e))
        .collect::<Vec<_>>()
}

fn build_class(entity: &clang::Entity) -> Class {
    let name = entity.get_name().unwrap();
    Class {
        methods: entity.get_children()
            .into_iter()
            // Is a method
            .filter(|e| e.get_kind() == clang::EntityKind::Method)
            // Is not deprecated
            .filter(|e| e.get_availability() == clang::Availability::Available)
            // Is public
            .filter(|e| e.get_accessibility() == Some(clang::Accessibility::Public))
            // Is not an operator or special
            .filter(|e| {
                e.get_name()
                    .map(|ref n| {
                        !n.starts_with("operator") &&
                            !SPECIAL_METHODS.iter()
                            .any(|m| m.0 == name &&  m.1 == n)
                    })
                    .unwrap_or(false)
            })
            .flat_map(|e| build_method(&e)
                      .map_err(|err| {
                          warn!("Could not translate method {}", e.get_display_name().unwrap_or_else(||"(unnamed)".to_owned()));
                          err
                      }))
            .collect(),
        name: name,
    }
}

fn build_method(entity: &clang::Entity) -> Result<Method, ()> {
    let display_name = try!(entity.get_display_name().ok_or(()));
    let name = try!(entity.get_name().ok_or(()));
    let args = try!(entity.get_arguments().ok_or(()));
    let args: Vec<Arg> = try!(args.iter().map(|e| build_arg(&e)).collect());

    let method_type = try!(entity.get_type().ok_or(()));
    let method_type_display_name = method_type.get_display_name();
    let ret_type = try!(method_type.get_result_type().ok_or(()));
    let ret_type = try!(build_ret_type(&ret_type));

    let mangled_name = METHOD_MANGLES.iter()
        .find(|m| {
            m.name == name &&
            (args.iter().any(|a| a.name == m.signature) || display_name.contains(m.signature) ||
             method_type_display_name.contains(m.signature))
        })
        .map(|m| m.mangle.to_owned());

    Ok(Method {
        is_static: entity.is_static_method(),
        mangled_name: mangled_name.unwrap_or_else(|| name.clone()),
        name: name,
        args: args,
        ret_type: ret_type,
    })
}

fn build_arg(entity: &clang::Entity) -> Result<Arg, ()> {
    Ok(Arg {
        name: try!(entity.get_name().ok_or(())),
        arg_type: try!(build_type(&entity.get_type().unwrap())),
    })
}

fn build_ret_type(typ: &clang::Type) -> Result<RetType, ()> {
    if typ.get_kind() == clang::TypeKind::Unexposed {
        let name = typ.get_display_name();

        if name.starts_with("MaybeLocal<") {
            let ref_type = try!(build_type(&get_first_tpl_arg(typ)));
            Ok(RetType::Maybe(Type::Ref(Box::new(ref_type))))
        } else if name.starts_with("Maybe<") {
            let ref_type = try!(build_type(&get_first_tpl_arg(typ)));
            Ok(RetType::Maybe(ref_type))
        } else {
            Ok(RetType::Direct(try!(build_type(typ))))
        }
    } else {
        Ok(RetType::Direct(try!(build_type(typ))))
    }
}

fn build_type(typ: &clang::Type) -> Result<Type, ()> {
    match typ.get_kind() {
        clang::TypeKind::Void => Ok(Type::Void),
        clang::TypeKind::Bool => Ok(Type::Bool),
        clang::TypeKind::CharS => {
            if typ.is_const_qualified() {
                Ok(Type::ConstChar)
            } else {
                Ok(Type::Char)
            }
        }
        clang::TypeKind::UInt => Ok(Type::UInt),
        clang::TypeKind::Int => Ok(Type::Int),
        clang::TypeKind::ULong => Ok(Type::ULong),
        clang::TypeKind::Long => Ok(Type::Long),
        clang::TypeKind::Double => Ok(Type::F64),
        clang::TypeKind::LongLong => Ok(Type::I64),
        clang::TypeKind::ULongLong => Ok(Type::U64),
        clang::TypeKind::Pointer => {
            let inner = try!(typ.get_pointee_type().ok_or(()));
            let inner = try!(build_type(&inner));
            Ok(Type::Ptr(Box::new(inner)))
        }
        clang::TypeKind::IncompleteArray => {
            let inner = try!(typ.get_element_type().ok_or(()));
            let inner = try!(build_type(&inner));
            Ok(Type::Arr(Box::new(inner)))
        }
        clang::TypeKind::Record => {
            // TODO: is this right?
            let name = typ.get_display_name().replace("v8::", "");
            if name.contains("::") {
                warn!("No support for nested type {:?}", name);
                Err(())
            } else {
                Ok(Type::Class(name))
            }
        }
        clang::TypeKind::Enum => {
            // TODO: is this right?
            let name = typ.get_display_name().replace("v8::", "");
            if name.contains("::") {
                warn!("No support for nested type {:?}", name);
                Err(())
            } else {
                Ok(Type::Enum(name))
            }
        }
        clang::TypeKind::Typedef => {
            // TODO: is this right?
            match typ.get_display_name().as_str() {
                "uint8_t" | "const uint8_t" => Ok(Type::U8),
                "int8_t" | "const int8_t" => Ok(Type::I8),
                "uint16_t" | "const uint16_t" => Ok(Type::U16),
                "int16_t" | "const int16_t" => Ok(Type::I16),
                "uint32_t" | "const uint32_t" => Ok(Type::U32),
                "int32_t" | "const int32_t" => Ok(Type::I32),
                "uint64_t" | "const uint64_t" => Ok(Type::U64),
                "int64_t" | "const int64_t" => Ok(Type::I64),
                "double" | "const double" => Ok(Type::F64),
                "size_t" | "const size_t" => Ok(Type::USize),
                s if s.ends_with("Callback") => Ok(Type::Callback(s.to_owned())),
                s => {
                    warn!("Unmapped type {:?} (a typedef)", s);
                    Err(())
                }
            }
        }
        clang::TypeKind::Unexposed => {
            if typ.get_display_name().starts_with("Local<") {
                let ref_type = try!(build_type(&get_first_tpl_arg(typ)));
                Ok(Type::Ref(Box::new(ref_type)))
            } else {
                match typ.get_display_name().as_str() {
                    // For some reason these fail to map
                    "v8::Isolate" => Ok(Type::Class("Isolate".to_owned())),
                    "v8::ObjectTemplate" => Ok(Type::Class("ObjectTemplate".to_owned())),
                    "v8::Value" => Ok(Type::Class("Value".to_owned())),
                    "v8::Local<v8::String>" => {
                        Ok(Type::Ref(Box::new(Type::Class("String".to_owned()))))
                    }
                    "v8::Local<v8::FunctionTemplate>" => {
                        Ok(Type::Ref(Box::new(Type::Class("FunctionTemplate".to_owned()))))
                    }
                    n => {
                        warn!("Unmapped type {:?} of kind {:?} (in unexposed exception table)",
                              n,
                              typ.get_kind());
                        Err(())
                    }
                }
            }
        }
        clang::TypeKind::LValueReference => {
            match typ.get_display_name().as_str() {
                "const v8::NamedPropertyHandlerConfiguration &" => {
                    Ok(Type::CallbackLValue("NamedPropertyHandlerConfiguration".to_owned()))
                }
                "const v8::IndexedPropertyHandlerConfiguration &" => {
                    Ok(Type::CallbackLValue("IndexedPropertyHandlerConfiguration".to_owned()))
                }
                n => {
                    warn!("Unmapped type {:?} of kind {:?} (in lvalue reference exception table)",
                          n,
                          typ.get_kind());
                    Err(())
                }
            }
        }
        _ => {
            warn!("Unmapped type {:?} of kind {:?} (in kind dispatch table)",
                  typ.get_display_name(),
                  typ.get_kind());
            Err(())
        }
    }
}

fn get_first_tpl_arg<'a>(typ: &clang::Type<'a>) -> clang::Type<'a> {
    let tpl_args = typ.get_template_argument_types().unwrap();
    tpl_args[0].unwrap()
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for class in self.classes.iter() {
            try!(writeln!(f, "{}", class));
        }
        Ok(())
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "class {}", self.name));
        for method in self.methods.iter() {
            try!(writeln!(f, "  {}", method));
        }
        Ok(())
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_static {
            try!(write!(f, "static "));
        }
        try!(write!(f, "{}(", self.name));

        let mut needs_sep = false;
        for arg in self.args.iter() {
            if needs_sep {
                try!(write!(f, ", "));
            }
            needs_sep = true;
            try!(write!(f, "{}", arg));
        }
        try!(write!(f, ") -> {}", self.ret_type));

        if self.mangled_name != self.name {
            try!(write!(f, " {{{}}}", self.mangled_name));
        }

        Ok(())
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{type} {name}", name=self.name, type=self.arg_type)
    }
}

impl fmt::Display for RetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RetType::Direct(ref t) => write!(f, "{}", t),
            RetType::Maybe(ref t) => write!(f, "maybe {}", t),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Type::Void => write!(f, "()"),
            Type::Bool => write!(f, "bool"),

            Type::Char => write!(f, "::std::os::raw::c_char"),
            Type::ConstChar => write!(f, "::std::os::raw::c_char"),
            Type::UInt => write!(f, "::std::os::raw::c_uint"),
            Type::Int => write!(f, "::std::os::raw::c_int"),
            Type::ULong => write!(f, "::std::os::raw::c_ulong"),
            Type::Long => write!(f, "::std::os::raw::c_long"),

            Type::U8 => write!(f, "u8"),
            Type::I8 => write!(f, "i8"),
            Type::U16 => write!(f, "u16"),
            Type::I16 => write!(f, "i16"),
            Type::U32 => write!(f, "u32"),
            Type::I32 => write!(f, "i32"),
            Type::U64 => write!(f, "u64"),
            Type::I64 => write!(f, "i64"),
            Type::F64 => write!(f, "f64"),
            Type::USize => write!(f, "usize"),
            Type::Enum(ref e) => write!(f, "enum {}", e),
            Type::Class(ref class) => write!(f, "class {}", class),
            Type::Callback(ref callback) => write!(f, "callback {}", callback),
            Type::CallbackLValue(ref v) => write!(f, "callback lvalue {}", v),
            Type::Ptr(ref target) => write!(f, "*mut {}", target),
            Type::Ref(ref target) => write!(f, "&{}", target),
            Type::Arr(ref target) => write!(f, "[{}]", target),
        }
    }
}
