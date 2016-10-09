//! A high-level wrapper around the [V8 Javascript engine][1].
//!
//! # Usage
//!
//! First, you need to create an [`Isolate`](isolate/struct.Isolate.html).  An isolate is a VM
//! instance with its own heap.  You should most likely create one per thread and re-use it as much
//! as possible.
//!
//! Then, you need to create a [`Context`](context/struct.Context.html).  A context is an execution
//! environment that allows separate, unrelated, JavaScript code to run in a single instance of V8.
//! You must explicitly specify the context in which you want any JavaScript code to be run.  You
//! should keep track of the context of a script manually as part of your application.
//!
//! # Example
//!
//! ```
//! use v8::{self, value};
//!
//! // Create a V8 heap
//! let isolate = v8::Isolate::new();
//! // Create a new context of execution
//! let context = v8::Context::new(&isolate);
//!
//! // Load the source code that we want to evaluate
//! let source = value::String::from_str(&isolate, "'Hello, ' + 'World!'");
//!
//! // Compile the source code.  `unwrap()` panics if the code is invalid,
//! // e.g. if there is a syntax  error.
//! let script = v8::Script::compile(&isolate, &context, &source).unwrap();
//!
//! // Run the compiled script.  `unwrap()` panics if the code threw an
//! // exception.
//! let result = script.run(&context).unwrap();
//!
//! // Convert the result to a value::String.
//! let result_str = result.to_string(&context);
//!
//! // Success!
//! assert_eq!("Hello, World!", result_str.value());
//! ```
//!
//! [1]: https://developers.google.com/v8/

#![cfg_attr(all(feature="unstable", test), feature(test))]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate v8_sys;

mod allocator;
mod platform;
#[macro_use]
mod util;

pub mod context;
pub mod error;
pub mod isolate;
pub mod script;
pub mod template;
pub mod value;

pub use context::Context;
pub use isolate::Isolate;
pub use script::Script;
pub use value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(source: &str) -> error::Result<(Isolate, Context, Value)> {
        let isolate = Isolate::new();
        let context = Context::new(&isolate);
        let name = value::String::from_str(&isolate, "test.js");
        let source = value::String::from_str(&isolate, source);
        let script = try!(Script::compile_with_name(&isolate, &context, &name, &source));
        let result = try!(script.run(&context));
        Ok((isolate, context, result))
    }

    #[test]
    fn hello_world() {
        let (_, _, v) = eval("'Hello, ' + 'World!'").unwrap();
        assert!(v.is_string());
        let result = v.into_string().unwrap().value();
        assert_eq!("Hello, World!", result);
    }

    #[test]
    fn eval_undefined() {
        let (_, _, v) = eval("undefined").unwrap();
        assert!(v.is_undefined());
    }

    #[test]
    fn eval_null() {
        let (_, _, v) = eval("null").unwrap();
        assert!(v.is_null());
    }

    #[test]
    fn eval_bool_false() {
        let (_, c, v) = eval("false").unwrap();
        assert!(v.is_boolean());
        assert!(v.is_false());
        assert_eq!(false, v.boolean_value(&c));
        let v = v.into_boolean().unwrap();
        assert_eq!(false, v.value());
    }

    #[test]
    fn eval_bool_true() {
        let (_, c, v) = eval("true").unwrap();
        assert!(v.is_boolean());
        assert!(v.is_true());
        assert_eq!(true, v.boolean_value(&c));
        let v = v.into_boolean().unwrap();
        assert_eq!(true, v.value());
    }

    #[test]
    fn eval_string() {
        let (_, c, v) = eval("'foo'").unwrap();
        assert!(v.is_string());
        assert_eq!("foo", v.to_string(&c).value());
        let v = v.into_string().unwrap();
        assert_eq!("foo", v.value());
    }

    #[test]
    fn eval_string_length() {
        let (_, _, v) = eval("'foo'").unwrap();
        assert!(v.is_string());
        let v = v.into_string().unwrap();
        assert_eq!(3, v.length());
    }

    #[test]
    fn eval_string_utf8_length_1() {
        let (_, _, v) = eval("'a'").unwrap();
        assert!(v.is_string());
        let v = v.into_string().unwrap();
        assert_eq!(1, v.utf8_length());
    }

    #[test]
    fn eval_string_utf8_length_2() {
        let (_, _, v) = eval("'ä'").unwrap();
        assert!(v.is_string());
        let v = v.into_string().unwrap();
        assert_eq!(2, v.utf8_length());
    }

    #[test]
    fn eval_string_utf8_length_3() {
        let (_, _, v) = eval("'௵'").unwrap();
        assert!(v.is_string());
        let v = v.into_string().unwrap();
        assert_eq!(3, v.utf8_length());
    }

    #[test]
    fn eval_string_utf8_length_4() {
        let (_, _, v) = eval("'𒀰'").unwrap();
        assert!(v.is_string());
        let v = v.into_string().unwrap();
        assert_eq!(4, v.utf8_length());
    }

    #[test]
    fn eval_string_edge_cases() {
        let (_, _, v) = eval(r#"'foo\u0000\uffff௵𒀰\uD808\uDC30'"#).unwrap();
        assert!(v.is_string());
        let v = v.into_string().unwrap();
        assert_eq!("foo\u{0000}\u{ffff}௵𒀰𒀰", v.value());
    }

    #[test]
    fn eval_uint32() {
        let (_, c, v) = eval("42").unwrap();
        assert!(v.is_number());
        assert!(v.is_uint32());
        assert_eq!(42, v.uint32_value(&c));
        let v = v.into_uint32().unwrap();
        assert_eq!(42, v.value());
    }

    #[test]
    fn eval_int32() {
        let (_, c, v) = eval("-42").unwrap();
        assert!(v.is_number());
        assert!(v.is_int32());
        assert_eq!(-42, v.int32_value(&c));
        let v = v.into_int32().unwrap();
        assert_eq!(-42, v.value());
    }

    #[test]
    fn eval_integer() {
        // Use largest possible integer n such that all values of 0..n
        // can be represented in Javascript
        let (_, c, v) = eval("9007199254740992").unwrap();
        assert!(v.is_number());
        assert_eq!(9007199254740992, v.integer_value(&c));
    }

    #[test]
    fn eval_function() {
        let (i, c, v) = eval("(function(a, b) { return a + b; })").unwrap();
        let a = value::Integer::new(&i, 3);
        let b = value::Integer::new(&i, 4);
        let f = v.into_function().unwrap();
        let r = f.call(&c, &[&a, &b]).unwrap();
        assert!(r.is_int32());
        assert_eq!(7, r.int32_value(&c));
    }

    #[test]
    fn eval_equals_true() {
        let (i, c, v) = eval("({a: '', b: []})").unwrap();
        assert!(v.is_object());
        let v = v.into_object().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let b_key = value::String::from_str(&i, "b");
        assert!(v.get(&c, &a_key).equals(&c, &v.get(&c, &b_key)));
    }

    #[test]
    fn eval_equals_false() {
        let (i, c, v) = eval("({a: '', b: 1})").unwrap();
        assert!(v.is_object());
        let v = v.into_object().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let b_key = value::String::from_str(&i, "b");
        assert!(!v.get(&c, &a_key).equals(&c, &v.get(&c, &b_key)));
    }

    #[test]
    fn eval_strict_equals_true() {
        let (i, c, v) = eval("({a: 2, b: 2})").unwrap();
        assert!(v.is_object());
        let v = v.into_object().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let b_key = value::String::from_str(&i, "b");
        assert!(v.get(&c, &a_key).strict_equals(&v.get(&c, &b_key)));
    }

    #[test]
    fn eval_strict_equals_false() {
        let (i, c, v) = eval("({a: '', b: []})").unwrap();
        assert!(v.is_object());
        let v = v.into_object().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let b_key = value::String::from_str(&i, "b");
        assert!(!v.get(&c, &a_key).strict_equals(&v.get(&c, &b_key)));
    }

    #[test]
    fn eval_same_value_true() {
        let (i, c, v) = eval("(function() { var a = {}; return {a: a, b: a}; })()").unwrap();
        assert!(v.is_object());
        let v = v.into_object().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let b_key = value::String::from_str(&i, "b");
        assert!(v.get(&c, &a_key).same_value(&v.get(&c, &b_key)));
    }

    #[test]
    fn eval_same_value_false() {
        let (i, c, v) = eval("({a: {}, b: {}})").unwrap();
        assert!(v.is_object());
        let v = v.into_object().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let b_key = value::String::from_str(&i, "b");
        assert!(!v.get(&c, &a_key).same_value(&v.get(&c, &b_key)));
    }

    #[test]
    fn eval_function_then_call() {
        let (i, c, v) = eval("(function(a) { return a + a; })").unwrap();
        assert!(v.is_function());
        let f = v.into_function().unwrap();
        let s = value::String::from_str(&i, "123");
        let r = f.call(&c, &[&s]).unwrap();
        assert!(r.is_string());
        assert_eq!("123123", r.into_string().unwrap().value());
    }

    #[test]
    fn eval_function_then_call_with_this() {
        let (i, c, v) = eval("(function() { return this.length; })").unwrap();
        assert!(v.is_function());
        let f = v.into_function().unwrap();
        let s = value::String::from_str(&i, "123");
        let r = f.call_with_this(&c, &s, &[]).unwrap();
        assert!(r.is_int32());
        assert_eq!(3, r.int32_value(&c));
    }

    #[test]
    fn eval_function_then_construct() {
        let (i, c, v) = eval("(function ctor(a) { this.a = a; })").unwrap();
        assert!(v.is_function());
        let f = v.into_function().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let s = value::String::from_str(&i, "123");
        let r = f.call_as_constructor(&c, &[&s]).unwrap();
        assert!(r.is_object());
        let r = r.into_object().unwrap();
        let r = r.get(&c, &a_key);
        assert!(r.is_string());
        let r = r.to_string(&c);
        assert_eq!("123", r.value());
    }

    #[test]
    fn eval_array() {
        let (_, c, v) = eval("[1, true, null]").unwrap();
        assert!(v.is_array());
        let v = v.into_object().unwrap();
        assert!(v.get_index(&c, 0).is_number());
        assert!(v.get_index(&c, 1).is_boolean());
        assert!(v.get_index(&c, 2).is_null());
    }

    #[test]
    fn eval_object() {
        let (i, c, v) = eval("({a: 2, b: true})").unwrap();
        assert!(v.is_object());
        let result = v.into_object().unwrap();
        let a_key = value::String::from_str(&i, "a");
        let b_key = value::String::from_str(&i, "b");
        assert_eq!(2, result.get(&c, &a_key).integer_value(&c));
        assert_eq!(true, result.get(&c, &b_key).boolean_value(&c));
    }

    #[test]
    fn eval_date() {
        let (_, _, v) = eval("new Date(0)").unwrap();
        assert!(v.is_date());
    }

    #[test]
    fn eval_arguments_object() {
        let (_, _, v) = eval("(function() { return arguments; })()").unwrap();
        assert!(v.is_arguments_object());
    }

    #[test]
    fn eval_boolean_object() {
        let (_, _, v) = eval("new Boolean(true)").unwrap();
        assert!(v.is_boolean_object());
    }

    #[test]
    fn eval_number_object() {
        let (_, _, v) = eval("new Number(42)").unwrap();
        assert!(v.is_number_object());
    }

    #[test]
    fn eval_string_object() {
        let (_, _, v) = eval("new String('abc')").unwrap();
        assert!(v.is_string_object());
    }

    #[test]
    fn eval_symbol_object() {
        let (_, _, v) = eval("Object(Symbol('abc'))").unwrap();
        assert!(v.is_symbol_object());
    }

    #[test]
    fn eval_native_error() {
        let (_, _, v) = eval("new Error()").unwrap();
        assert!(v.is_native_error());
    }

    #[test]
    fn eval_reg_exp() {
        let (_, _, v) = eval("/./").unwrap();
        assert!(v.is_reg_exp());
    }

    #[test]
    fn eval_generator_function() {
        let (_, _, v) = eval("(function* () {})").unwrap();
        assert!(v.is_generator_function());
    }

    #[test]
    fn eval_generator_object() {
        let (_, _, v) = eval("(function* () {})()").unwrap();
        assert!(v.is_generator_object());
    }

    #[test]
    fn eval_promise() {
        let (_, _, v) = eval("new Promise(function() {})").unwrap();
        assert!(v.is_promise());
    }

    #[test]
    fn eval_map() {
        let (_, _, v) = eval("new Map()").unwrap();
        assert!(v.is_map());
    }

    #[test]
    fn eval_set() {
        let (_, _, v) = eval("new Set()").unwrap();
        assert!(v.is_set());
    }

    #[test]
    fn eval_map_iterator() {
        // TODO: how?
    }

    #[test]
    fn eval_set_iterator() {
        // TODO: how?
    }

    #[test]
    fn eval_syntax_error() {
        let result = eval("(");

        let error = result.unwrap_err();
        match error.kind() {
            &error::ErrorKind::Javascript(ref msg, _) => {
                assert_eq!("Uncaught SyntaxError: Unexpected end of input", msg);
            }
            x => panic!("Unexpected error kind: {:?}", x),
        }
    }

    #[test]
    fn eval_exception() {
        let result = eval("throw 'x';");

        let error = result.unwrap_err();
        match error.kind() {
            &error::ErrorKind::Javascript(ref msg, _) => {
                assert_eq!("Uncaught x", msg);
            }
            x => panic!("Unexpected error kind: {:?}", x),
        }
    }

    #[test]
    fn eval_exception_stack() {
        let result = eval(r#"
(function() {
  function x() {
    y();
  }
  function y() {
    eval("z()");
  }
  function z() {
    new w();
  }
  function w() {
    throw new Error('x');
  }
  x();
})();
"#);

        let error = result.unwrap_err();
        match error.kind() {
            &error::ErrorKind::Javascript(ref msg, ref stack_trace) => {
                assert_eq!("Uncaught Error: x", msg);
                assert_eq!("    at new w (test.js:13:11)\n    at z (test.js:10:5)\n    at eval \
                            <anon>:1:1\n    at y (test.js:7:5)\n    at x (test.js:4:5)\n    at \
                            test.js:15:3\n    at test.js:16:3\n",
                           format!("{}", stack_trace));
            }
            x => panic!("Unexpected error kind: {:?}", x),
        }
    }

    #[test]
    fn run_native_function_call() {
        let isolate = Isolate::new();
        let context = Context::new(&isolate);

        let function = value::Function::new(&isolate, &context, 1, Box::new(|mut info| {
            info.args.remove(0)
        }));
        let param = value::Integer::new(&isolate, 42);

        let result = function.call(&context, &[&param]).unwrap();
        assert_eq!(42, result.uint32_value(&context));
    }

    #[test]
    fn run_defined_function() {
        let i = Isolate::new();
        let c = Context::new(&i);

        let fi = i.clone();
        let fc = c.clone();
        let f = value::Function::new(&i, &c, 2, Box::new(move |info| {
            assert_eq!(2, info.length);
            let ref a = info.args[0];
            assert!(a.is_int32());
            let a = a.int32_value(&fc);
            let ref b = info.args[1];
            assert!(b.is_int32());
            let b = b.int32_value(&fc);

            value::Integer::new(&fi, a + b).into()
        }));

        let k = value::String::from_str(&i, "f");
        c.global().set(&c, &k, &f);

        let name = value::String::from_str(&i, "test.js");
        let source = value::String::from_str(&i, "f(2, 3)");
        let script = Script::compile_with_name(&i, &c, &name, &source).unwrap();
        let result = script.run(&c).unwrap();

        assert!(result.is_int32());
        assert_eq!(5, result.int32_value(&c));
    }

    fn test_function(info: value::FunctionCallbackInfo) -> value::Value {
        let i = info.isolate;
        let c = i.current_context().unwrap();

        assert_eq!(2, info.length);
        let ref a = info.args[0];
        assert!(a.is_int32());
        let a = a.int32_value(&c);
        let ref b = info.args[1];
        assert!(b.is_int32());
        let b = b.int32_value(&c);

        value::Integer::new(&i, a + b).into()
    }

    #[test]
    fn run_defined_static_function() {
        let i = Isolate::new();
        let c = Context::new(&i);
        let f = value::Function::new(&i, &c, 2, Box::new(test_function));

        let k = value::String::from_str(&i, "f");
        c.global().set(&c, &k, &f);

        let name = value::String::from_str(&i, "test.js");
        let source = value::String::from_str(&i, "f(2, 3)");
        let script = Script::compile_with_name(&i, &c, &name, &source).unwrap();
        let result = script.run(&c).unwrap();

        assert!(result.is_int32());
        assert_eq!(5, result.int32_value(&c));
    }

    #[test]
    fn run_defined_function_template_instance() {
        let i = Isolate::new();
        let c = Context::new(&i);
        let ft = template::FunctionTemplate::new(&i, &c, Box::new(test_function));
        let f = ft.get_function(&c);

        let k = value::String::from_str(&i, "f");
        c.global().set(&c, &k, &f);

        let name = value::String::from_str(&i, "test.js");
        let source = value::String::from_str(&i, "f(2, 3)");
        let script = Script::compile_with_name(&i, &c, &name, &source).unwrap();
        let result = script.run(&c).unwrap();

        assert!(result.is_int32());
        assert_eq!(5, result.int32_value(&c));
    }

    #[test]
    fn create_object_template_instance() {
        let i = Isolate::new();
        let c = Context::new(&i);
        let ot = template::ObjectTemplate::new(&i);
        ot.set("test", &value::Integer::new(&i, 5));

        let o = ot.new_instance(&c);
        let k = value::String::from_str(&i, "o");
        c.global().set(&c, &k, &o);

        let name = value::String::from_str(&i, "test.js");
        let source = value::String::from_str(&i, "o.test");
        let script = Script::compile_with_name(&i, &c, &name, &source).unwrap();
        let result = script.run(&c).unwrap();

        assert!(result.is_int32());
        assert_eq!(5, result.int32_value(&c));
    }

    #[test]
    fn run_object_template_instance_function() {
        let i = Isolate::new();
        let c = Context::new(&i);
        let ot = template::ObjectTemplate::new(&i);
        let ft = template::FunctionTemplate::new(&i, &c, Box::new(test_function));
        ot.set("f", &ft);

        let o = ot.new_instance(&c);
        let k = value::String::from_str(&i, "o");
        c.global().set(&c, &k, &o);

        let name = value::String::from_str(&i, "test.js");
        let source = value::String::from_str(&i, "o.f(2, 3)");
        let script = Script::compile_with_name(&i, &c, &name, &source).unwrap();
        let result = script.run(&c).unwrap();

        assert!(result.is_int32());
        assert_eq!(5, result.int32_value(&c));
    }

    #[test]
    fn isolate_rc() {
        let (f, c, p) = {
            let isolate = Isolate::new();
            let context = Context::new(&isolate);
            let param = value::Integer::new(&isolate, 42);

            let function = value::Function::new(&isolate, &context, 1, Box::new(|mut info| {
                info.args.remove(0)
            }));
            (function, context, param)
        };

        let result = f.call(&c, &[&p]).unwrap();
        assert_eq!(42, result.uint32_value(&c));
    }

    #[test]
    fn closure_lifetime() {
        struct Foo {
            msg: String
        }

        let isolate = Isolate::new();
        let context = Context::new(&isolate);

        let f = {
            let foo = Foo {
                msg: "Hello, World!".into()
            };

            let closure_isolate = isolate.clone();
            value::Function::new(&isolate, &context, 0, Box::new(move |_| {
                assert_eq!("Hello, World!", &foo.msg);

                value::undefined(&closure_isolate).into()
            }))
        };

        let bar = Foo {
            msg: "Goodbye, World!".into()
        };
        let name = value::String::from_str(&isolate, "f");
        context.global().set(&context, &name, &f);

        let source = value::String::from_str(&isolate, "f();");
        let script = Script::compile(&isolate, &context, &source).unwrap();
        let result = script.run(&context).unwrap();

        assert_eq!("Goodbye, World!", bar.msg);
        assert!(result.is_undefined());
    }
}

#[cfg(all(feature="unstable", test))]
mod benches {
    extern crate test;

    use super::*;

    #[bench]
    fn js_function_call(bencher: &mut test::Bencher) {
        let isolate = Isolate::new();
        let context = Context::new(&isolate);
        let name = value::String::from_str(&isolate, "test.js");
        let source = value::String::from_str(&isolate, "(function(a) { return a; })");
        let script = Script::compile_with_name(&isolate, &context, &name, &source).unwrap();
        let result = script.run(&context).unwrap();

        let function = result.into_function().unwrap();
        let param = value::Integer::new(&isolate, 42);

        bencher.iter(|| {
            function.call(&context, &[&param]).unwrap()
        });
    }

    #[bench]
    fn native_function_call(bencher: &mut test::Bencher) {
        let isolate = Isolate::new();
        let context = Context::new(&isolate);

        let function = value::Function::new(&isolate, &context, 1, Box::new(|mut info| {
            info.args.remove(0)
        }));
        let param = value::Integer::new(&isolate, 42);

        bencher.iter(|| {
            function.call(&context, &[&param]).unwrap()
        });
    }
}
