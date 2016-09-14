use v8_sys as v8;

use context;
use error;
use isolate;
use value;
use util;

pub struct Script<'a>(&'a isolate::Isolate, v8::ScriptRef);

impl<'a> Script<'a> {
    pub fn compile(isolate: &'a isolate::Isolate,
                   context: &context::Context,
                   source: &value::String)
                   -> error::Result<Script<'a>> {
        unsafe {
            Ok(Script(isolate,
                      try!(util::invoke(isolate, |c| {
                          v8::Script_Compile(c, context.as_raw(), source.as_raw())
                      }))))
        }
    }

    pub fn run(&self, context: &context::Context) -> error::Result<Option<value::Value>> {
        unsafe {
            Ok(try!(util::invoke_nullable(self.0, |c| v8::Script_Run(c, self.1, context.as_raw())))
                .map(|p| value::Value::from_raw(self.0, p)))
        }
    }
}

impl<'a> Drop for Script<'a> {
    fn drop(&mut self) {
        unsafe { v8::Script_DestroyRef(self.1) }
    }
}
