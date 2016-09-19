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
//! // Run the compiled script.  The first `unwrap()` panics if the code threw
//! // an exception.  The second `unwrap()` is for if the script did not yield a
//! // result.
//! let result = script.run(&context).unwrap().unwrap();
//!
//! // Convert the result to a value::String.  `unwrap()` panics if the value is
//! // not a string.
//! let result_str = result.to_string(&context).unwrap();
//!
//! // Success!
//! assert_eq!("Hello, World!", result_str.to_string());
//! ```
//!
//! [1]: https://developers.google.com/v8/

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate v8_ng_sys as v8_sys;

mod allocator;
mod platform;
#[macro_use]
mod util;

pub mod context;
pub mod error;
pub mod isolate;
pub mod script;
pub mod value;

pub use context::Context;
pub use isolate::Isolate;
pub use script::Script;
pub use value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    fn eval<F, A>(source: &str, with_result: F) -> error::Result<A>
        where F: FnOnce(&Isolate, &Context, &Value) -> A
    {
        let isolate = Isolate::new();
        let context = Context::new(&isolate);
        let source = value::String::from_str(&isolate, source);
        let script = try!(Script::compile(&isolate, &context, &source));
        let result = try!(script.run(&context)).unwrap();
        Ok(with_result(&isolate, &context, &result))
    }

    #[test]
    fn hello_world() {
        eval("'Hello, ' + 'World!'", |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(c).map(|s| s.to_string());
                assert_eq!(Some("Hello, World!".to_owned()), result);
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
                let result = v.boolean_value(c);
                assert_eq!(Some(false), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_bool_true() {
        eval("true", |_, c, v| {
                assert!(v.is_boolean());
                assert!(v.is_true());
                let result = v.boolean_value(c);
                assert_eq!(Some(true), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_string() {
        eval("'foo'", |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(c).map(|s| s.to_string());
                assert_eq!(Some("foo".to_owned()), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_string_edge_cases() {
        eval(r#"'foo\u0000\uffff௵𒀰\uD808\uDC30'"#, |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(c).map(|s| s.to_string());
                assert_eq!(Some("foo\u{0000}\u{ffff}௵𒀰𒀰".to_owned()), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_uint32() {
        eval("42", |_, c, v| {
                assert!(v.is_number());
                assert!(v.is_uint32());
                let result = v.uint32_value(c);
                assert_eq!(Some(42), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_int32() {
        eval("-42", |_, c, v| {
                assert!(v.is_number());
                assert!(v.is_int32());
                let result = v.int32_value(c);
                assert_eq!(Some(-42), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_integer() {
        // Use largest possible integer n such that all values of 0..n
        // can be represented in Javascript
        eval("9007199254740992", |_, c, v| {
                assert!(v.is_number());
                let result = v.integer_value(c);
                assert_eq!(Some(9007199254740992), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_function() {
        eval("(function() {})", |_, _, v| {
                assert!(v.is_function());
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_call() {
        eval("(function(a) { return a + a; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(c).unwrap();
                let s = value::String::from_str(i, "123");
                let r = f.call(c, &[&s]).unwrap().unwrap();
                assert!(r.is_string());
                let r = r.to_string(c).unwrap();
                assert_eq!("123123", r.to_string());
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_call_with_this() {
        eval("(function() { return this.length; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(c).unwrap();
                let s = value::String::from_str(i, "123");
                let r = f.call_with_this(c, &s, &[]).unwrap().unwrap();
                assert!(r.is_int32());
                let r = r.int32_value(c).unwrap();
                assert_eq!(3, r);
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_construct() {
        eval("(function ctor(a) { this.a = a; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(c).unwrap();
                let a_key = value::String::from_str(i, "a");
                let s = value::String::from_str(i, "123");
                let r = f.call_as_constructor(c, &[&s]).unwrap().unwrap();
                assert!(r.is_object());
                let r = r.to_object(c).unwrap();
                let r = r.get(c, &a_key).unwrap();
                assert!(r.is_string());
                let r = r.to_string(c).unwrap();
                assert_eq!("123", r.to_string());
            })
            .unwrap();
    }

    #[test]
    fn eval_array() {
        eval("[1, true, null]", |_, _, v| {
                assert!(v.is_array());
                // TODO: Try indexing the array?
            })
            .unwrap();
    }

    #[test]
    fn eval_object() {
        eval("({a: 2, b: true})", |i, c, v| {
                assert!(v.is_object());
                let result = v.to_object(c).unwrap();
                let a_key = value::String::from_str(i, "a");
                let b_key = value::String::from_str(i, "b");
                assert_eq!(Some(2),
                           result.get(c, &a_key)
                               .and_then(|v| v.integer_value(c)));
                assert_eq!(Some(true),
                           result.get(c, &b_key)
                               .and_then(|v| v.boolean_value(c)));
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
                          |_, _, _| {
                          });

        let error = result.unwrap_err();
        match error.kind() {
            &error::ErrorKind::Javascript(ref msg, ref stack_trace) => {
                assert_eq!("Uncaught Error: x", msg);
                assert_eq!("    at new w (<unknown>:13:11)\n    at z (<unknown>:10:5)\n    at eval <unknown>:1:1\n    at y (<unknown>:7:5)\n    at x (<unknown>:4:5)\n    at <unknown>:15:3\n    at <unknown>:16:3\n", format!("{}", stack_trace));
            }
            x => panic!("Unexpected error kind: {:?}", x),
        }
    }

    // TODO: test more types
}
