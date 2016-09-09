use v8_sys as v8;

use isolate;

pub struct Context(*mut v8::Isolate, *mut v8::Context);

impl Context {
    pub fn new(isolate: &isolate::Isolate) -> Context {
        unsafe { Context(isolate.as_raw(), v8::Context_New(isolate.as_raw())) }
    }

    pub fn as_raw(&self) -> *mut v8::Context {
        self.1
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { v8::Context_Destroy(self.0, self.1) }
    }
}
