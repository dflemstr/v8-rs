//! Script and source code compilation, execution, origins and management.
use v8_sys as v8;

use context;
use error;
use isolate;
use value;
use util;

/// A compiled JavaScript script, tied to a Context which was active when the script was compiled.
#[derive(Debug)]
pub struct Script(isolate::Isolate, v8::ScriptRef);

impl Script {
    /// Compiles the specified source code into a compiled script.
    pub fn compile(isolate: &isolate::Isolate,
                   context: &context::Context,
                   source: &value::String)
                   -> error::Result<Script> {
        let raw = unsafe {
            try!(util::invoke_ctx(isolate,
                                  context,
                                  |c| v8::v8_Script_Compile(c, context.as_raw(), source.as_raw())))
        };
        Ok(Script(isolate.clone(), raw))
    }

    /// Compiles the specified source code into a compiled script.
    ///
    /// The specified name will be reported as the script's origin and will show up in stack traces,
    /// for example.
    pub fn compile_with_name(isolate: &isolate::Isolate,
                             context: &context::Context,
                             name: &value::Value,
                             source: &value::String)
                             -> error::Result<Script> {
        use std::ptr::null_mut as n;
        let raw = unsafe {
            try!(util::invoke_ctx(isolate, context, |c| {
                v8::v8_Script_Compile_Origin(c,
                                          context.as_raw(),
                                          source.as_raw(),
                                          name.as_raw(),
                                          n(),
                                          n(),
                                          n(),
                                          n(),
                                          n(),
                                          n(),
                                          n())
            }))
        };
        Ok(Script(isolate.clone(), raw))
    }

    /// Runs this script in the specified context.
    ///
    /// If the script returns a value, meaning that the last line of the script evaluates to an
    /// expression or there is an explicit return, that value will be returned from this method.  If
    /// the script throws an exception, that will reslt in this method also throwing an exception.
    pub fn run(&self, context: &context::Context) -> error::Result<value::Value> {
        unsafe {
            let raw = try!(util::invoke_ctx(&self.0,
                                            context,
                                            |c| v8::v8_Script_Run(c, self.1, context.as_raw())));
            Ok(value::Value::from_raw(&self.0, raw))
        }
    }
}

reference!(Script, v8::v8_Script_CloneRef, v8::v8_Script_DestroyRef);
