use v8_sys as v8;
use isolate;
use util;

pub struct Context(v8::ContextRef);

impl Context {
    pub fn new(isolate: &isolate::Isolate) -> Context {
        unsafe { Context(util::invoke(isolate, |c| v8::Context_New(c)).unwrap()) }
    }

    pub fn as_raw(&self) -> v8::ContextRef {
        self.0
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { v8::Context_DestroyRef(self.0) }
    }
}
