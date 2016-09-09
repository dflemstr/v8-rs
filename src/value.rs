use v8_sys as v8;
use context;
use isolate;
use std::mem;
use std::ops;
use std::os;

/// The superclass of all JavaScript values and objects.
pub struct Value<'a>(&'a isolate::Isolate, *mut v8::Value);

/// The superclass of primitive values.  See ECMA-262 4.3.2.
pub struct Primitive<'a>(&'a isolate::Isolate, *mut v8::Primitive);

/// A primitive boolean value (ECMA-262, 4.3.14).  Either the true or false value.
pub struct Boolean<'a>(&'a isolate::Isolate, *mut v8::Boolean);

/// A superclass for symbols and strings.
pub struct Name<'a>(&'a isolate::Isolate, *mut v8::Name);

/// A JavaScript string value (ECMA-262, 4.3.17).
pub struct String<'a>(&'a isolate::Isolate, *mut v8::String);

/// A JavaScript symbol (ECMA-262 edition 6)
///
/// This is an experimental feature. Use at your own risk.
pub struct Symbol<'a>(&'a isolate::Isolate, *mut v8::Symbol);

/// A private symbol
///
/// This is an experimental feature. Use at your own risk.
pub struct Private<'a>(&'a isolate::Isolate, *mut v8::Private);

/// A JavaScript number value (ECMA-262, 4.3.20)
pub struct Number<'a>(&'a isolate::Isolate, *mut v8::Number);

/// A JavaScript value representing a signed integer.
pub struct Integer<'a>(&'a isolate::Isolate, *mut v8::Integer);

/// A JavaScript value representing a 32-bit signed integer.
pub struct Int32<'a>(&'a isolate::Isolate, *mut v8::Int32);

/// A JavaScript value representing a 32-bit unsigned integer.
pub struct Uint32<'a>(&'a isolate::Isolate, *mut v8::Uint32);

/// A JavaScript object (ECMA-262, 4.3.3)
pub struct Object<'a>(&'a isolate::Isolate, *mut v8::Object);

/// An instance of the built-in array constructor (ECMA-262, 15.4.2).
pub struct Array<'a>(&'a isolate::Isolate, *mut v8::Array);

/// An instance of the built-in Map constructor (ECMA-262, 6th Edition, 23.1.1).
pub struct Map<'a>(&'a isolate::Isolate, *mut v8::Map);

/// An instance of the built-in Set constructor (ECMA-262, 6th Edition, 23.2.1).
pub struct Set<'a>(&'a isolate::Isolate, *mut v8::Set);

/// A JavaScript function object (ECMA-262, 15.3).
pub struct Function<'a>(&'a isolate::Isolate, *mut v8::Function);

/// An instance of the built-in Promise constructor (ES6 draft).
///
/// This API is experimental. Only works with --harmony flag.
pub struct Promise<'a>(&'a isolate::Isolate, *mut v8::Promise);

/// An instance of the built-in Proxy constructor (ECMA-262, 6th Edition, 26.2.1).
pub struct Proxy<'a>(&'a isolate::Isolate, *mut v8::Proxy);

pub struct WasmCompiledModule<'a>(&'a isolate::Isolate, *mut v8::WasmCompiledModule);

/// An instance of the built-in ArrayBuffer constructor (ES6 draft 15.13.5).
///
/// This API is experimental and may change significantly.
pub struct ArrayBuffer<'a>(&'a isolate::Isolate, *mut v8::ArrayBuffer);

/// A base class for an instance of one of "views" over ArrayBuffer, including TypedArrays and
/// DataView (ES6 draft 15.13).
///
/// This API is experimental and may change significantly.
pub struct ArrayBufferView<'a>(&'a isolate::Isolate, *mut v8::ArrayBufferView);

/// A base class for an instance of TypedArray series of constructors (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct TypedArray<'a>(&'a isolate::Isolate, *mut v8::TypedArray);

/// An instance of Uint8Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Uint8Array<'a>(&'a isolate::Isolate, *mut v8::Uint8Array);

/// An instance of Uint8ClampedArray constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Uint8ClampedArray<'a>(&'a isolate::Isolate, *mut v8::Uint8ClampedArray);

/// An instance of Int8Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Int8Array<'a>(&'a isolate::Isolate, *mut v8::Int8Array);

/// An instance of Uint16Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Uint16Array<'a>(&'a isolate::Isolate, *mut v8::Uint16Array);

/// An instance of Int16Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Int16Array<'a>(&'a isolate::Isolate, *mut v8::Int16Array);

/// An instance of Uint32Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Uint32Array<'a>(&'a isolate::Isolate, *mut v8::Uint32Array);

/// An instance of Int32Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Int32Array<'a>(&'a isolate::Isolate, *mut v8::Int32Array);

/// An instance of Float32Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Float32Array<'a>(&'a isolate::Isolate, *mut v8::Float32Array);

/// An instance of Float64Array constructor (ES6 draft 15.13.6).
///
/// This API is experimental and may change significantly.
pub struct Float64Array<'a>(&'a isolate::Isolate, *mut v8::Float64Array);

/// An instance of DataView constructor (ES6 draft 15.13.7).
///
/// This API is experimental and may change significantly.
pub struct DataView<'a>(&'a isolate::Isolate, *mut v8::DataView);

/// An instance of the built-in SharedArrayBuffer constructor.
///
/// This API is experimental and may change significantly.
pub struct SharedArrayBuffer<'a>(&'a isolate::Isolate, *mut v8::SharedArrayBuffer);

/// An instance of the built-in Date constructor (ECMA-262, 15.9).
pub struct Date<'a>(&'a isolate::Isolate, *mut v8::Date);

/// A Number object (ECMA-262, 4.3.21).
pub struct NumberObject<'a>(&'a isolate::Isolate, *mut v8::NumberObject);

/// A Boolean object (ECMA-262, 4.3.15).
pub struct BooleanObject<'a>(&'a isolate::Isolate, *mut v8::BooleanObject);

/// A String object (ECMA-262, 4.3.18).
pub struct StringObject<'a>(&'a isolate::Isolate, *mut v8::StringObject);

/// A Symbol object (ECMA-262 edition 6).
///
/// This is an experimental feature. Use at your own risk.
pub struct SymbolObject<'a>(&'a isolate::Isolate, *mut v8::SymbolObject);

/// An instance of the built-in RegExp constructor (ECMA-262, 15.10).
pub struct RegExp<'a>(&'a isolate::Isolate, *mut v8::RegExp);

/// A JavaScript value that wraps an external value. This type of value is mainly used to associate
/// native data structures with JavaScript objects.
pub struct External<'a>(&'a isolate::Isolate, *mut v8::External);

macro_rules! inherit {
    ($child:ident, $parent:ident) => {
        impl<'a> ops::Deref for $child<'a> {
            type Target = $parent<'a>;

            fn deref(&self) -> &Self::Target {
                // SAFETY: This is unsafe because we assume all value `struct`s can be transmuted
                // into each other.
                //
                // I think this is safe because:
                //
                //   - All the structs in this module purposefully have exactly the same fields: one
                //     borrowed `Isolate` and one raw void pointer.
                //
                //   - The `Isolate` field is an immutable borrow and implements `Copy`, so it's
                //     safe to reinterpret.
                //
                //   - The raw void pointers can be converted between each other because that's what
                //     the `Cast` methods all do in `v8.h`.  This will not change because `Cast` is
                //     `template`d and inlined, so it would be a breaking API change.
                //
                unsafe { mem::transmute(self) }
            }
        }
    }
}

macro_rules! drop {
    ($typ:ident, $dtor:expr) => {
        impl<'a> Drop for $typ<'a> {
            fn drop(&mut self) {
                // SAFETY: This is unsafe because it calls a native method with two void pointers.
                // It's safe because the macro is only used with a type and its corresponding
                // destructor.
                unsafe {
                    $dtor(self.0.as_raw(), self.1)
                }
            }
        }
    }
}

macro_rules! type_predicate {
    ($name:ident, $wrapped:expr) => {
        pub fn $name(&self) -> bool {
            // SAFETY: This is unsafe because it calls a native method with two void pointers.  It's
            // safe because the macro is only used with the Value class and one of its methods.
            unsafe { 0 != $wrapped(self.0.as_raw(), self.1) }
        }
    }
}

macro_rules! partial_conversion {
    ($name:ident, $wrapped:expr, $target:ident) => {
        pub fn $name(&self, context: &context::Context) -> Option<$target> {
            // SAFETY: This is unsafe because it calls a native method with two void pointers that
            // returns a pointer.  It's safe because the method belongs to the class of the second
            // pointer, and a null check is made on the returned pointer.
            unsafe {
                map_nullable($wrapped(self.0.as_raw(), self.1, context.as_raw()),
                             |p| $target(self.0, p))
            }
        }
    }
}

macro_rules! partial_get {
    ($name:ident, $wrapped:expr, $target:ident) => {
        pub fn $name(&self, context: &context::Context) -> Option<$target> {
            // SAFETY: This is unsafe because it calls a native method with two void pointers that
            // returns partially uninitialized memory.  It is safe because the pointers are of the
            // right types, and the `is_set` flag specifies whether the rest of the returned
            // `struct` is initialized.
            unsafe {
                let maybe = $wrapped(self.0.as_raw(), self.1, context.as_raw());
                if 0 != maybe.is_set {
                    Some(maybe.value)
                } else {
                    None
                }
            }
        }
    }
}


impl<'a> Value<'a> {
    // TODO: Doc strings for methods

    type_predicate!(is_undefined, v8::Value_IsUndefined);
    type_predicate!(is_null, v8::Value_IsNull);
    type_predicate!(is_true, v8::Value_IsTrue);
    type_predicate!(is_false, v8::Value_IsFalse);
    type_predicate!(is_name, v8::Value_IsName);
    type_predicate!(is_string, v8::Value_IsString);
    type_predicate!(is_symbol, v8::Value_IsSymbol);
    type_predicate!(is_function, v8::Value_IsFunction);
    type_predicate!(is_array, v8::Value_IsArray);
    type_predicate!(is_object, v8::Value_IsObject);
    type_predicate!(is_boolean, v8::Value_IsBoolean);
    type_predicate!(is_number, v8::Value_IsNumber);
    type_predicate!(is_external, v8::Value_IsExternal);
    type_predicate!(is_int32, v8::Value_IsInt32);
    type_predicate!(is_uint32, v8::Value_IsUint32);
    type_predicate!(is_date, v8::Value_IsDate);
    type_predicate!(is_arguments_object, v8::Value_IsArgumentsObject);
    type_predicate!(is_boolean_object, v8::Value_IsBooleanObject);
    type_predicate!(is_number_object, v8::Value_IsNumberObject);
    type_predicate!(is_string_object, v8::Value_IsStringObject);
    type_predicate!(is_symbol_object, v8::Value_IsSymbolObject);
    type_predicate!(is_native_error, v8::Value_IsNativeError);
    type_predicate!(is_reg_exp, v8::Value_IsRegExp);
    type_predicate!(is_generator_function, v8::Value_IsGeneratorFunction);
    type_predicate!(is_generator_object, v8::Value_IsGeneratorObject);
    type_predicate!(is_promise, v8::Value_IsPromise);
    type_predicate!(is_map, v8::Value_IsMap);
    type_predicate!(is_set, v8::Value_IsSet);
    type_predicate!(is_map_iterator, v8::Value_IsMapIterator);
    type_predicate!(is_set_iterator, v8::Value_IsSetIterator);
    type_predicate!(is_weak_map, v8::Value_IsWeakMap);
    type_predicate!(is_weak_set, v8::Value_IsWeakSet);
    type_predicate!(is_array_buffer, v8::Value_IsArrayBuffer);
    type_predicate!(is_array_buffer_view, v8::Value_IsArrayBufferView);
    type_predicate!(is_typed_array, v8::Value_IsTypedArray);
    type_predicate!(is_uint8_array, v8::Value_IsUint8Array);
    type_predicate!(is_uint8_clamped_array, v8::Value_IsUint8ClampedArray);
    type_predicate!(is_int8_array, v8::Value_IsInt8Array);
    type_predicate!(is_uint16_array, v8::Value_IsUint16Array);
    type_predicate!(is_int16_array, v8::Value_IsInt16Array);
    type_predicate!(is_uint32_array, v8::Value_IsUint32Array);
    type_predicate!(is_int32_array, v8::Value_IsInt32Array);
    type_predicate!(is_float32_array, v8::Value_IsFloat32Array);
    type_predicate!(is_float64_array, v8::Value_IsFloat64Array);
    type_predicate!(is_float_32x4, v8::Value_IsFloat32x4);
    type_predicate!(is_data_view, v8::Value_IsDataView);
    type_predicate!(is_shared_array_buffer, v8::Value_IsSharedArrayBuffer);
    type_predicate!(is_proxy, v8::Value_IsProxy);
    type_predicate!(is_web_assembly_compiled_module,
                    v8::Value_IsWebAssemblyCompiledModule);

    partial_conversion!(to_boolean, v8::Value_ToBoolean, Boolean);
    partial_conversion!(to_number, v8::Value_ToNumber, Number);
    partial_conversion!(to_string, v8::Value_ToString, String);
    partial_conversion!(to_detail_string, v8::Value_ToDetailString, String);
    partial_conversion!(to_object, v8::Value_ToObject, Object);
    partial_conversion!(to_integer, v8::Value_ToInteger, Integer);
    partial_conversion!(to_uint32, v8::Value_ToUint32, Uint32);
    partial_conversion!(to_int32, v8::Value_ToInt32, Int32);
    partial_conversion!(to_array_index, v8::Value_ToNumber, Uint32);

    pub fn boolean_value(&self, context: &context::Context) -> Option<bool> {
        // SAFETY: This is unsafe for the same reason as `partial_conversion!`.  The only difference
        // is that extra `0 != ` checks have been added.
        unsafe {
            let maybe = v8::Value_BooleanValue(self.0.as_raw(), self.1, context.as_raw());
            if 0 != maybe.is_set {
                Some(0 != maybe.value)
            } else {
                None
            }
        }
    }

    partial_get!(number_value, v8::Value_NumberValue, f64);
    partial_get!(integer_value, v8::Value_IntegerValue, i64);
    partial_get!(uint32_value, v8::Value_Uint32Value, u32);
    partial_get!(int32_value, v8::Value_Int32Value, i32);

    pub fn equals(&self, context: &context::Context, that: &Value) -> Option<bool> {
        // SAFETY: This is unsafe for the same reason as `boolean_value`.  The only difference is
        // that an additional pointer is involved.
        unsafe {
            let maybe = v8::Value_Equals(self.0.as_raw(), self.1, context.as_raw(), that.as_raw());
            if 0 != maybe.is_set {
                Some(0 != maybe.value)
            } else {
                None
            }
        }
    }

    pub fn strict_equals(&self, that: &Value) -> bool {
        // SAFETY: This is unsafe for the same reason as `boolean_value`.  The only difference is
        // that an additional pointer is involved.
        unsafe { 0 != v8::Value_StrictEquals(self.0.as_raw(), self.1, that.as_raw()) }
    }

    pub fn same_value(&self, that: &Value) -> bool {
        // SAFETY: This is unsafe for the same reason as `boolean_value`.  The only difference is
        // that an additional pointer is involved.
        unsafe { 0 != v8::Value_SameValue(self.0.as_raw(), self.1, that.as_raw()) }
    }

    /// Creates a value from a set of raw pointers
    // SAFETY: This is unsafe because the passed-in pointer actually has type `void *` and could be
    // pointing to anything.
    pub unsafe fn from_raw(isolate: &'a isolate::Isolate, raw: *mut v8::Value) -> Value<'a> {
        Value(isolate, raw)
    }

    /// Returns the underlying raw pointer behind this value.
    pub fn as_raw(&self) -> *mut v8::Value {
        self.1
    }
}

impl<'a> PartialEq for Value<'a> {
    fn eq(&self, other: &Value) -> bool {
        self.strict_equals(other)
    }
}

impl<'a> String<'a> {
    pub fn from_str(isolate: &'a isolate::Isolate, str: &str) -> String<'a> {
        // SAFETY: This is unsafe because a native method is called that reads from memory.  It is
        // safe because the method only reads from the sent-in pointer up to the sent-in length.
        unsafe {
            String(isolate,
                   v8::String_NewFromUtf8_Normal(isolate.as_raw(),
                                                 str.as_ptr() as *const i8,
                                                 str.len() as os::raw::c_int))
        }
    }

    pub fn internalized_from_str(isolate: &'a isolate::Isolate, str: &str) -> String<'a> {
        // SAFETY: This is unsafe for the same reasons as `from_str`.
        unsafe {
            String(isolate,
                   v8::String_NewFromUtf8_Internalized(isolate.as_raw(),
                                                       str.as_ptr() as *const i8,
                                                       str.len() as os::raw::c_int))
        }
    }

    pub fn to_string(&self) -> ::std::string::String {
        // SAFETY: This is unsafe because native code is getting called.  It is safe because the
        // method is a member of the String class.
        let len = unsafe { v8::String_Utf8Length(self.0.as_raw(), self.1) } as usize;
        let mut buf = vec![0u8; len];

        // SAFETY: This is unsafe because native code writes to managed memory, and it might not be
        // valid UTF-8.  It is safe because the underlying method should only write up to the
        // specified length and valid UTF-8.
        unsafe {
            v8::String_WriteUtf8(self.0.as_raw(),
                                 self.1,
                                 buf.as_mut_ptr() as *mut i8,
                                 len as i32);
            ::std::string::String::from_utf8_unchecked(buf)
        }
    }

    pub fn as_raw(&self) -> *mut v8::String {
        self.1
    }
}

impl<'a> Object<'a> {
    pub fn get(&self, context: &context::Context, key: &Value) -> Option<Value> {
        // SAFETY: This is unsafe because a native method is being called.  It is safe because the
        // method is a member of the Object class, and a null check is performed on the returned
        // pointer.
        unsafe {
            let ptr = v8::Object_Get(self.0.as_raw(), self.1, context.as_raw(), key.as_raw());
            map_nullable(ptr, |p| Value(self.0, p))
        }
    }
}

// unsafe: Don't add another `inherit!` line if you don't know the implications (see the comments
// around the macro declaration).
inherit!(Primitive, Value);
inherit!(Boolean, Primitive);
inherit!(Name, Primitive);
inherit!(String, Name);
inherit!(Symbol, Name);
inherit!(Private, Name);
inherit!(Number, Primitive);
inherit!(Integer, Number);
inherit!(Int32, Integer);
inherit!(Uint32, Integer);
inherit!(Object, Value);
inherit!(Array, Object);
inherit!(Map, Object);
inherit!(Set, Object);
inherit!(Function, Object);
inherit!(Promise, Object);
inherit!(Proxy, Object);
inherit!(WasmCompiledModule, Object);
inherit!(ArrayBuffer, Object);
inherit!(ArrayBufferView, Object);
inherit!(TypedArray, ArrayBufferView);
inherit!(Uint8Array, TypedArray);
inherit!(Uint8ClampedArray, TypedArray);
inherit!(Int8Array, TypedArray);
inherit!(Uint16Array, TypedArray);
inherit!(Int16Array, TypedArray);
inherit!(Uint32Array, TypedArray);
inherit!(Int32Array, TypedArray);
inherit!(Float32Array, TypedArray);
inherit!(Float64Array, TypedArray);
inherit!(DataView, ArrayBufferView);
inherit!(SharedArrayBuffer, Object);
inherit!(Date, Object);
inherit!(NumberObject, Object);
inherit!(BooleanObject, Object);
inherit!(StringObject, Object);
inherit!(SymbolObject, Object);
inherit!(RegExp, Object);
inherit!(External, Value);

// unsafe: Don't add another `drop!` line if you don't know the implications (see the comments
// around the macro declaration).
drop!(Value, v8::Value_Destroy);
drop!(Primitive, v8::Primitive_Destroy);
drop!(Boolean, v8::Boolean_Destroy);
drop!(Name, v8::Name_Destroy);
drop!(String, v8::String_Destroy);
drop!(Symbol, v8::Symbol_Destroy);
drop!(Private, v8::Private_Destroy);
drop!(Number, v8::Number_Destroy);
drop!(Integer, v8::Integer_Destroy);
drop!(Int32, v8::Int32_Destroy);
drop!(Uint32, v8::Uint32_Destroy);
drop!(Object, v8::Object_Destroy);
drop!(Array, v8::Array_Destroy);
drop!(Map, v8::Map_Destroy);
drop!(Set, v8::Set_Destroy);
drop!(Function, v8::Function_Destroy);
drop!(Promise, v8::Promise_Destroy);
drop!(Proxy, v8::Proxy_Destroy);
drop!(WasmCompiledModule, v8::WasmCompiledModule_Destroy);
drop!(ArrayBuffer, v8::ArrayBuffer_Destroy);
drop!(ArrayBufferView, v8::ArrayBufferView_Destroy);
drop!(TypedArray, v8::TypedArray_Destroy);
drop!(Uint8Array, v8::Uint8Array_Destroy);
drop!(Uint8ClampedArray, v8::Uint8ClampedArray_Destroy);
drop!(Int8Array, v8::Int8Array_Destroy);
drop!(Uint16Array, v8::Uint16Array_Destroy);
drop!(Int16Array, v8::Int16Array_Destroy);
drop!(Uint32Array, v8::Uint32Array_Destroy);
drop!(Int32Array, v8::Int32Array_Destroy);
drop!(Float32Array, v8::Float32Array_Destroy);
drop!(Float64Array, v8::Float64Array_Destroy);
drop!(DataView, v8::DataView_Destroy);
drop!(SharedArrayBuffer, v8::SharedArrayBuffer_Destroy);
drop!(Date, v8::Date_Destroy);
drop!(NumberObject, v8::NumberObject_Destroy);
drop!(BooleanObject, v8::BooleanObject_Destroy);
drop!(StringObject, v8::StringObject_Destroy);
drop!(SymbolObject, v8::SymbolObject_Destroy);
drop!(RegExp, v8::RegExp_Destroy);
drop!(External, v8::External_Destroy);

fn map_nullable<A, B, F>(ptr: *mut A, func: F) -> Option<B>
    where F: FnOnce(*mut A) -> B
{
    if ptr.is_null() { None } else { Some(func(ptr)) }
}
