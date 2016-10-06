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
//! assert_eq!("Hello, World!", result_str.to_string());
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

    fn eval<F, A>(source: &str, with_result: F) -> error::Result<A>
        where F: FnOnce(Isolate, Context, Value) -> A
    {
        let isolate = Isolate::new();
        let context = Context::new(&isolate);
        let name = value::String::from_str(&isolate, "test.js");
        let source = value::String::from_str(&isolate, source);
        let script = try!(Script::compile_with_name(&isolate, &context, &name, &source));
        let result = try!(script.run(&context));
        Ok(with_result(isolate, context, result))
    }

    #[test]
    fn hello_world() {
        eval("'Hello, ' + 'World!'", |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(&c).to_string();
                assert_eq!("Hello, World!".to_owned(), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_undefined() {
        eval("undefined", |_, _, v| {
                assert!(v.is_undefined());
            })
            .unwrap();
    }

    #[test]
    fn eval_null() {
        eval("null", |_, _, v| {
                assert!(v.is_null());
            })
            .unwrap();
    }

    #[test]
    fn eval_bool_false() {
        eval("false", |_, c, v| {
                assert!(v.is_boolean());
                assert!(v.is_false());
                assert_eq!(false, v.boolean_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_bool_true() {
        eval("true", |_, c, v| {
                assert!(v.is_boolean());
                assert!(v.is_true());
                assert_eq!(true, v.boolean_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_string() {
        eval("'foo'", |_, c, v| {
                assert!(v.is_string());
                assert_eq!("foo".to_owned(), v.to_string(&c).to_string());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_length() {
        eval("'foo'", |_, c, v| {
                assert!(v.is_string());
                assert_eq!(3, v.to_string(&c).length());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_utf8_length_1() {
        eval("'a'", |_, c, v| {
                assert!(v.is_string());
                assert_eq!(1, v.to_string(&c).utf8_length());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_utf8_length_2() {
        eval("'ä'", |_, c, v| {
                assert!(v.is_string());
                assert_eq!(2, v.to_string(&c).utf8_length());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_utf8_length_3() {
        eval("'௵'", |_, c, v| {
                assert!(v.is_string());
                assert_eq!(3, v.to_string(&c).utf8_length());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_utf8_length_4() {
        eval("'𒀰'", |_, c, v| {
                assert!(v.is_string());
                assert_eq!(4, v.to_string(&c).utf8_length());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_edge_cases() {
        eval(r#"'foo\u0000\uffff௵𒀰\uD808\uDC30'"#, |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(&c).to_string();
                assert_eq!("foo\u{0000}\u{ffff}௵𒀰𒀰".to_owned(), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_uint32() {
        eval("42", |_, c, v| {
                assert!(v.is_number());
                assert!(v.is_uint32());
                assert_eq!(42, v.uint32_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_int32() {
        eval("-42", |_, c, v| {
                assert!(v.is_number());
                assert!(v.is_int32());
                assert_eq!(-42, v.int32_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_integer() {
        // Use largest possible integer n such that all values of 0..n
        // can be represented in Javascript
        eval("9007199254740992", |_, c, v| {
                assert!(v.is_number());
                assert_eq!(9007199254740992, v.integer_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_function() {
        eval("(function(a, b) { return a + b; })", |i, c, v| {
                let a = value::Integer::new(&i, 3);
                let b = value::Integer::new(&i, 4);
                let f = v.into_function().unwrap();
                let r = f.call(&c, &[&a, &b]).unwrap();
                assert!(r.is_int32());
                assert_eq!(7, r.int32_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_equals_true() {
        eval("({a: '', b: []})", |i, c, v| {
                assert!(v.is_object());
                let v = v.to_object(&c);
                let a_key = value::String::from_str(&i, "a");
                let b_key = value::String::from_str(&i, "b");
                assert!(v.get(&c, &a_key).equals(&c, &v.get(&c, &b_key)))
            })
            .unwrap();
    }

    #[test]
    fn eval_equals_false() {
        eval("({a: '', b: 1})", |i, c, v| {
                assert!(v.is_object());
                let v = v.to_object(&c);
                let a_key = value::String::from_str(&i, "a");
                let b_key = value::String::from_str(&i, "b");
                assert!(!v.get(&c, &a_key).equals(&c, &v.get(&c, &b_key)))
            })
            .unwrap();
    }

    #[test]
    fn eval_strict_equals_true() {
        eval("({a: 2, b: 2})", |i, c, v| {
                assert!(v.is_object());
                let v = v.to_object(&c);
                let a_key = value::String::from_str(&i, "a");
                let b_key = value::String::from_str(&i, "b");
                assert!(v.get(&c, &a_key).strict_equals(&v.get(&c, &b_key)))
            })
            .unwrap();
    }

    #[test]
    fn eval_strict_equals_false() {
        eval("({a: '', b: []})", |i, c, v| {
                assert!(v.is_object());
                let v = v.to_object(&c);
                let a_key = value::String::from_str(&i, "a");
                let b_key = value::String::from_str(&i, "b");
                assert!(!v.get(&c, &a_key).strict_equals(&v.get(&c, &b_key)))
            })
            .unwrap();
    }

    #[test]
    fn eval_same_value_true() {
        eval("(function() { var a = {}; return {a: a, b: a}; })()",
             |i, c, v| {
            assert!(v.is_object());
            let v = v.to_object(&c);
            let a_key = value::String::from_str(&i, "a");
            let b_key = value::String::from_str(&i, "b");
            assert!(v.get(&c, &a_key).same_value(&v.get(&c, &b_key)))
        })
            .unwrap();
    }

    #[test]
    fn eval_same_value_false() {
        eval("({a: {}, b: {}})", |i, c, v| {
                assert!(v.is_object());
                let v = v.to_object(&c);
                let a_key = value::String::from_str(&i, "a");
                let b_key = value::String::from_str(&i, "b");
                assert!(!v.get(&c, &a_key).same_value(&v.get(&c, &b_key)))
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_call() {
        eval("(function(a) { return a + a; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(&c);
                let s = value::String::from_str(&i, "123");
                let r = f.call(&c, &[&s]).unwrap();
                assert!(r.is_string());
                assert_eq!("123123", r.to_string(&c).to_string());
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_call_with_this() {
        eval("(function() { return this.length; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(&c);
                let s = value::String::from_str(&i, "123");
                let r = f.call_with_this(&c, &s, &[]).unwrap();
                assert!(r.is_int32());
                assert_eq!(3, r.int32_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_construct() {
        eval("(function ctor(a) { this.a = a; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(&c);
                let a_key = value::String::from_str(&i, "a");
                let s = value::String::from_str(&i, "123");
                let r = f.call_as_constructor(&c, &[&s]).unwrap();
                assert!(r.is_object());
                let r = r.to_object(&c);
                let r = r.get(&c, &a_key);
                assert!(r.is_string());
                let r = r.to_string(&c);
                assert_eq!("123", r.to_string());
            })
            .unwrap();
    }

    #[test]
    fn eval_array() {
        eval("[1, true, null]", |_, c, v| {
                assert!(v.is_array());
                let v = v.to_object(&c);
                assert!(v.get_index(&c, 0).is_number());
                assert!(v.get_index(&c, 1).is_boolean());
                assert!(v.get_index(&c, 2).is_null());
            })
            .unwrap();
    }

    #[test]
    fn eval_object() {
        eval("({a: 2, b: true})", |i, c, v| {
                assert!(v.is_object());
                let result = v.to_object(&c);
                let a_key = value::String::from_str(&i, "a");
                let b_key = value::String::from_str(&i, "b");
                assert_eq!(2, result.get(&c, &a_key).integer_value(&c));
                assert_eq!(true, result.get(&c, &b_key).boolean_value(&c));
            })
            .unwrap();
    }

    #[test]
    fn eval_date() {
        eval("new Date(0)", |_, _, v| {
                assert!(v.is_date());
            })
            .unwrap();
    }

    #[test]
    fn eval_arguments_object() {
        eval("(function() { return arguments; })()", |_, _, v| {
                assert!(v.is_arguments_object());
            })
            .unwrap();
    }

    #[test]
    fn eval_boolean_object() {
        eval("new Boolean(true)", |_, _, v| {
                assert!(v.is_boolean_object());
            })
            .unwrap();
    }

    #[test]
    fn eval_number_object() {
        eval("new Number(42)", |_, _, v| {
                assert!(v.is_number_object());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_object() {
        eval("new String('abc')", |_, _, v| {
                assert!(v.is_string_object());
            })
            .unwrap();
    }

    #[test]
    fn eval_symbol_object() {
        // TODO: how?
    }

    #[test]
    fn eval_reg_exp() {
        eval("/./", |_, _, v| {
                assert!(v.is_reg_exp());
            })
            .unwrap();
    }

    #[test]
    fn eval_generator_function() {
        eval("(function* () {})", |_, _, v| {
                assert!(v.is_generator_function());
            })
            .unwrap();
    }

    #[test]
    fn eval_generator_object() {
        // TODO: how?
    }

    #[test]
    fn eval_promise() {
        eval("new Promise(function() {})", |_, _, v| {
                assert!(v.is_promise());
            })
            .unwrap();
    }

    #[test]
    fn eval_map() {
        eval("new Map()", |_, _, v| {
                assert!(v.is_map());
            })
            .unwrap();
    }

    #[test]
    fn eval_set() {
        eval("new Set()", |_, _, v| {
                assert!(v.is_set());
            })
            .unwrap();
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
        let result = eval("(", |_, _, _| {
        });

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
        let result = eval("throw 'x';", |_, _, _| {
        });

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
"#,
                          |_, _, _| {});

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
    fn run_defined_function() {
        let i = Isolate::new();
        let c = Context::new(&i);

        let f = value::Function::new(&i, &c, 2, |info: value::FunctionCallbackInfo| {
            assert_eq!(2, info.length);
            let ref a = info.args[0];
            assert!(a.is_int32());
            let a = a.int32_value(&c);
            let ref b = info.args[1];
            assert!(b.is_int32());
            let b = b.int32_value(&c);

            value::Integer::new(&i, a + b).into()
        });

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
        let fr = &test_function;
        let f = value::Function::new(&i, &c, 2, fr);

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
        let fr = &test_function;
        let ft = template::FunctionTemplate::new(&i, &c, fr);
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
        let fr = &test_function;
        let ft = template::FunctionTemplate::new(&i, &c, fr);
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

        let function = value::Function::new(&isolate, &context, 1, |mut info: value::FunctionCallbackInfo| {
            info.args.remove(0)
        });
        let param = value::Integer::new(&isolate, 42);

        bencher.iter(|| {
            function.call(&context, &[&param]).unwrap()
        });
    }
}
