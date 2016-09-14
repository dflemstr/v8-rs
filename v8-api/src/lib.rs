extern crate clang;
#[macro_use]
extern crate log;

use std::fmt;
use std::path;

#[derive(Debug)]
pub struct Api {
    pub classes: Vec<Class>,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug)]
pub struct Method {
    pub is_static: bool,
    pub name: String,
    pub mangled_name: String,
    pub args: Vec<Arg>,
    pub ret_type: RetType,
}

#[derive(Debug)]
pub enum RetType {
    Direct(Type),
    Maybe(Type),
}

#[derive(Debug)]
pub struct Arg {
    pub name: String,
    pub arg_type: Type,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Type {
    Void,
    Bool,

    Char,
    ConstChar,
    UInt,
    Int,
    ULong,
    Long,

    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F64,

    Class(String),

    Ref(Box<Type>),
    Ptr(Box<Type>),
    Arr(Box<Type>),
}

struct MethodMangle {
    name: &'static str,
    unique_arg: &'static str,
    mangle: &'static str,
}

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

    // v8-platform.h
    // These are all pre-requisites for creating an Isolate so
    // hand-roll them
    "Platform",
    "Task",
    "IdleTask",
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const SPECIAL_METHODS: &'static [&'static str] = &[
    "Compile", // Because ScriptOrigin param
    "GetScriptOrigin", // Because ScriptOrigin
    "SetAlignedPointerInInternalFields", // Because annoying-to-map signature
    "CallAsFunction", // Because annoying-to-map signature
    "CallAsConstructor", // Because annoying-to-map signature
    "NewInstance", // Because annoying-to-map signature
    "Call", // Because annoying-to-map signature
    "CreateSnapshotDataBlob", // Because annoying-to-map signature
    "WarmUpSnapshotDataBlob", // Because annoying-to-map signature
    "WriteUtf8", // Because annoying-to-map signature
    "Initialize", // V8::Initialize takes no context
    "Dispose", // V8::Dispose takes no context
    "InitializePlatform", // V8::InitializePlatform takes no context
    "ShutdownPlatform", // V8::ShutdownPlatform takes no context
];

#[cfg_attr(rustfmt, rustfmt_skip)]
const METHOD_MANGLES: &'static [MethodMangle] = &[
    MethodMangle { name: "Set", unique_arg: "index", mangle: "Set_Index"},
    MethodMangle { name: "Set", unique_arg: "key", mangle: "Set_Key"},
    MethodMangle { name: "CreateDataProperty", unique_arg: "index", mangle: "CreateDataProperty_Index"},
    MethodMangle { name: "CreateDataProperty", unique_arg: "key", mangle: "CreateDataProperty_Key"},
    MethodMangle { name: "Get", unique_arg: "index", mangle: "Get_Index"},
    MethodMangle { name: "Get", unique_arg: "key", mangle: "Get_Key"},
    MethodMangle { name: "Has", unique_arg: "index", mangle: "Has_Index"},
    MethodMangle { name: "Has", unique_arg: "key", mangle: "Has_Key"},
    MethodMangle { name: "Delete", unique_arg: "index", mangle: "Delete_Index"},
    MethodMangle { name: "Delete", unique_arg: "key", mangle: "Delete_Key"},
    MethodMangle { name: "HasOwnProperty", unique_arg: "index", mangle: "HasOwnProperty_Index"},
    MethodMangle { name: "HasOwnProperty", unique_arg: "key", mangle: "HasOwnProperty_Key"},
    MethodMangle { name: "InitializeExternalStartupData", unique_arg: "natives_blob", mangle: "InitializeExternalStartupData_Blobs"},
    MethodMangle { name: "InitializeExternalStartupData", unique_arg: "directory_path", mangle: "InitializeExternalStartupData_Directory"},
];

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

    for include in extra_includes {
        args.push(format!("-I{:?}", include.as_ref()));
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
        .filter(|e| e.get_kind() == clang::EntityKind::ClassDecl)
        .filter(|e| !e.get_children().is_empty())
        .filter(|e| {
            e.get_name().map(|ref n| !SPECIAL_CLASSES.contains(&n.as_str())).unwrap_or(false)
        })
        .map(|e| build_class(&e))
        .collect::<Vec<_>>()
}

fn build_class(entity: &clang::Entity) -> Class {
    Class {
        name: entity.get_name().unwrap(),
        methods: entity.get_children()
            .into_iter()
            .filter(|e| e.get_kind() == clang::EntityKind::Method)
            .filter(|e| e.get_availability() == clang::Availability::Available)
            .filter(|e| e.get_accessibility() == Some(clang::Accessibility::Public))
            .filter(|e| {
                e.get_name()
                    .map(|ref n| {
                        !n.starts_with("operator") && !SPECIAL_METHODS.contains(&n.as_str())
                    })
                    .unwrap_or(false)
            })
            .flat_map(|e| build_method(&e))
            .collect(),
    }
}

fn build_method(entity: &clang::Entity) -> Result<Method, ()> {
    let name = try!(entity.get_name().ok_or(()));
    let args = try!(entity.get_arguments().ok_or(()));
    let args: Vec<Arg> = try!(args.iter().map(|e| build_arg(&e)).collect());

    let ret_type = try!(try!(entity.get_type().ok_or(())).get_result_type().ok_or(()));
    let ret_type = try!(build_ret_type(&ret_type));

    let mangled_name = METHOD_MANGLES.iter()
        .find(|m| m.name == name && args.iter().any(|a| m.unique_arg == a.name))
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
                s => {
                    warn!("Unmapped type {:?}", s);
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
                    n => {
                        warn!("Unmapped type {:?} of kind {:?}", n, typ.get_kind());
                        Err(())
                    }
                }
            }
        }
        _ => {
            warn!("Unmapped type {:?} of kind {:?}",
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
            Type::Class(ref class) => write!(f, "class {}", class),
            Type::Ptr(ref target) => write!(f, "*mut {}", target),
            Type::Ref(ref target) => write!(f, "&{}", target),
            Type::Arr(ref target) => write!(f, "[{}]", target),
        }
    }
}
