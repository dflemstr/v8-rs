use v8_sys as v8;

use context;
use isolate;
use value;

pub struct Script<'a>(&'a isolate::Isolate, *mut v8::Script);

impl<'a> Script<'a> {
    pub fn compile(isolate: &'a isolate::Isolate,
                   context: &context::Context,
                   source: &value::String)
                   -> Script<'a> {
        unsafe {
            Script(isolate,
                   v8::Script_Compile(isolate.as_raw(), context.as_raw(), source.as_raw()))
        }
    }

    pub fn run(&self, context: &context::Context) -> Option<value::Value> {
        unsafe {
            let ptr = v8::Script_Run(self.0.as_raw(), self.1, context.as_raw());
            if ptr.is_null() {
                None
            } else {
                Some(value::Value::from_raw(self.0, ptr))
            }
        }
    }
}

impl<'a> Drop for Script<'a> {
    fn drop(&mut self) {
        unsafe { v8::Script_Destroy(self.0.as_raw(), self.1) }
    }
}
