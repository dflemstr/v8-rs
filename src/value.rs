//! Javascript values that user code can interact with.
use v8_sys;
use context;
use error;
use isolate;
use util;
use std::mem;
use std::ops;
use std::os;
use std::ptr;
use template;

/// The superclass of values and API object templates.
#[derive(Debug)]
pub struct Data(v8_sys::Data);

/// The superclass of all JavaScript values and objects.
#[derive(Debug)]
pub struct Value(v8_sys::Value);

/// The superclass of primitive values.  See ECMA-262 4.3.2.
#[derive(Debug)]
pub struct Primitive(v8_sys::Primitive);

/// A primitive boolean value (ECMA-262, 4.3.14).  Either the true or false value.
#[derive(Debug)]
pub struct Boolean(v8_sys::Boolean);

/// A superclass for symbols and strings.
#[derive(Debug)]
pub struct Name(v8_sys::Name);

/// A JavaScript string value (ECMA-262, 4.3.17).
#[derive(Debug)]
pub struct String(v8_sys::String);

/// A JavaScript symbol (ECMA-262 edition 6)
///
/// This is an experimental feature. Use at your own risk.
#[derive(Debug)]
pub struct Symbol(v8_sys::Symbol);

/// A private symbol
///
/// This is an experimental feature. Use at your own risk.
#[derive(Debug)]
pub struct Private(v8_sys::Private);

/// A JavaScript number value (ECMA-262, 4.3.20)
#[derive(Debug)]
pub struct Number(v8_sys::Number);

/// A JavaScript value representing a signed integer.
#[derive(Debug)]
pub struct Integer(v8_sys::Integer);

/// A JavaScript value representing a 32-bit signed integer.
#[derive(Debug)]
pub struct Int32(v8_sys::Int32);

/// A JavaScript value representing a 32-bit unsigned integer.
#[derive(Debug)]
pub struct Uint32(v8_sys::Uint32);

/// A JavaScript object (ECMA-262, 4.3.3)
#[derive(Debug)]
pub struct Object(v8_sys::Object);

/// An instance of the built-in array constructor (ECMA-262, 15.4.2).
#[derive(Debug)]
pub struct Array(v8_sys::Array);

/// An instance of the built-in Map constructor (ECMA-262, 6th Edition, 23.1.1).
#[derive(Debug)]
pub struct Map(v8_sys::Map);

/// An instance of the built-in Set constructor (ECMA-262, 6th Edition, 23.2.1).
#[derive(Debug)]
pub struct Set(v8_sys::Set);

/// A JavaScript function object (ECMA-262, 15.3).
#[derive(Debug)]
pub struct Function(v8_sys::Function);

/// An instance of the built-in Promise constructor (ES6 draft).
///
/// This API is experimental. Only works with --harmony flag.
#[derive(Debug)]
pub struct Promise(v8_sys::Promise);

/// An instance of the built-in Proxy constructor (ECMA-262, 6th Edition, 26.2.1).
#[derive(Debug)]
pub struct Proxy(v8_sys::Proxy);

/// An instance of the built-in ArrayBuffer constructor (ES6 draft 15.13.5).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct ArrayBuffer(v8_sys::ArrayBuffer);

/// A base class for an instance of one of "views" over ArrayBuffer, including TypedArrays and
/// DataView (ES6 draft 15.13).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct ArrayBufferView(v8_sys::ArrayBufferView);

/// A base class for an instance of TypedArray series of constructors (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct TypedArray(v8_sys::TypedArray);

/// An instance of Uint8Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Uint8Array(v8_sys::Uint8Array);

/// An instance of Uint8ClampedArray constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Uint8ClampedArray(v8_sys::Uint8ClampedArray);

/// An instance of Int8Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Int8Array(v8_sys::Int8Array);

/// An instance of Uint16Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Uint16Array(v8_sys::Uint16Array);

/// An instance of Int16Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Int16Array(v8_sys::Int16Array);

/// An instance of Uint32Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Uint32Array(v8_sys::Uint32Array);

/// An instance of Int32Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Int32Array(v8_sys::Int32Array);

/// An instance of Float32Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Float32Array(v8_sys::Float32Array);

/// An instance of Float64Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct Float64Array(v8_sys::Float64Array);

/// An instance of DataView constructor (ES6 draft 15.13.7).
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct DataView(v8_sys::DataView);

/// An instance of the built-in SharedArrayBuffer constructor.
///
/// This API is experimental and may change significantly.
#[derive(Debug)]
pub struct SharedArrayBuffer(v8_sys::SharedArrayBuffer);

/// An instance of the built-in Date constructor (ECMA-262, 15.9).
#[derive(Debug)]
pub struct Date(v8_sys::Date);

/// A Number object (ECMA-262, 4.3.21).
#[derive(Debug)]
pub struct NumberObject(v8_sys::NumberObject);

/// A Boolean object (ECMA-262, 4.3.15).
#[derive(Debug)]
pub struct BooleanObject(v8_sys::BooleanObject);

/// A String object (ECMA-262, 4.3.18).
#[derive(Debug)]
pub struct StringObject(v8_sys::StringObject);

/// A Symbol object (ECMA-262 edition 6).
///
/// This is an experimental feature. Use at your own risk.
#[derive(Debug)]
pub struct SymbolObject(v8_sys::SymbolObject);

/// An instance of the built-in RegExp constructor (ECMA-262, 15.10).
#[derive(Debug)]
pub struct RegExp(v8_sys::RegExp);

/// A JavaScript value that wraps an external value. This type of value is mainly used to associate
/// native data structures with JavaScript objects.
#[derive(Debug)]
pub struct External(v8_sys::External);

pub struct Exception(v8_sys::Exception);

pub struct PropertyCallbackInfo {
    pub this: Object,
    pub holder: Object,
}

pub struct FunctionCallbackInfo {
    pub isolate: isolate::Isolate,
    pub length: isize,
    pub args: Vec<Value>,
    pub this: Object,
    pub holder: Object,
    pub new_target: Value,
    pub is_construct_call: bool,
}

pub type FunctionCallback = Fn(FunctionCallbackInfo) -> Result<Value, Value> + 'static;

pub fn undefined(isolate: &isolate::Isolate) -> Primitive {
    let raw = unsafe { util::invoke(isolate, |c| v8_sys::v8_Undefined(c)).unwrap() };
    Primitive(isolate.clone(), raw)
}

pub fn null(isolate: &isolate::Isolate) -> Primitive {
    let raw = unsafe { util::invoke(isolate, |c| v8_sys::v8_Null(c)).unwrap() };
    Primitive(isolate.clone(), raw)
}

pub fn true_(isolate: &isolate::Isolate) -> Boolean {
    let raw = unsafe { util::invoke(isolate, |c| v8_sys::v8_True(c)).unwrap() };
    Boolean(isolate.clone(), raw)
}

pub fn false_(isolate: &isolate::Isolate) -> Boolean {
    let raw = unsafe { util::invoke(isolate, |c| v8_sys::v8_False(c)).unwrap() };
    Boolean(isolate.clone(), raw)
}

macro_rules! downcast {
    ($predicate:ident, $predicate_doc:expr, $wrapped:expr) => {
        #[doc=$predicate_doc]
        pub fn $predicate(&self) -> bool {
            unsafe { util::invoke(&self.0, |i| $wrapped(i, self.1)).map(|r|  r).unwrap_or(false) }
        }
    };
    ($predicate:ident, $predicate_doc:expr,
     $conversion:ident, $conversion_doc:expr,
     $wrapped:expr, $result:ident) => {
        downcast!($predicate, $predicate_doc, $wrapped);

        #[doc=$conversion_doc]
        pub fn $conversion(self) -> Option<$result> {
            if self.$predicate() {
                Some(unsafe { mem::transmute(self) })
            } else {
                None
            }
        }
    };
}

macro_rules! partial_conversion {
    ($name:ident, $wrapped:expr, $target:ident) => {
        pub fn $name(&self, context: &context::Context) -> $target {
            unsafe {
                util::invoke_ctx(&self.0, context, |i| $wrapped(i, self.1, context.as_raw()))
                    .map(|p| $target(self.0.clone(), p))
                    .unwrap()
            }
        }
    }
}

macro_rules! partial_get {
    ($name:ident, $wrapped:expr, $target:ident) => {
        pub fn $name(&self, context: &context::Context) -> $target {
            unsafe {
                let maybe = util::invoke_ctx(&self.0, context, |c| $wrapped(c, self.1, context.as_raw())).unwrap();
                assert!( maybe.is_set);

                maybe.value
            }
        }
    }
}

impl Data {
    /// Creates a data from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::DataRef) -> Data {
        Data(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this primitive.
    pub fn as_raw(&self) -> v8_sys::DataRef {
        self.1
    }
}

impl Value {
    downcast!(is_undefined,
              "Returns true if this value is the undefined value.  See ECMA-262 4.3.10.",
              v8_sys::v8_Value_IsUndefined);
    downcast!(is_null,
              "Returns true if this value is the null value.  See ECMA-262 4.3.11.",
              v8_sys::v8_Value_IsNull);
    downcast!(is_true,
              "Returns true if this value is true.",
              v8_sys::v8_Value_IsTrue);
    downcast!(is_false,
              "Returns true if this value is false.",
              v8_sys::v8_Value_IsFalse);
    downcast!(is_name,
              "Returns true if this value is a symbol or a string.\n\nThis is an experimental \
               feature.",
              into_name,
              "",
              v8_sys::v8_Value_IsName,
              Name);
    downcast!(is_string,
              "Returns true if this value is an instance of the String type.  See ECMA-262 8.4.",
              into_string,
              "",
              v8_sys::v8_Value_IsString,
              String);
    downcast!(is_symbol,
              "Returns true if this value is a symbol.\n\nThis is an experimental feature.",
              into_symbol,
              "",
              v8_sys::v8_Value_IsSymbol,
              Symbol);
    downcast!(is_function,
              "Returns true if this value is a function.",
              into_function,
              "",
              v8_sys::v8_Value_IsFunction,
              Function);
    downcast!(is_array,
              "Returns true if this value is an array.  Note that it will return false for an \
               Proxy for an array.",
              into_array,
              "",
              v8_sys::v8_Value_IsArray,
              Array);
    downcast!(is_object,
              "Returns true if this value is an object.",
              into_object,
              "",
              v8_sys::v8_Value_IsObject,
              Object);
    downcast!(is_boolean,
              "Returns true if this value is boolean.",
              into_boolean,
              "",
              v8_sys::v8_Value_IsBoolean,
              Boolean);
    downcast!(is_number,
              "Returns true if this value is a number.",
              into_number,
              "",
              v8_sys::v8_Value_IsNumber,
              Number);
    downcast!(is_external,
              "Returns true if this value is external.",
              into_external,
              "",
              v8_sys::v8_Value_IsExternal,
              External);
    downcast!(is_int32,
              "Returns true if this value is a 32-bit signed integer.",
              into_int32,
              "",
              v8_sys::v8_Value_IsInt32,
              Int32);
    downcast!(is_uint32,
              "Returns true if this value is a 32-bit unsigned integer.",
              into_uint32,
              "",
              v8_sys::v8_Value_IsUint32,
              Uint32);
    downcast!(is_date,
              "Returns true if this value is a Date.",
              into_date,
              "",
              v8_sys::v8_Value_IsDate,
              Date);
    downcast!(is_arguments_object,
              "Returns true if this value is an Arguments object.",
              v8_sys::v8_Value_IsArgumentsObject);
    downcast!(is_boolean_object,
              "Returns true if this value is a Boolean object.",
              into_boolean_object,
              "",
              v8_sys::v8_Value_IsBooleanObject,
              BooleanObject);
    downcast!(is_number_object,
              "Returns true if this value is a Number object.",
              into_number_object,
              "",
              v8_sys::v8_Value_IsNumberObject,
              NumberObject);
    downcast!(is_string_object,
              "Returns true if this value is a String object.",
              into_string_object,
              "",
              v8_sys::v8_Value_IsStringObject,
              StringObject);
    downcast!(is_symbol_object,
              "Returns true if this value is a Symbol object.\n\nThis is an experimental feature.",
              into_symbol_object,
              "",
              v8_sys::v8_Value_IsSymbolObject,
              Symbol);
    downcast!(is_native_error,
              "Returns true if this value is a NativeError.",
              v8_sys::v8_Value_IsNativeError);
    downcast!(is_reg_exp,
              "Returns true if this value is a RegExp.",
              into_reg_exp,
              "",
              v8_sys::v8_Value_IsRegExp,
              RegExp);
    downcast!(is_generator_function,
              "Returns true if this value is a Generator function.\n\nThis is an experimental \
               feature.",
              v8_sys::v8_Value_IsGeneratorFunction);
    downcast!(is_generator_object,
              "Returns true if this value is a Generator object (iterator).\n\nThis is an \
               experimental feature.",
              v8_sys::v8_Value_IsGeneratorObject);
    downcast!(is_promise,
              "Returns true if this value is a Promise.\n\nThis is an experimental feature.",
              into_promise,
              "",
              v8_sys::v8_Value_IsPromise,
              Promise);
    downcast!(is_map,
              "Returns true if this value is a Map.",
              into_map,
              "",
              v8_sys::v8_Value_IsMap,
              Map);
    downcast!(is_set,
              "Returns true if this value is a Set.",
              into_set,
              "",
              v8_sys::v8_Value_IsSet,
              Set);
    downcast!(is_map_iterator,
              "Returns true if this value is a Map Iterator.",
              v8_sys::v8_Value_IsMapIterator);
    downcast!(is_set_iterator,
              "Returns true if this value is a Set Iterator.",
              v8_sys::v8_Value_IsSetIterator);
    downcast!(is_weak_map,
              "Returns true if this value is a WeakMap.",
              v8_sys::v8_Value_IsWeakMap);
    downcast!(is_weak_set,
              "Returns true if this value is a WeakSet.",
              v8_sys::v8_Value_IsWeakSet);
    downcast!(is_array_buffer,
              "Returns true if this value is an ArrayBuffer.\n\nThis is an experimental feature.",
              into_array_buffer,
              "",
              v8_sys::v8_Value_IsArrayBuffer,
              ArrayBuffer);
    downcast!(is_array_buffer_view,
              "Returns true if this value is an ArrayBufferView.\n\nThis is an experimental \
               feature.",
              into_array_buffer_view,
              "",
              v8_sys::v8_Value_IsArrayBufferView,
              ArrayBufferView);
    downcast!(is_typed_array,
              "Returns true if this value is one of TypedArrays.\n\nThis is an experimental \
               feature.",
              into_typed_array,
              "",
              v8_sys::v8_Value_IsTypedArray,
              TypedArray);
    downcast!(is_uint8_array,
              "Returns true if this value is an Uint8Array.\n\nThis is an experimental feature.",
              into_uint8_array,
              "",
              v8_sys::v8_Value_IsUint8Array,
              Uint8Array);
    downcast!(is_uint8_clamped_array,
              "Returns true if this value is an Uint8ClampedArray.\n\nThis is an experimental \
               feature.",
              into_uint8_clamped_array,
              "",
              v8_sys::v8_Value_IsUint8ClampedArray,
              Uint8ClampedArray);
    downcast!(is_int8_array,
              "Returns true if this value is an Int8Array.\n\nThis is an experimental feature.",
              into_int8_array,
              "",
              v8_sys::v8_Value_IsInt8Array,
              Int8Array);
    downcast!(is_uint16_array,
              "Returns true if this value is an Uint16Array.\n\nThis is an experimental feature.",
              into_uint16_array,
              "",
              v8_sys::v8_Value_IsUint16Array,
              Uint16Array);
    downcast!(is_int16_array,
              "Returns true if this value is an Int16Array.\n\nThis is an experimental feature.",
              into_int16_array,
              "",
              v8_sys::v8_Value_IsInt16Array,
              Int16Array);
    downcast!(is_uint32_array,
              "Returns true if this value is an Uint32Array.\n\nThis is an experimental feature.",
              into_uint32_array,
              "",
              v8_sys::v8_Value_IsUint32Array,
              Uint32Array);
    downcast!(is_int32_array,
              "Returns true if this value is an Int32Array.\n\nThis is an experimental feature.",
              into_int32_array,
              "",
              v8_sys::v8_Value_IsInt32Array,
              Int32Array);
    downcast!(is_float32_array,
              "Returns true if this value is a Float32Array.\n\nThis is an experimental feature.",
              into_float32_array,
              "",
              v8_sys::v8_Value_IsFloat32Array,
              Float32Array);
    downcast!(is_float64_array,
              "Returns true if this value is a Float64Array.\n\nThis is an experimental feature.",
              into_float64_array,
              "",
              v8_sys::v8_Value_IsFloat64Array,
              Float64Array);
    downcast!(is_data_view,
              "Returns true if this value is a DataView.\n\nThis is an experimental feature.",
              into_data_view,
              "",
              v8_sys::v8_Value_IsDataView,
              DataView);
    downcast!(is_shared_array_buffer,
              "Returns true if this value is a SharedArrayBuffer.\n\nThis is an experimental \
               feature.",
              into_shared_array_buffer,
              "",
              v8_sys::v8_Value_IsSharedArrayBuffer,
              SharedArrayBuffer);
    downcast!(is_proxy,
              "Returns true if this value is a JavaScript Proxy.",
              into_proxy,
              "",
              v8_sys::v8_Value_IsProxy,
              Proxy);

    partial_conversion!(to_boolean, v8_sys::v8_Value_ToBoolean, Boolean);
    partial_conversion!(to_number, v8_sys::v8_Value_ToNumber, Number);
    partial_conversion!(to_string, v8_sys::v8_Value_ToString, String);
    partial_conversion!(to_detail_string, v8_sys::v8_Value_ToDetailString, String);
    partial_conversion!(to_object, v8_sys::v8_Value_ToObject, Object);
    partial_conversion!(to_integer, v8_sys::v8_Value_ToInteger, Integer);
    partial_conversion!(to_uint32, v8_sys::v8_Value_ToUint32, Uint32);
    partial_conversion!(to_int32, v8_sys::v8_Value_ToInt32, Int32);
    partial_conversion!(to_array_index, v8_sys::v8_Value_ToArrayIndex, Uint32);

    pub fn boolean_value(&self, context: &context::Context) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0,
                                     context,
                                     |i| v8_sys::v8_Value_BooleanValue(i, self.1, context.as_raw()))
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    partial_get!(number_value, v8_sys::v8_Value_NumberValue, f64);
    partial_get!(integer_value, v8_sys::v8_Value_IntegerValue, i64);
    partial_get!(uint32_value, v8_sys::v8_Value_Uint32Value, u32);
    partial_get!(int32_value, v8_sys::v8_Value_Int32Value, i32);

    pub fn equals(&self, context: &context::Context, that: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Value_Equals(c, self.1, context.as_raw(), that.as_raw())
                })
                .unwrap();
            assert!( m.is_set);

             m.value
        }
    }

    pub fn strict_equals(&self, that: &Value) -> bool {
        unsafe {
            util::invoke(&self.0, |c| v8_sys::v8_Value_StrictEquals(c, self.1, that.as_raw())).unwrap()
        }
    }

    pub fn same_value(&self, that: &Value) -> bool {
        unsafe {
            util::invoke(&self.0, |c| v8_sys::v8_Value_SameValue(c, self.1, that.as_raw())).unwrap()
        }
    }

    /// Creates a value from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::ValueRef) -> Value {
        Value(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this value.
    pub fn as_raw(&self) -> v8_sys::ValueRef {
        self.1
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        self.strict_equals(other)
    }
}

impl Primitive {
    /// Creates a primitive from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::PrimitiveRef) -> Primitive {
        Primitive(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this primitive.
    pub fn as_raw(&self) -> v8_sys::PrimitiveRef {
        self.1
    }
}

impl Boolean {
    pub fn new(isolate: &isolate::Isolate, value: bool) -> Boolean {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Boolean_New(c, isolate.as_raw(), value)).unwrap()
        };
        Boolean(isolate.clone(), raw)
    }

    pub fn value(&self) -> bool {
        unsafe {  util::invoke(&self.0, |c| v8_sys::v8_Boolean_Value(c, self.1)).unwrap() }
    }

    /// Creates a boolean from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::BooleanRef) -> Boolean {
        Boolean(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this boolean.
    pub fn as_raw(&self) -> v8_sys::BooleanRef {
        self.1
    }
}

impl Name {
    /// Returns the identity hash for this object.
    ///
    /// The current implementation uses an inline property on the object to store the identity
    /// hash.
    ///
    /// The return value will never be 0.  Also, it is not guaranteed
    /// to be unique.
    pub fn get_identity_hash(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Name_GetIdentityHash(c, self.1)).unwrap() as u32 }
    }

    /// Creates a name from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::NameRef) -> Name {
        Name(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this primitive.
    pub fn as_raw(&self) -> v8_sys::NameRef {
        self.1
    }
}

impl String {
    pub fn empty(isolate: &isolate::Isolate) -> String {
        let raw =
            unsafe { util::invoke(&isolate, |c| v8_sys::v8_String_Empty(c, isolate.as_raw())).unwrap() };
        String(isolate.clone(), raw)
    }

    /// Allocates a new string from UTF-8 data.
    pub fn from_str(isolate: &isolate::Isolate, str: &str) -> String {
        let raw = unsafe {
            let ptr = mem::transmute(str.as_ptr());
            let len = str.len() as os::raw::c_int;
            util::invoke(&isolate, |c| v8_sys::v8_String_NewFromUtf8_Normal(c, ptr, len)).unwrap()
        };
        String(isolate.clone(), raw)
    }

    /// Allocates a new internalized string from UTF-8 data.
    pub fn internalized_from_str(isolate: &isolate::Isolate, str: &str) -> String {
        unsafe {
            let ptr = mem::transmute(str.as_ptr());
            let len = str.len() as os::raw::c_int;
            let raw = util::invoke(&isolate,
                                   |c| v8_sys::v8_String_NewFromUtf8_Internalized(c, ptr, len))
                .unwrap();
            String(isolate.clone(), raw)
        }
    }

    /// Returns the number of characters in this string.
    pub fn length(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_String_Length(c, self.1)).unwrap() as u32 }
    }

    /// Returns the number of bytes in the UTF-8 encoded representation of this string.
    pub fn utf8_length(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_String_Utf8Length(c, self.1)).unwrap() as u32 }
    }

    /// Returns whether this string is known to contain only one byte data.
    ///
    /// Does not read the string.
    ///
    /// False negatives are possible.
    pub fn is_one_byte(&self) -> bool {
        unsafe {  util::invoke(&self.0, |c| v8_sys::v8_String_IsOneByte(c, self.1)).unwrap() }
    }

    /// Returns whether this string contain only one byte data.
    ///
    /// Will read the entire string in some cases.
    pub fn contains_only_one_byte(&self) -> bool {
        unsafe {
             util::invoke(&self.0, |c| v8_sys::v8_String_ContainsOnlyOneByte(c, self.1)).unwrap()
        }
    }

    pub fn value(&self) -> ::std::string::String {
        let len = unsafe {
            util::invoke(&self.0, |c| v8_sys::v8_String_Utf8Length(c, self.1)).unwrap()
        } as usize;
        let mut buf = vec![0u8; len];

        unsafe {
            let ptr = mem::transmute(buf.as_mut_ptr());
            util::invoke(&self.0, |c| {
                    v8_sys::v8_String_WriteUtf8(c, self.1, ptr, len as i32)
                })
                .unwrap();
            ::std::string::String::from_utf8_unchecked(buf)
        }
    }

    /// Creates a string from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::StringRef) -> String {
        String(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this string.
    pub fn as_raw(&self) -> v8_sys::StringRef {
        self.1
    }
}

impl Symbol {
    /// Access global symbol registry.
    ///
    /// Note that symbols created this way are never collected, so they should only be used for
    /// statically fixed properties.  Also, there is only one global name space for the names used
    /// as keys.  To minimize the potential for clashes, use qualified names as keys.
    pub fn for_name(isolate: &isolate::Isolate, name: &String) -> Symbol {
        let raw = unsafe {
            util::invoke(&isolate,
                         |c| v8_sys::v8_Symbol_For(c, isolate.as_raw(), name.as_raw()))
                .unwrap()
        };
        Symbol(isolate.clone(), raw)
    }

    /// Retrieve a global symbol.
    ///
    /// Similar to `for_name`, but using a separate registry that is not accessible by (and cannot
    /// clash with) JavaScript code.
    pub fn for_api_name(isolate: &isolate::Isolate, name: &String) -> Symbol {
        let raw = unsafe {
            util::invoke(&isolate,
                         |c| v8_sys::v8_Symbol_ForApi(c, isolate.as_raw(), name.as_raw()))
                .unwrap()
        };
        Symbol(isolate.clone(), raw)
    }

    /// Well-known symbol `Symbol.iterator`.
    pub fn get_iterator(isolate: &isolate::Isolate) -> Symbol {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Symbol_GetIterator(c, isolate.as_raw())).unwrap()
        };
        Symbol(isolate.clone(), raw)
    }

    /// Well-known symbol `Symbol.unscopables`.
    pub fn get_unscopables(isolate: &isolate::Isolate) -> Symbol {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Symbol_GetUnscopables(c, isolate.as_raw())).unwrap()
        };
        Symbol(isolate.clone(), raw)
    }

    /// Well-known symbol `Symbol.toStringTag`.
    pub fn get_to_string_tag(isolate: &isolate::Isolate) -> Symbol {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Symbol_GetToStringTag(c, isolate.as_raw())).unwrap()
        };
        Symbol(isolate.clone(), raw)
    }

    /// Well-known symbol `Symbol.isConcatSpreadable`.
    pub fn get_is_concat_spreadable(isolate: &isolate::Isolate) -> Symbol {
        let raw = unsafe {
            util::invoke(&isolate,
                         |c| v8_sys::v8_Symbol_GetIsConcatSpreadable(c, isolate.as_raw()))
                .unwrap()
        };
        Symbol(isolate.clone(), raw)
    }

    /// Creates a symbol from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::SymbolRef) -> Symbol {
        Symbol(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this symbol.
    pub fn as_raw(&self) -> v8_sys::SymbolRef {
        self.1
    }
}

impl Private {
    /// Create a private symbol.
    ///
    /// If name is not empty, it will be the description.
    pub fn new(isolate: &isolate::Isolate, name: &String) -> Private {
        let raw = unsafe {
            util::invoke(&isolate,
                         |c| v8_sys::v8_Private_New(c, isolate.as_raw(), name.as_raw()))
                .unwrap()
        };
        Private(isolate.clone(), raw)
    }

    /// Retrieve a global private symbol.
    ///
    /// If a symbol with this name has not been retrieved in the same isolate before, it is
    /// created.  Note that private symbols created this way are never collected, so they should
    /// only be used for statically fixed properties.  Also, there is only one global name space
    /// for the names used as keys.  To minimize the potential for clashes, use qualified names as
    /// keys, e.g., "Class#property".
    pub fn for_api_name(isolate: &isolate::Isolate, name: &String) -> Private {
        let raw = unsafe {
            util::invoke(&isolate,
                         |c| v8_sys::v8_Private_ForApi(c, isolate.as_raw(), name.as_raw()))
                .unwrap()
        };
        Private(isolate.clone(), raw)
    }

    /// Creates a private from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::PrivateRef) -> Private {
        Private(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this private.
    pub fn as_raw(&self) -> v8_sys::PrivateRef {
        self.1
    }
}

impl Number {
    pub fn new(isolate: &isolate::Isolate, value: f64) -> Number {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Number_New(c, isolate.as_raw(), value)).unwrap()
        };
        Number(isolate.clone(), raw)
    }

    pub fn value(&self) -> f64 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Number_Value(c, self.1)).unwrap() }
    }

    /// Creates a number from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::NumberRef) -> Number {
        Number(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this number.
    pub fn as_raw(&self) -> v8_sys::NumberRef {
        self.1
    }
}

impl Integer {
    pub fn new(isolate: &isolate::Isolate, value: i32) -> Integer {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Integer_New(c, isolate.as_raw(), value)).unwrap()
        };
        Integer(isolate.clone(), raw)
    }

    pub fn new_from_unsigned(isolate: &isolate::Isolate, value: u32) -> Integer {
        let raw = unsafe {
            util::invoke(&isolate,
                         |c| v8_sys::v8_Integer_NewFromUnsigned(c, isolate.as_raw(), value))
                .unwrap()
        };
        Integer(isolate.clone(), raw)
    }

    pub fn value(&self) -> i64 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Integer_Value(c, self.1)).unwrap() }
    }

    /// Creates an integer from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::IntegerRef) -> Integer {
        Integer(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this integer.
    pub fn as_raw(&self) -> v8_sys::IntegerRef {
        self.1
    }
}

impl Int32 {
    pub fn value(&self) -> i32 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Int32_Value(c, self.1)).unwrap() }
    }

    /// Creates a 32-bit integer from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::Int32Ref) -> Int32 {
        Int32(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this 32-bit integer.
    pub fn as_raw(&self) -> v8_sys::Int32Ref {
        self.1
    }
}

impl Uint32 {
    pub fn value(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Uint32_Value(c, self.1)).unwrap() }
    }

    /// Creates a 32-bit unsigned integer from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::Uint32Ref) -> Uint32 {
        Uint32(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this 32-bit unsigned integer.
    pub fn as_raw(&self) -> v8_sys::Uint32Ref {
        self.1
    }
}

impl Object {
    pub fn new(isolate: &isolate::Isolate, context: &context::Context) -> Object {
        let _g = context.make_current();
        let raw = unsafe {
            util::invoke_ctx(&isolate, context, |c| v8_sys::v8_Object_New(c, isolate.as_raw())).unwrap()
        };
        Object(isolate.clone(), raw)
    }

    pub fn set(&self, context: &context::Context, key: &Value, value: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_Set_Key(c, self.1, context.as_raw(), key.as_raw(), value.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn set_index(&self, context: &context::Context, index: u32, value: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_Set_Index(c, self.1, context.as_raw(), index, value.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn create_data_property(&self,
                                context: &context::Context,
                                key: &Name,
                                value: &Value)
                                -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_CreateDataProperty_Key(c,
                                                      self.1,
                                                      context.as_raw(),
                                                      key.as_raw(),
                                                      value.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn create_data_property_index(&self,
                                      context: &context::Context,
                                      index: u32,
                                      value: &Value)
                                      -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_CreateDataProperty_Index(c,
                                                        self.1,
                                                        context.as_raw(),
                                                        index,
                                                        value.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn get(&self, context: &context::Context, key: &Value) -> Value {
        unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Object_Get_Key(c, self.1, context.as_raw(), key.as_raw()))
                .map(|p| Value(self.0.clone(), p))
                .unwrap()
        }
    }

    pub fn get_index(&self, context: &context::Context, index: u32) -> Value {
        let raw = unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Object_Get_Index(c, self.1, context.as_raw(), index))
                .unwrap()
        };
        Value(self.0.clone(), raw)
    }

    pub fn delete(&self, context: &context::Context, key: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_Delete_Key(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn delete_index(&self, context: &context::Context, index: u32) -> bool {
        unsafe {
            let m =
                util::invoke_ctx(&self.0,
                                 context,
                                 |c| v8_sys::v8_Object_Delete_Index(c, self.1, context.as_raw(), index))
                    .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn has(&self, context: &context::Context, key: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_Has_Key(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn has_index(&self, context: &context::Context, index: u32) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0,
                                     context,
                                     |c| v8_sys::v8_Object_Has_Index(c, self.1, context.as_raw(), index))
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    /// Returns an array containing the names of the enumerable properties of this object,
    /// including properties from prototype objects.
    ///
    /// The array returned by this method contains the same values as would be enumerated by a
    /// for-in statement over this object.
    pub fn get_property_names(&self, context: &context::Context) -> Array {
        unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Object_GetPropertyNames(c, self.1, context.as_raw()))
                .map(|p| Array(self.0.clone(), p))
                .unwrap()
        }
    }

    /// This function has the same functionality as `get_property_names` but the returned array
    /// doesn't contain the names of properties from prototype objects.
    pub fn get_own_property_names(&self, context: &context::Context) -> Array {
        unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Object_GetOwnPropertyNames(c, self.1, context.as_raw()))
                .map(|p| Array(self.0.clone(), p))
                .unwrap()
        }
    }


    pub fn set_private(&self, context: &context::Context, key: &Private, value: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_SetPrivate(c, self.1, context.as_raw(), key.as_raw(), value.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn get_private(&self, context: &context::Context, key: &Private) -> Value {
        unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Object_GetPrivate(c, self.1, context.as_raw(), key.as_raw()))
                .map(|p| Value(self.0.clone(), p))
                .unwrap()
        }
    }

    pub fn delete_private(&self, context: &context::Context, key: &Private) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_DeletePrivate(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn has_private(&self, context: &context::Context, key: &Private) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_HasPrivate(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    /// Get the prototype object.
    ///
    /// This does not skip objects marked to be skipped by `__proto__` and it does not consult the
    /// security handler.
    pub fn get_prototype(&self) -> Value {
        let raw = unsafe { util::invoke(&self.0, |c| v8_sys::v8_Object_GetPrototype(c, self.1)).unwrap() };
        Value(self.0.clone(), raw)
    }

    /// Set the prototype object.
    ///
    /// This does not skip objects marked to be skipped by `__proto__` and it does not consult the
    /// security handler.
    pub fn set_prototype(&self, context: &context::Context, prototype: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_SetPrototype(c, self.1, context.as_raw(), prototype.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    /// Call builtin Object.prototype.toString on this object.
    ///
    /// This is different from `Value::to_string` that may call user-defined `toString`
    /// function. This one does not.
    pub fn object_proto_to_string(&self, context: &context::Context) -> String {
        let raw = unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Object_ObjectProtoToString(c, self.1, context.as_raw()))
                .unwrap()
        };
        String(self.0.clone(), raw)
    }

    /// Returns the name of the function invoked as a constructor for this object.
    pub fn get_constructor_name(&self) -> String {
        let raw =
            unsafe { util::invoke(&self.0, |c| v8_sys::v8_Object_GetConstructorName(c, self.1)).unwrap() };

        String(self.0.clone(), raw)
    }

    /// Gets the number of internal fields for this Object
    pub fn internal_field_count(&self) -> u32 {
        unsafe {
            util::invoke(&self.0, |c| v8_sys::v8_Object_InternalFieldCount(c, self.1)).unwrap() as u32
        }
    }

    /// Gets the value from an internal field.
    pub unsafe fn get_internal_field(&self, index: u32) -> Value {
        let raw = util::invoke(&self.0,
                               |c| v8_sys::v8_Object_GetInternalField(c, self.1, index as os::raw::c_int))
            .unwrap();
        Value(self.0.clone(), raw)
    }

    /// Sets the value in an internal field.
    pub unsafe fn set_internal_field(&self, index: u32, value: &Value) {
        util::invoke(&self.0, |c| {
                v8_sys::v8_Object_SetInternalField(c, self.1, index as os::raw::c_int, value.as_raw())
            })
            .unwrap()
    }

    /// Gets a 2-byte-aligned native pointer from an internal field.
    ///
    /// This field must have been set by `set_aligned_pointer_in_internal_field`, everything else
    /// leads to undefined behavior.
    pub unsafe fn get_aligned_pointer_from_internal_field<A>(&self, index: u32) -> *mut A {
        util::invoke(&self.0, |c| {
                v8_sys::v8_Object_GetAlignedPointerFromInternalField(c, self.1, index as os::raw::c_int)
            })
            .unwrap() as *mut A
    }

    /// Sets a 2-byte-aligned native pointer in an internal field.
    ///
    /// To retrieve such a field, `get_aligned_pointer_from_internal_field` must be used, everything
    /// else leads to undefined behavior.
    pub unsafe fn set_aligned_pointer_in_internal_field<A>(&self, index: u32, value: *mut A) {
        util::invoke(&self.0, |c| {
                v8_sys::v8_Object_SetAlignedPointerInInternalField(c,
                                                            self.1,
                                                            index as os::raw::c_int,
                                                            value as *mut os::raw::c_void)
            })
            .unwrap()
    }

    pub fn has_own_property(&self, context: &context::Context, key: &Name) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_HasOwnProperty_Key(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn has_own_property_index(&self, context: &context::Context, index: u32) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_HasOwnProperty_Index(c, self.1, context.as_raw(), index)
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn has_real_named_property(&self, context: &context::Context, key: &Name) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_HasRealNamedProperty(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn has_real_indexed_property(&self, context: &context::Context, index: u32) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Object_HasRealIndexedProperty(c, self.1, context.as_raw(), index)
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    /// Returns the identity hash for this object.
    ///
    /// The current implementation uses a hidden property on the object to store the identity hash.
    ///
    /// The return value will never be 0. Also, it is not guaranteed to be unique.
    pub fn get_identity_hash(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Object_GetIdentityHash(c, self.1)).unwrap() as u32 }
    }

    /// Clone this object with a fast but shallow copy.
    ///
    /// Values will point to the same values as the original object.
    pub fn clone_object(&self) -> Object {
        let raw = unsafe { util::invoke(&self.0, |c| v8_sys::v8_Object_Clone(c, self.1)).unwrap() };

        Object(self.0.clone(), raw)
    }

    pub fn creation_context(&self) -> context::Context {
        unsafe {
            let raw = util::invoke(&self.0, |c| v8_sys::v8_Object_CreationContext(c, self.1)).unwrap();
            context::Context::from_raw(&self.0, raw)
        }
    }

    /// Checks whether a callback is set by the ObjectTemplate::SetCallAsFunctionHandler method.
    ///
    /// When an Object is callable this method returns true.
    pub fn is_callable(&self) -> bool {
        unsafe {  util::invoke(&self.0, |c| v8_sys::v8_Object_IsCallable(c, self.1)).unwrap() }
    }

    /// True if this object is a constructor.
    pub fn is_constructor(&self) -> bool {
        unsafe {  util::invoke(&self.0, |c| v8_sys::v8_Object_IsConstructor(c, self.1)).unwrap() }
    }

    /// Call an Object as a function if a callback is set by the
    /// ObjectTemplate::SetCallAsFunctionHandler method.
    pub fn call(&self, context: &context::Context, args: &[&Value]) -> error::Result<Value> {
        let mut arg_ptrs = args.iter().map(|v| v.1).collect::<Vec<_>>();
        let raw = unsafe {
            try!(util::invoke_ctx(&self.0, context, |c| {
                v8_sys::v8_Object_CallAsFunction(c,
                                          self.1,
                                          context.as_raw(),
                                          ptr::null_mut(),
                                          arg_ptrs.len() as i32,
                                          arg_ptrs.as_mut_ptr())
            }))
        };
        Ok(Value(self.0.clone(), raw))
    }

    /// Call an Object as a function if a callback is set by the
    /// ObjectTemplate::SetCallAsFunctionHandler method.
    pub fn call_with_this(&self,
                          context: &context::Context,
                          this: &Value,
                          args: &[&Value])
                          -> error::Result<Value> {
        let mut arg_ptrs = args.iter().map(|v| v.1).collect::<Vec<_>>();
        let raw = unsafe {
            try!(util::invoke_ctx(&self.0, context, |c| {
                v8_sys::v8_Object_CallAsFunction(c,
                                          self.1,
                                          context.as_raw(),
                                          this.as_raw(),
                                          arg_ptrs.len() as i32,
                                          arg_ptrs.as_mut_ptr())
            }))
        };
        Ok(Value(self.0.clone(), raw))
    }

    /// Call an Object as a constructor if a callback is set by the
    /// ObjectTemplate::SetCallAsFunctionHandler method.
    ///
    /// Note: This method behaves like the Function::NewInstance method.
    pub fn call_as_constructor(&self,
                               context: &context::Context,
                               args: &[&Value])
                               -> error::Result<Value> {
        let mut arg_ptrs = args.iter().map(|v| v.1).collect::<Vec<_>>();
        let raw = unsafe {
            try!(util::invoke_ctx(&self.0, context, |c| {
                v8_sys::v8_Object_CallAsConstructor(c,
                                             self.1,
                                             context.as_raw(),
                                             arg_ptrs.len() as i32,
                                             arg_ptrs.as_mut_ptr())
            }))
        };
        Ok(Value(self.0.clone(), raw))
    }

    /// Creates an object from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::ObjectRef) -> Object {
        Object(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this object.
    pub fn as_raw(&self) -> v8_sys::ObjectRef {
        self.1
    }
}

impl Array {
    pub fn new(isolate: &isolate::Isolate, context: &context::Context, length: u32) -> Array {
        let _g = context.make_current();
        let raw = unsafe {
            util::invoke_ctx(&isolate,
                             context,
                             |c| v8_sys::v8_Array_New(c, isolate.as_raw(), length as i32))
                .unwrap()
        };
        Array(isolate.clone(), raw)
    }

    /// Creates an array from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::ArrayRef) -> Array {
        Array(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this array.
    pub fn as_raw(&self) -> v8_sys::ArrayRef {
        self.1
    }
}

impl Map {
    pub fn new(isolate: &isolate::Isolate) -> Map {
        let raw = unsafe { util::invoke(&isolate, |c| v8_sys::v8_Map_New(c, isolate.as_raw())).unwrap() };
        Map(isolate.clone(), raw)
    }

    pub fn size(&self) -> usize {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Map_Size(c, self.1)).unwrap() as usize }
    }

    pub fn clear(&self) {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Map_Clear(c, self.1)).unwrap() }
    }

    pub fn get(&self, context: &context::Context, key: &Value) -> Value {
        let raw = unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Map_Get_Key(c, self.1, context.as_raw(), key.as_raw()))
                .unwrap()
        };
        Value(self.0.clone(), raw)
    }

    pub fn set(&self, context: &context::Context, key: &Value, value: &Value) {
        unsafe {
            util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Map_Set_Key(c, self.1, context.as_raw(), key.as_raw(), value.as_raw())
                })
                .unwrap();
        }
    }

    pub fn has(&self, context: &context::Context, key: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0,
                                     context,
                                     |c| v8_sys::v8_Map_Has_Key(c, self.1, context.as_raw(), key.as_raw()))
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn delete(&self, context: &context::Context, key: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Map_Delete_Key(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    /// Returns an array of length Size() * 2, where index N is the Nth key and index N + 1 is the
    /// Nth value.
    pub fn as_array(&self) -> Array {
        let raw = unsafe { util::invoke(&self.0, |c| v8_sys::v8_Map_AsArray(c, self.1)).unwrap() };
        Array(self.0.clone(), raw)
    }

    /// Creates a map from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::MapRef) -> Map {
        Map(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this map.
    pub fn as_raw(&self) -> v8_sys::MapRef {
        self.1
    }
}

impl Set {
    /// Creates a new empty Set.
    pub fn new(isolate: &isolate::Isolate) -> Set {
        let raw = unsafe { util::invoke(&isolate, |c| v8_sys::v8_Set_New(c, isolate.as_raw())).unwrap() };
        Set(isolate.clone(), raw)
    }

    pub fn size(&self) -> usize {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Set_Size(c, self.1)).unwrap() as usize }
    }

    pub fn clear(&self) {
        unsafe { util::invoke(&self.0, |c| v8_sys::v8_Set_Clear(c, self.1)).unwrap() }
    }

    pub fn add(&self, context: &context::Context, key: &Value) {
        unsafe {
            util::invoke_ctx(&self.0,
                             context,
                             |c| v8_sys::v8_Set_Add(c, self.1, context.as_raw(), key.as_raw()))
                .unwrap();
        }
    }

    pub fn has(&self, context: &context::Context, key: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0,
                                     context,
                                     |c| v8_sys::v8_Set_Has_Key(c, self.1, context.as_raw(), key.as_raw()))
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    pub fn delete(&self, context: &context::Context, key: &Value) -> bool {
        unsafe {
            let m = util::invoke_ctx(&self.0, context, |c| {
                    v8_sys::v8_Set_Delete_Key(c, self.1, context.as_raw(), key.as_raw())
                })
                .unwrap();

            assert!( m.is_set);
             m.value
        }
    }

    /// Returns an array of the keys in this Set.
    pub fn as_array(&self) -> Array {
        let raw = unsafe { util::invoke(&self.0, |c| v8_sys::v8_Set_AsArray(c, self.1)).unwrap() };
        Array(self.0.clone(), raw)
    }

    /// Creates a set from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::SetRef) -> Set {
        Set(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this set.
    pub fn as_raw(&self) -> v8_sys::SetRef {
        self.1
    }
}

impl Function {
    /// Create a function in the current execution context for a given callback.
    pub fn new(isolate: &isolate::Isolate,
               context: &context::Context,
               length: usize,
               callback: Box<FunctionCallback>)
               -> Function {
        unsafe {
            let callback_ptr = Box::into_raw(Box::new(callback));
            let callback_ext =
                External::new::<Box<FunctionCallback>>(&isolate, callback_ptr);

            let template = template::ObjectTemplate::new(isolate);
            template.set_internal_field_count(1);

            let closure = template.new_instance(context);
            closure.set_internal_field(0, &callback_ext);

            let raw = util::invoke_ctx(&isolate, context, |c| {
                    v8_sys::v8_Function_New(c,
                                     context.as_raw(),
                                     Some(util::callback),
                                     (&closure as &Value).as_raw(),
                                     length as os::raw::c_int,
                                     v8_sys::ConstructorBehavior::ConstructorBehavior_kAllow)
                })
                .unwrap();
            Function(isolate.clone(), raw)
        }
    }

    /// Call an Object as a function if a callback is set by the
    /// ObjectTemplate::SetCallAsFunctionHandler method.
    pub fn call(&self, context: &context::Context, args: &[&Value]) -> error::Result<Value> {
        let mut arg_ptrs = args.iter().map(|v| v.1).collect::<Vec<_>>();
        let raw = unsafe {
            try!(util::invoke_ctx(&self.0, context, |c| {
                v8_sys::v8_Function_Call(c,
                                  self.1,
                                  context.as_raw(),
                                  ptr::null_mut(),
                                  arg_ptrs.len() as i32,
                                  arg_ptrs.as_mut_ptr())
            }))
        };
        Ok(Value(self.0.clone(), raw))
    }

    /// Call an Object as a function if a callback is set by the
    /// ObjectTemplate::SetCallAsFunctionHandler method.
    pub fn call_with_this(&self,
                          context: &context::Context,
                          this: &Value,
                          args: &[&Value])
                          -> error::Result<Value> {
        let mut arg_ptrs = args.iter().map(|v| v.1).collect::<Vec<_>>();
        let raw = unsafe {
            try!(util::invoke_ctx(&self.0, context, |c| {
                v8_sys::v8_Function_Call(c,
                                  self.1,
                                  context.as_raw(),
                                  this.as_raw(),
                                  arg_ptrs.len() as i32,
                                  arg_ptrs.as_mut_ptr())
            }))
        };
        Ok(Value(self.0.clone(), raw))
    }

    /// Creates a function from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::FunctionRef) -> Function {
        Function(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this function.
    pub fn as_raw(&self) -> v8_sys::FunctionRef {
        self.1
    }
}

impl External {
    pub unsafe fn new<A>(isolate: &isolate::Isolate, value: *mut A) -> External {
        let raw = util::invoke(&isolate, |c| {
                v8_sys::v8_External_New(c, isolate.as_raw(), value as *mut os::raw::c_void)
            })
            .unwrap();
        External(isolate.clone(), raw)
    }

    pub unsafe fn value<A>(&self) -> *mut A {
        util::invoke(&self.0, |c| v8_sys::v8_External_Value(c, self.1)).unwrap() as *mut A
    }

    /// Creates an external from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8_sys::ExternalRef) -> External {
        External(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this external.
    pub fn as_raw(&self) -> v8_sys::ExternalRef {
        self.1
    }
}

impl Exception {
    pub fn range_error(isolate: &isolate::Isolate, message: &String) -> Value {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Exception_RangeError(c, message.as_raw())).unwrap()
        };
        Value(isolate.clone(), raw)
    }

    pub fn reference_error(isolate: &isolate::Isolate, message: &String) -> Value {
        let raw = unsafe {
            util::invoke(&isolate,
                         |c| v8_sys::v8_Exception_ReferenceError(c, message.as_raw()))
                .unwrap()
        };
        Value(isolate.clone(), raw)
    }

    pub fn syntax_error(isolate: &isolate::Isolate, message: &String) -> Value {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Exception_SyntaxError(c, message.as_raw())).unwrap()
        };
        Value(isolate.clone(), raw)
    }

    pub fn type_error(isolate: &isolate::Isolate, message: &String) -> Value {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Exception_TypeError(c, message.as_raw())).unwrap()
        };
        Value(isolate.clone(), raw)
    }

    pub fn error(isolate: &isolate::Isolate, message: &String) -> Value {
        let raw = unsafe {
            util::invoke(&isolate, |c| v8_sys::v8_Exception_Error(c, message.as_raw())).unwrap()
        };
        Value(isolate.clone(), raw)
    }
}

// unsafe: Don't add another `inherit!` line if you don't know the implications (see the comments
// around the macro declaration).
inherit!(Value, Data);

inherit!(Primitive, Value);

inherit!(Boolean, Primitive);
subtype!(Boolean, Value);

inherit!(Name, Primitive);
subtype!(Name, Value);

inherit!(String, Name);
subtype!(String, Primitive);
subtype!(String, Value);

inherit!(Symbol, Name);
subtype!(Symbol, Primitive);
subtype!(Symbol, Value);

inherit!(Private, Name);
subtype!(Private, Primitive);
subtype!(Private, Value);

inherit!(Number, Primitive);
subtype!(Number, Value);

inherit!(Integer, Number);
subtype!(Integer, Primitive);
subtype!(Integer, Value);

inherit!(Int32, Integer);
subtype!(Int32, Number);
subtype!(Int32, Primitive);
subtype!(Int32, Value);

inherit!(Uint32, Integer);
subtype!(Uint32, Number);
subtype!(Uint32, Primitive);
subtype!(Uint32, Value);

inherit!(Object, Value);

inherit!(Array, Object);
subtype!(Array, Value);

inherit!(Map, Object);
subtype!(Map, Value);

inherit!(Set, Object);
subtype!(Set, Value);

inherit!(Function, Object);
subtype!(Function, Value);

inherit!(Promise, Object);
subtype!(Promise, Value);

inherit!(Proxy, Object);
subtype!(Proxy, Value);

inherit!(ArrayBuffer, Object);
subtype!(ArrayBuffer, Value);

inherit!(ArrayBufferView, Object);
subtype!(ArrayBufferView, Value);

inherit!(TypedArray, ArrayBufferView);
subtype!(TypedArray, Object);
subtype!(TypedArray, Value);

inherit!(Uint8Array, TypedArray);
subtype!(Uint8Array, ArrayBufferView);
subtype!(Uint8Array, Object);
subtype!(Uint8Array, Value);

inherit!(Uint8ClampedArray, TypedArray);
subtype!(Uint8ClampedArray, ArrayBufferView);
subtype!(Uint8ClampedArray, Object);
subtype!(Uint8ClampedArray, Value);

inherit!(Int8Array, TypedArray);
subtype!(Int8Array, ArrayBufferView);
subtype!(Int8Array, Object);
subtype!(Int8Array, Value);

inherit!(Uint16Array, TypedArray);
subtype!(Uint16Array, ArrayBufferView);
subtype!(Uint16Array, Object);
subtype!(Uint16Array, Value);

inherit!(Int16Array, TypedArray);
subtype!(Int16Array, ArrayBufferView);
subtype!(Int16Array, Object);
subtype!(Int16Array, Value);

inherit!(Uint32Array, TypedArray);
subtype!(Uint32Array, ArrayBufferView);
subtype!(Uint32Array, Object);
subtype!(Uint32Array, Value);

inherit!(Int32Array, TypedArray);
subtype!(Int32Array, ArrayBufferView);
subtype!(Int32Array, Object);
subtype!(Int32Array, Value);

inherit!(Float32Array, TypedArray);
subtype!(Float32Array, ArrayBufferView);
subtype!(Float32Array, Object);
subtype!(Float32Array, Value);

inherit!(Float64Array, TypedArray);
subtype!(Float64Array, ArrayBufferView);
subtype!(Float64Array, Object);
subtype!(Float64Array, Value);

inherit!(DataView, ArrayBufferView);
subtype!(DataView, Object);
subtype!(DataView, Value);

inherit!(SharedArrayBuffer, Object);
subtype!(SharedArrayBuffer, Value);

inherit!(Date, Object);
subtype!(Date, Value);

inherit!(NumberObject, Object);
subtype!(NumberObject, Value);

inherit!(BooleanObject, Object);
subtype!(BooleanObject, Value);

inherit!(StringObject, Object);
subtype!(StringObject, Value);

inherit!(SymbolObject, Object);
subtype!(SymbolObject, Value);

inherit!(RegExp, Object);
subtype!(RegExp, Value);

inherit!(External, Value);

// unsafe: Don't add another `drop!` line if you don't know the implications (see the comments
// around the macro declaration).
reference!(Data, v8_sys::v8_Data_CloneRef, v8_sys::v8_Data_DestroyRef);
reference!(Value, v8_sys::v8_Value_CloneRef, v8_sys::v8_Value_DestroyRef);
reference!(Primitive, v8_sys::v8_Primitive_CloneRef, v8_sys::v8_Primitive_DestroyRef);
reference!(Boolean, v8_sys::v8_Boolean_CloneRef, v8_sys::v8_Boolean_DestroyRef);
reference!(Name, v8_sys::v8_Name_CloneRef, v8_sys::v8_Name_DestroyRef);
reference!(String, v8_sys::v8_String_CloneRef, v8_sys::v8_String_DestroyRef);
reference!(Symbol, v8_sys::v8_Symbol_CloneRef, v8_sys::v8_Symbol_DestroyRef);
reference!(Private, v8_sys::v8_Private_CloneRef, v8_sys::v8_Private_DestroyRef);
reference!(Number, v8_sys::v8_Number_CloneRef, v8_sys::v8_Number_DestroyRef);
reference!(Integer, v8_sys::v8_Integer_CloneRef, v8_sys::v8_Integer_DestroyRef);
reference!(Int32, v8_sys::v8_Int32_CloneRef, v8_sys::v8_Int32_DestroyRef);
reference!(Uint32, v8_sys::v8_Uint32_CloneRef, v8_sys::v8_Uint32_DestroyRef);
reference!(Object, v8_sys::v8_Object_CloneRef, v8_sys::v8_Object_DestroyRef);
reference!(Array, v8_sys::v8_Array_CloneRef, v8_sys::v8_Array_DestroyRef);
reference!(Map, v8_sys::v8_Map_CloneRef, v8_sys::v8_Map_DestroyRef);
reference!(Set, v8_sys::v8_Set_CloneRef, v8_sys::v8_Set_DestroyRef);
reference!(Function, v8_sys::v8_Function_CloneRef, v8_sys::v8_Function_DestroyRef);
reference!(Promise, v8_sys::v8_Promise_CloneRef, v8_sys::v8_Promise_DestroyRef);
reference!(Proxy, v8_sys::v8_Proxy_CloneRef, v8_sys::v8_Proxy_DestroyRef);
reference!(ArrayBuffer,
           v8_sys::v8_ArrayBuffer_CloneRef,
           v8_sys::v8_ArrayBuffer_DestroyRef);
reference!(ArrayBufferView,
           v8_sys::v8_ArrayBufferView_CloneRef,
           v8_sys::v8_ArrayBufferView_DestroyRef);
reference!(TypedArray,
           v8_sys::v8_TypedArray_CloneRef,
           v8_sys::v8_TypedArray_DestroyRef);
reference!(Uint8Array,
           v8_sys::v8_Uint8Array_CloneRef,
           v8_sys::v8_Uint8Array_DestroyRef);
reference!(Uint8ClampedArray,
           v8_sys::v8_Uint8ClampedArray_CloneRef,
           v8_sys::v8_Uint8ClampedArray_DestroyRef);
reference!(Int8Array, v8_sys::v8_Int8Array_CloneRef, v8_sys::v8_Int8Array_DestroyRef);
reference!(Uint16Array,
           v8_sys::v8_Uint16Array_CloneRef,
           v8_sys::v8_Uint16Array_DestroyRef);
reference!(Int16Array,
           v8_sys::v8_Int16Array_CloneRef,
           v8_sys::v8_Int16Array_DestroyRef);
reference!(Uint32Array,
           v8_sys::v8_Uint32Array_CloneRef,
           v8_sys::v8_Uint32Array_DestroyRef);
reference!(Int32Array,
           v8_sys::v8_Int32Array_CloneRef,
           v8_sys::v8_Int32Array_DestroyRef);
reference!(Float32Array,
           v8_sys::v8_Float32Array_CloneRef,
           v8_sys::v8_Float32Array_DestroyRef);
reference!(Float64Array,
           v8_sys::v8_Float64Array_CloneRef,
           v8_sys::v8_Float64Array_DestroyRef);
reference!(DataView, v8_sys::v8_DataView_CloneRef, v8_sys::v8_DataView_DestroyRef);
reference!(SharedArrayBuffer,
           v8_sys::v8_SharedArrayBuffer_CloneRef,
           v8_sys::v8_SharedArrayBuffer_DestroyRef);
reference!(Date, v8_sys::v8_Date_CloneRef, v8_sys::v8_Date_DestroyRef);
reference!(NumberObject,
           v8_sys::v8_NumberObject_CloneRef,
           v8_sys::v8_NumberObject_DestroyRef);
reference!(BooleanObject,
           v8_sys::v8_BooleanObject_CloneRef,
           v8_sys::v8_BooleanObject_DestroyRef);
reference!(StringObject,
           v8_sys::v8_StringObject_CloneRef,
           v8_sys::v8_StringObject_DestroyRef);
reference!(SymbolObject,
           v8_sys::v8_SymbolObject_CloneRef,
           v8_sys::v8_SymbolObject_DestroyRef);
reference!(RegExp, v8_sys::v8_RegExp_CloneRef, v8_sys::v8_RegExp_DestroyRef);
reference!(External, v8_sys::v8_External_CloneRef, v8_sys::v8_External_DestroyRef);
reference!(Exception, v8_sys::v8_Exception_CloneRef, v8_sys::v8_Exception_DestroyRef);
