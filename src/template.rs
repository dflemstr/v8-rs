use v8_sys as v8;
use isolate;
use util;
use value;
use context;
use std::os;
use std::ptr;

#[derive(Debug)]
pub struct Template<'a>(&'a isolate::Isolate, v8::TemplateRef);

#[derive(Debug)]
pub struct FunctionTemplate<'a>(&'a isolate::Isolate, v8::FunctionTemplateRef);

#[derive(Debug)]
pub struct ObjectTemplate<'a>(&'a isolate::Isolate, v8::ObjectTemplateRef);

/// An ObjectTemplate is used to create objects at runtime.
///
/// Properties added to an ObjectTemplate are added to each object created from the ObjectTemplate.
impl<'a> ObjectTemplate<'a> {
    pub fn new(isolate: &'a isolate::Isolate) -> ObjectTemplate<'a> {
        let raw = unsafe {
            util::invoke(isolate,
                         |c| v8::ObjectTemplate_New(c, isolate.as_raw(), ptr::null_mut()))
                .unwrap()
        };
        ObjectTemplate(isolate, raw)
    }

    pub fn set_internal_field_count(&self, value: usize) {
        unsafe {
            util::invoke(self.0, |c| {
                    v8::ObjectTemplate_SetInternalFieldCount(c, self.1, value as os::raw::c_int)
                })
                .unwrap()
        };
    }

    pub fn new_instance(&self, context: &context::Context) -> value::Object {
        unsafe {
            let raw = util::invoke(self.0,
                                   |c| v8::ObjectTemplate_NewInstance(c, self.1, context.as_raw()))
                .unwrap();
            value::Object::from_raw(self.0, raw)
        }
    }

    /// Creates an object template from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &'a isolate::Isolate,
                           raw: v8::ObjectTemplateRef)
                           -> ObjectTemplate<'a> {
        ObjectTemplate(isolate, raw)
    }

    /// Returns the underlying raw pointer behind this object template.
    pub fn as_raw(&self) -> v8::ObjectTemplateRef {
        self.1
    }
}

drop!(Template, v8::Template_DestroyRef);
drop!(FunctionTemplate, v8::FunctionTemplate_DestroyRef);
drop!(ObjectTemplate, v8::ObjectTemplate_DestroyRef);
