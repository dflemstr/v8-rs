use v8_sys as v8;
use error;
use isolate;
use util;

pub struct Context(*mut v8::Context);

impl Context {
    pub fn new(isolate: &isolate::Isolate) -> error::Result<Context> {
        unsafe { Ok(Context(try!(util::invoke(isolate, |c| v8::Context_New(c))))) }
    }

    pub fn as_raw(&self) -> *mut v8::Context {
        self.0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { v8::Context_Destroy(self.0) }
    }
}
