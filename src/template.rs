use v8_sys as v8;
use isolate;
use util;
use value;
use value::Data;
use context;
use std::os;
use std::ptr;
use std::mem;
use std::ops;
use std::ffi;

#[derive(Debug)]
pub struct Template<'a>(&'a isolate::Isolate, v8::TemplateRef);

#[derive(Debug)]
pub struct FunctionTemplate<'a>(&'a isolate::Isolate, v8::FunctionTemplateRef);

#[derive(Debug)]
pub struct ObjectTemplate<'a>(&'a isolate::Isolate, v8::ObjectTemplateRef);

#[derive(Debug)]
pub struct Signature<'a>(&'a isolate::Isolate, v8::SignatureRef);

/// A Signature specifies which receiver is valid for a function.
impl<'a> Signature<'a> {
    pub fn new(isolate: &'a isolate::Isolate) -> Signature<'a> {
        let raw = unsafe {
            util::invoke(isolate,
                         |c| v8::Signature_New(c, isolate.as_raw(), ptr::null_mut()))
                .unwrap()
        };
        Signature(isolate, raw)
    }

    pub fn new_with_receiver(isolate: &'a isolate::Isolate, receiver: &FunctionTemplate) -> Signature<'a> {
        let raw = unsafe {
            util::invoke(isolate,
                         |c| v8::Signature_New(c, isolate.as_raw(), receiver.1))
                .unwrap()
        };
        Signature(isolate, raw)
    }
}

/// A FunctionTemplate is used to create functions at runtime. 
/// There can only be one function created from a FunctionTemplate in a context.
///
/// Any modification of a FunctionTemplate after first instantiation will trigger a crash.
/// A FunctionTemplate can have properties, these properties are added to the function object when it is created.
impl<'a> FunctionTemplate<'a> {
    pub fn new<'b>(isolate: &'a isolate::Isolate,
               context: &context::Context<'a>,
               callback: &'b Fn(&'a value::FunctionCallbackInfo) -> value::Value<'a>)
                -> FunctionTemplate<'a> {
        let raw = unsafe {
            let callback = Box::into_raw(Box::new(callback));
            let template = ObjectTemplate::new(isolate);
            template.set_internal_field_count(1);
            let closure = template.new_instance(context);
            closure.set_aligned_pointer_in_internal_field(0, callback);

            util::invoke(isolate,
                         |c| v8::FunctionTemplate_New(c,
                                                      context.as_raw(),
                                                      Some(util::callback),
                                                      (&closure as &value::Value).as_raw(),
                                                      ptr::null_mut(),
                                                      0,
                                                      v8::ConstructorBehavior::ConstructorBehavior_kAllow))
                .unwrap()
        };
        FunctionTemplate(isolate, raw)
    }

    pub fn get_function(&self, context: &context::Context) -> value::Function<'a> {
        unsafe {
            let raw = util::invoke(self.0,
                                   |c| v8::FunctionTemplate_GetFunction(c, self.1, context.as_raw()))
                .unwrap();
            value::Function::from_raw(self.0, raw)
        }
    }
}


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

    pub fn set(&self, name: &str, value: &value::Data) {
        let cname = ffi::CString::new(name).unwrap();
        let template: &Template = self;
        unsafe {
            util::invoke(self.0, |c| {
                v8::Template_Set_Raw(c, template.1, self.0.as_raw(), cname.as_ptr(), value.as_raw())
            })
            .unwrap()  
        };
    }

    pub fn new_instance(&self, context: &context::Context) -> value::Object<'a> {
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

inherit!(Template, Data);
inherit!(ObjectTemplate, Template);
inherit!(FunctionTemplate, Template);
inherit!(Signature, Data);

drop!(Template, v8::Template_DestroyRef);
drop!(FunctionTemplate, v8::FunctionTemplate_DestroyRef);
drop!(ObjectTemplate, v8::ObjectTemplate_DestroyRef);
drop!(Signature, v8::Signature_DestroyRef);
