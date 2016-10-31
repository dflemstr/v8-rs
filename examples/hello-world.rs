extern crate v8;

use v8::value;

fn main() {
    let isolate = v8::Isolate::new();
    let context = v8::Context::new(&isolate);

    let source = value::String::from_str(&isolate, "'Hello, ' + 'World!'");
    let script = v8::Script::compile(&isolate, &context, &source).unwrap();

    let result = script.run(&context).unwrap();
    let result_str = result.to_string(&context);

    println!("{}", result_str.value());
}
