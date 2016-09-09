#[macro_use]
extern crate lazy_static;
extern crate v8_sys;

mod allocator;
mod platform;

pub mod context;
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

    fn eval<F>(source: &str, with_result: F)
        where F: FnOnce(&Isolate, &Context, &Value)
    {
        let isolate = Isolate::new();
        let context = Context::new(&isolate);
        let source = value::String::from_str(&isolate, source);
        let script = Script::compile(&isolate, &context, &source);
        let result = script.run(&context).unwrap();
        with_result(&isolate, &context, &result)
    }

    #[test]
    fn hello_world() {
        eval("'Hello, ' + 'World!'", |_, c, v| {
            let result = v.to_string(c).map(|s| s.to_string());
            assert_eq!(Some("Hello, World!".to_owned()), result);
        });
    }

    #[test]
    fn eval_bool_false() {
        eval("false", |_, c, v| {
            let result = v.boolean_value(c);
            assert_eq!(Some(false), result);
        });
    }

    #[test]
    fn eval_bool_true() {
        eval("true", |_, c, v| {
            let result = v.boolean_value(c);
            assert_eq!(Some(true), result);
        });
    }

    #[test]
    fn eval_string() {
        eval("'foo'", |_, c, v| {
            let result = v.to_string(c).unwrap();
            assert_eq!("foo", result.to_string());
        });
    }

    #[test]
    fn eval_string_edge_cases() {
        eval(r#"'foo\u0000\uffff௵𒀰\uD808\uDC30'"#, |_, c, v| {
            let result = v.to_string(c).unwrap();
            assert_eq!("foo\u{0000}\u{ffff}௵𒀰𒀰", result.to_string());
        });
    }

    #[test]
    fn eval_uint32() {
        eval("42", |_, c, v| {
            let result = v.uint32_value(c).unwrap();
            assert_eq!(42, result);
        });
    }

    #[test]
    fn eval_int32() {
        eval("-42", |_, c, v| {
            let result = v.int32_value(c).unwrap();
            assert_eq!(-42, result);
        });
    }

    #[test]
    fn eval_integer() {
        eval("92233720368", |_, c, v| {
            let result = v.integer_value(c).unwrap();
            assert_eq!(92233720368, result);
        });
    }

    #[test]
    fn eval_object() {
        eval("({a: 2, b: true})", |i, c, v| {
            let result = v.to_object(c).unwrap();
            let a_key = value::String::from_str(i, "a");
            let b_key = value::String::from_str(i, "b");
            assert_eq!(Some(2),
                       result.get(c, &a_key).and_then(|v| v.integer_value(c)));
            assert_eq!(Some(true),
                       result.get(c, &b_key).and_then(|v| v.boolean_value(c)));
        });
    }
}
