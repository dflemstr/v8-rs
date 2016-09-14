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
        let context = try!(Context::new(&isolate));
        let source = try!(value::String::from_str(&isolate, source));
        let script = try!(Script::compile(&isolate, &context, &source));
        let result = try!(script.run(&context)).unwrap();
        Ok(with_result(&isolate, &context, &result))
    }

    #[test]
    fn hello_world() {
        eval("'Hello, ' + 'World!'", |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(c).unwrap().map(|s| s.to_string().unwrap());
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
                let result = v.boolean_value(c).unwrap();
                assert_eq!(Some(false), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_bool_true() {
        eval("true", |_, c, v| {
                assert!(v.is_boolean());
                assert!(v.is_true());
                let result = v.boolean_value(c).unwrap();
                assert_eq!(Some(true), result);
            })
            .unwrap();
    }

    #[test]
    fn eval_string() {
        eval("'foo'", |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(c).unwrap().unwrap();
                assert_eq!("foo", result.to_string().unwrap());
            })
            .unwrap();
    }

    #[test]
    fn eval_string_edge_cases() {
        eval(r#"'foo\u0000\uffff௵𒀰\uD808\uDC30'"#, |_, c, v| {
                assert!(v.is_string());
                let result = v.to_string(c).unwrap().unwrap();
                assert_eq!("foo\u{0000}\u{ffff}௵𒀰𒀰",
                           result.to_string().unwrap());
            })
            .unwrap();
    }

    #[test]
    fn eval_uint32() {
        eval("42", |_, c, v| {
                assert!(v.is_number());
                assert!(v.is_uint32());
                let result = v.uint32_value(c).unwrap().unwrap();
                assert_eq!(42, result);
            })
            .unwrap();
    }

    #[test]
    fn eval_int32() {
        eval("-42", |_, c, v| {
                assert!(v.is_number());
                assert!(v.is_int32());
                let result = v.int32_value(c).unwrap().unwrap();
                assert_eq!(-42, result);
            })
            .unwrap();
    }

    #[test]
    fn eval_integer() {
        eval("92233720368", |_, c, v| {
                assert!(v.is_number());
                let result = v.integer_value(c).unwrap().unwrap();
                assert_eq!(92233720368, result);
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
                let f = v.to_object(c).unwrap().unwrap();
                let s = value::String::from_str(i, "123").unwrap();
                let r = f.call(c, &[&s]).unwrap().unwrap();
                assert!(r.is_string());
                let r = r.to_string(c).unwrap().unwrap();
                assert_eq!("123123", r.to_string().unwrap());
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_call_with_this() {
        eval("(function() { return this.length; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(c).unwrap().unwrap();
                let s = value::String::from_str(i, "123").unwrap();
                let r = f.call_with_this(c, &s, &[]).unwrap().unwrap();
                assert!(r.is_int32());
                let r = r.int32_value(c).unwrap().unwrap();
                assert_eq!(3, r);
            })
            .unwrap();
    }

    #[test]
    fn eval_function_then_construct() {
        eval("(function ctor(a) { this.a = a; })", |i, c, v| {
                assert!(v.is_function());
                let f = v.to_object(c).unwrap().unwrap();
                let a_key = value::String::from_str(i, "a").unwrap();
                let s = value::String::from_str(i, "123").unwrap();
                let r = f.call_as_constructor(c, &[&s]).unwrap().unwrap();
                assert!(r.is_object());
                let r = r.to_object(c).unwrap().unwrap();
                let r = r.get_key(c, &a_key).unwrap().unwrap();
                assert!(r.is_string());
                let r = r.to_string(c).unwrap().unwrap();
                assert_eq!("123", r.to_string().unwrap());
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
                let result = v.to_object(c).unwrap().unwrap();
                let a_key = value::String::from_str(i, "a").unwrap();
                let b_key = value::String::from_str(i, "b").unwrap();
                assert_eq!(Some(2),
                           result.get_key(c, &a_key)
                               .unwrap()
                               .and_then(|v| v.integer_value(c).unwrap()));
                assert_eq!(Some(true),
                           result.get_key(c, &b_key)
                               .unwrap()
                               .and_then(|v| v.boolean_value(c).unwrap()));
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
            &error::ErrorKind::Javascript(ref msg) => {
                assert_eq!("Uncaught SyntaxError: Unexpected end of input", msg);
            }
            x => panic!("Unexpected error kind: {:?}", x),
        }
    }

    // TODO: test more types
}
