use v8_sys as v8;
use isolate;
use util;
use value;

#[derive(Debug)]
pub struct Context<'a>(&'a isolate::Isolate, v8::ContextRef);

impl<'a> Context<'a> {
    pub fn new(isolate: &isolate::Isolate) -> Context {
        unsafe {
            Context(isolate,
                    util::invoke(isolate, |c| v8::Context_New(c)).unwrap())
        }
    }

    pub fn global(&self) -> value::Object<'a> {
        unsafe {
            value::Object::from_raw(self.0,
                                    util::invoke(self.0, |c| v8::Context_Global(c, self.1))
                                        .unwrap())
        }
    }

    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8::ContextRef) -> Context {
        Context(isolate, raw)
    }

    pub fn as_raw(&self) -> v8::ContextRef {
        self.1
    }
}

impl<'a> Drop for Context<'a> {
    fn drop(&mut self) {
        unsafe { v8::Context_DestroyRef(self.1) }
    }
}
